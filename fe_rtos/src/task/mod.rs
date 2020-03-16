extern crate alloc;
extern crate crossbeam_queue;

mod task_state;
mod tick;

use crate::syscall;
use crate::task::task_state::{TaskState, TaskStateStruct};
use crate::task::tick::TickCounter;
use alloc::collections::LinkedList;
use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, Ordering};
use crossbeam_queue::SegQueue;
use fe_osi;
use fe_osi::semaphore::Semaphore;

#[repr(C)]
union StackPtr {
    reference: &'static u32,
    ptr: *const u32,
    num: u32,
}

#[repr(C)]
pub (crate) struct Task {
    //Stack pointer
    sp: StackPtr,
    //Entry point into task
    //Holds the smart pointer to a dynamically allocated stack is applicable
    //It needs to be a smart type to easily allow for deallocation of the stack
    //if a task is deleted
    dynamic_stack: Vec<u32>,
    //Stores the param of the task if it was created with the task_spawn syscall
    //This will be a raw pointer to a Box that will need to be freed
    task_info: Option<Box<NewTaskInfo>>,
    state: TaskStateStruct,
    queued: AtomicBool,
}

unsafe impl Send for Task {}
unsafe impl Sync for Task {}

pub (crate) struct NewTaskInfo {
    pub ep: *const u32,
    pub param: *mut u32,
}

const INIT_XPSR: u32 = 0x01000000;
pub const DEFAULT_STACK_SIZE: usize = 1024;

static mut KERNEL_STACK: [u32; DEFAULT_STACK_SIZE] = [0; DEFAULT_STACK_SIZE];
static mut TICKS: TickCounter = TickCounter::new();
static PUSHING_TASK: AtomicBool = AtomicBool::new(false);
static mut PLACEHOLDER_TASK: Task = Task {
    sp: StackPtr { num: 0 },
    dynamic_stack: Vec::new(),
    task_info: None,
    state: TaskStateStruct::new(),
    queued: AtomicBool::new(false),
};
static mut KERNEL_TASK: Option<Arc<Task>> = None;
static mut NEXT_TASK: Option<Arc<Task>> = None;
static mut CUR_TASK: &mut Task = unsafe { &mut PLACEHOLDER_TASK };
lazy_static! {
    static ref NEW_TASK_QUEUE: SegQueue<Arc<Task>> = { SegQueue::new() };
    static ref SCHEDULER_QUEUE: SegQueue<Arc<Task>> = { SegQueue::new() };
}

static mut TRIGGER_CONTEXT_SWITCH: fn() = idle;

extern "C" {
    pub fn context_switch();
}

#[no_mangle]
pub (crate) unsafe extern "C" fn get_cur_task() -> &'static Task {
    CUR_TASK
}

#[no_mangle]
pub (crate) unsafe extern "C" fn set_cur_task(new_val: &'static mut Task) {
    CUR_TASK = new_val;
}

#[no_mangle]
pub (crate) unsafe extern "C" fn get_next_task() -> &'static Task {
    match &NEXT_TASK {
        Some(task) => task,
        None => &PLACEHOLDER_TASK,
    }
}

unsafe fn scheduler() {
    let default_task = match &KERNEL_TASK {
        Some(task) => Arc::clone(&task),
        //The code should never get here
        None => panic!("No valid KERNEL_TASK in scheduler"),
    };

    //If a task is being currently being pushed by the kernel, run it so it can finish
    if PUSHING_TASK.load(Ordering::SeqCst) {
        NEXT_TASK = Some(Arc::clone(&default_task));
        return;
    }

    //Find the next task to run if there is one
    match SCHEDULER_QUEUE.pop() {
        Ok(task) => {
            let task_state = task.state.try_get().unwrap_or(TaskState::Ignore);

            match task_state {
                TaskState::Runnable => {
                    NEXT_TASK = Some(Arc::clone(&task));
                },
                _ => {
                    //If the popped task is not runnable, we should still signal that
                    //it is no longer queud
                    task.queued.store(false, Ordering::SeqCst);
                    NEXT_TASK = Some(Arc::clone(&default_task));
                },
            }
        },
        Err(_) => {
            NEXT_TASK = Some(Arc::clone(&default_task));
        },
    }

    match &NEXT_TASK {
        Some(task) => {
            task.queued.store(false, Ordering::SeqCst);
        },
        //The code shouldn't get here
        None => {
            panic!("No valid NEXT_TASK at the end of scheduling");
        },
    }
}

unsafe fn do_context_switch() {
    super::disable_interrupts();
    scheduler();
    super::enable_interrupts();
    TRIGGER_CONTEXT_SWITCH();
}

pub unsafe extern "C" fn sys_tick() {
    TICKS.inc();
    do_context_switch();
}

//Puts the currently running thread to sleep for at least the specified number
//of ticks
pub (crate) fn sleep(sleep_ticks: u64) -> bool{
    unsafe {
        let ret_val = CUR_TASK.state.try_set(TaskState::Asleep(TICKS.get() + sleep_ticks));
        //Trigger a context switch and wait until that happens
        do_context_switch();

        ret_val
    }
}

//Has the currently running thread block until the semaphore it's blocking on
//is available
pub (crate) fn block(sem: *const Semaphore) -> bool{
    unsafe {
        let ret_val = CUR_TASK.state.try_set(TaskState::Blocking(sem));
        do_context_switch();

        ret_val
    }
}

//See section 2.5.7.1 and Figure 2-7 in the TM4C123 datasheet for more information
unsafe fn set_initial_stack(stack_ptr: *const u32, entry_point: *const u32, param: *mut u32) -> *const u32 {
    let mut cur_ptr = ((stack_ptr as u32) - 4) as *mut u32;

    //Set the xPSR
    *cur_ptr = INIT_XPSR;
    cur_ptr = (cur_ptr as u32 - 4) as *mut u32;

    //Set the PC
    *cur_ptr = entry_point as u32;
    cur_ptr = (cur_ptr as u32 - 4) as *mut u32;

    //Set the LR
    *cur_ptr = idle as u32;
    cur_ptr = (cur_ptr as u32 - 4) as *mut u32;

    for _i in 0..13 {
        *cur_ptr = param as u32;
        cur_ptr = (cur_ptr as u32 - 4) as *mut u32;
    }
    (cur_ptr as u32 + 4) as *const u32
}

pub (crate) unsafe fn add_task(stack_size: usize, entry_point: *const u32, param: *mut u32,) -> Arc<Task> {
    let mut new_task = Task {
        sp: StackPtr { num: 0 },
        dynamic_stack: vec![0; stack_size],
        task_info: None,
        state: TaskStateStruct::new(),
        queued: AtomicBool::new(false),
    };

    //Convert the adress of the first element of the vector into a ptr for the stack
    new_task.sp.ptr = &new_task.dynamic_stack[0] as *const u32;
    //Arm uses a full descending stack so we have to start from the top
    new_task.sp.num += 4 * (stack_size as u32);

    new_task.sp.ptr = set_initial_stack(new_task.sp.ptr, entry_point, param);

    let task_ref = Arc::new(new_task);

    NEW_TASK_QUEUE.push(Arc::clone(&task_ref));

    task_ref
}

pub (crate) unsafe fn add_task_static(stack_ptr: &'static u32, stack_size: usize, entry_point: *const u32, param: Option<*mut u32>) -> Arc<Task> {
    let mut new_task = Task {
        sp: StackPtr { reference: stack_ptr, },
        dynamic_stack: Vec::new(),
        task_info: None,
        state: TaskStateStruct::new(),
        queued: AtomicBool::new(false),
    };

    //Arm uses a full descending stack so we have to start from the top
    new_task.sp.num += 4 * (stack_size as u32);
    let param_ptr = match param {
        Some(p) => p as *mut u32,
        None => 0 as *mut u32
    };

    new_task.sp.ptr = set_initial_stack(new_task.sp.ptr, entry_point, param_ptr);

    let task_ref = Arc::new(new_task);

    NEW_TASK_QUEUE.push(Arc::clone(&task_ref));

    task_ref
}

//This function is called by the task spawn syscall.
//This function handles cleaning up after a task when
//it returns
pub (crate) fn new_task_helper(task_info: Box<NewTaskInfo>) -> ! {
    let task_param : &u32 = unsafe { &*task_info.param };
    let task_ep : fn(&u32) = unsafe { core::mem::transmute(task_info.ep) };
    let task = unsafe { &mut CUR_TASK };

    task.task_info = Some(task_info);

    task_ep(task_param);

    fe_osi::exit();

    loop{}
}

pub (crate) unsafe fn remove_task() -> bool {
    let ret_val = CUR_TASK.state.try_set(TaskState::Zombie);
    do_context_switch();

    ret_val
}

pub fn start_scheduler(
    trigger_context_switch: fn(),
    mut systick: cortex_m::peripheral::SYST,
    reload_val: u32,
) {
    syscall::link_syscalls();

    unsafe {
        let kernel_task_ref = add_task_static(&KERNEL_STACK[0], DEFAULT_STACK_SIZE, kernel as *const u32, None);
        //Add something to the scheduler queue so something can run right away
        kernel_task_ref.queued.store(true, Ordering::SeqCst);
        KERNEL_TASK = Some(Arc::clone(&kernel_task_ref));
        SCHEDULER_QUEUE.push(kernel_task_ref);
        TRIGGER_CONTEXT_SWITCH = trigger_context_switch;
    }

    unsafe {
        let mut peripherals = cortex_m::Peripherals::steal();
        peripherals.SCB.set_priority(cortex_m::peripheral::scb::SystemHandler::PendSV, 7);
    }

    //Basically, wait for the scheduler to start
    systick.set_reload(reload_val);
    systick.clear_current();
    systick.enable_counter();
    systick.enable_interrupt();

    loop {}
}

fn kernel(_: &mut u32) {
    let mut task_list: LinkedList<Arc<Task>> = LinkedList::new();

    loop {
        let mut task_num: usize = 0;
        let mut deleted_task_num: usize = 0;
        let mut delete_task = false;

        //Add all new tasks to the task list
        while !NEW_TASK_QUEUE.is_empty() {
            match NEW_TASK_QUEUE.pop() {
                Ok(new_task) => { task_list.push_back(new_task); }
                Err(_) => { break; },
            }
        }

        for task in task_list.iter() {
            let mut new_state = TaskState::Runnable;
            let mut has_new_state = false;
            let task_state = task.state.try_get().unwrap_or(TaskState::Ignore);

            match task_state {
                TaskState::Runnable => {
                    if task.queued.compare_and_swap(false, true, Ordering::SeqCst) == false {
                        PUSHING_TASK.store(true, Ordering::SeqCst);
                        SCHEDULER_QUEUE.push(Arc::clone(&task));
                        PUSHING_TASK.store(false, Ordering::SeqCst);
                    } else if SCHEDULER_QUEUE.is_empty() {
                        //If the task is marked as queued, but the queue is empty,
                        //unmark it as queued.
                        task.queued.store(false, Ordering::SeqCst);
                    }
                },
                TaskState::Asleep(wake_up_ticks) => {
                    unsafe {
                        //If this task is asleep, see if we should wake it up
                        if wake_up_ticks < TICKS.get() {
                            //If we wake the task up, we can run it
                            has_new_state = true;
                            new_state = TaskState::Runnable;
                        }
                    }
                }
                TaskState::Blocking(sem) => {
                    let sem_ref: &Semaphore = unsafe { &*sem };
                    //If the semaphore this task is blocking on is available
                    //to be taken, wake up the task so it can attempt to take it
                    if sem_ref.is_available() {
                        has_new_state = true;
                        new_state = TaskState::Runnable;
                    }
                }
                TaskState::Zombie => {
                    delete_task = true;
                    deleted_task_num = task_num;
                    continue;
                },
                TaskState::Ignore => {},
            }

            if has_new_state {
                //If this fails, the kernel will just take care of it in the next run through
                task.state.try_set(new_state);
            }

            task_num += 1;
        }

        //Delete a task if there's a task to be deleted
        if delete_task {
            let mut split_list = task_list.split_off(deleted_task_num);

            match split_list.pop_front() {
                Some(task) => {
                    //If this task has been removed, remove it from the list
                    //and rust will dealloc everything
                    match &task.task_info {
                        Some(info) => {
                            if !info.param.is_null() {
                                unsafe { core::mem::drop(Box::from_raw(info.param)); }
                            }
                        },
                        None => (),
                    }
                },
                None => (),
            }

            //Put the task list back together
            task_list.append(&mut split_list);
        }

        //Going through the loop multiple times without anything else running is
        //useless since their states will not have changed, so do a context switch
        unsafe { do_context_switch(); }
    }
}

fn idle() {
    loop {}
}
