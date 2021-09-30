extern crate alloc;
extern crate crossbeam_queue;

mod schedule;
mod task_state;
mod tick;

use crate::arch;
use crate::spinlock::Spinlock;
use crate::syscall;
use crate::task::schedule::{RoundRobin, Scheduler};
use crate::task::task_state::{TaskState, TaskStateStruct};
use crate::task::tick::TickCounter;
use alloc::boxed::Box;
use alloc::collections::LinkedList;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::mem::size_of;
use core::ptr::null_mut;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use crossbeam_queue::SegQueue;
use fe_osi::semaphore::Semaphore;

#[repr(C)]
union StackPtr {
    reference: &'static usize,
    ptr: *const usize,
    num: usize,
}

#[repr(C)]
pub(crate) struct Task {
    //Stack pointer
    sp: StackPtr,
    //Reference to the stack canary
    canary: &'static usize,
    //Entry point into task
    //Holds the smart pointer to a dynamically allocated stack is applicable
    //It needs to be a smart type to easily allow for deallocation of the stack
    //if a task is deleted
    dynamic_stack: Vec<usize>,
    //Stores the param of the task if it was created with the task_spawn syscall
    //This will be a raw pointer to a Box that will need to be freed
    task_info: Option<Box<NewTaskInfo>>,
    state: TaskStateStruct,
    pub(crate) pid: usize,
}

unsafe impl Send for Task {}
unsafe impl Sync for Task {}

impl Task {
    pub(crate) fn give(&self) {
        if let Some(task_info) = &self.task_info {
            task_info.sem.give();
        }
    }
}

pub(crate) struct NewTaskInfo {
    pub ep: *const u32,
    pub param: *mut u32,
    pub sem: Arc<Semaphore>,
}

pub(crate) const STACK_CANARY: usize = 0xC0DE5AFE;
/// The default and recommended stack size for a task.
pub const DEFAULT_STACK_SIZE: usize = 1536;

static mut KERNEL_STACK: [usize; DEFAULT_STACK_SIZE] = [0; DEFAULT_STACK_SIZE];
static mut TICKS: TickCounter = TickCounter::new();
static mut PLACEHOLDER_TASK: Task = Task {
    sp: StackPtr { num: 0 },
    canary: &STACK_CANARY,
    dynamic_stack: Vec::new(),
    task_info: None,
    state: TaskStateStruct::new(),
    pid: 0,
};
static PUSHING_TASK: AtomicBool = AtomicBool::new(false);
static mut SCHEDULER: RoundRobin = RoundRobin::new();
static mut KERNEL_TASK: Option<Arc<Task>> = None;
static mut NEXT_TASK: Option<Arc<Task>> = None;
static mut CUR_TASK: Option<&mut Task> = None;
lazy_static! {
    static ref NEW_TASK_QUEUE: SegQueue<Arc<Task>> = SegQueue::new();
}

unsafe fn get_cur_task_mut() -> &'static mut Task {
    match &mut CUR_TASK {
        Some(task) => task,
        None => &mut PLACEHOLDER_TASK,
    }
}

#[no_mangle]
pub(crate) unsafe extern "C" fn get_cur_task() -> &'static Task {
    match &CUR_TASK {
        Some(task) => task,
        None => &PLACEHOLDER_TASK,
    }
}

#[no_mangle]
pub(crate) unsafe extern "C" fn set_cur_task(new_val: &'static mut Task) {
    CUR_TASK = Some(new_val);
}

#[no_mangle]
pub(crate) unsafe extern "C" fn get_next_task() -> &'static Task {
    match &NEXT_TASK {
        Some(task) => task,
        None => &PLACEHOLDER_TASK,
    }
}

unsafe fn scheduler() {
    let default_task = match &KERNEL_TASK {
        Some(task) => Arc::clone(task),
        //The code should never get here
        None => panic!("No valid KERNEL_TASK in scheduler"),
    };

    //Make sure we don't accidentally drop a task
    let count = match &NEXT_TASK {
        Some(task) => Arc::strong_count(task),
        None => 0xFF,
    };
    if count == 1 {
        return;
    }

    //Find the next task to run if there is one
    match SCHEDULER.next() {
        Some(task) => {
            NEXT_TASK = Some(task);
        }
        None => {
            NEXT_TASK = Some(Arc::clone(&default_task));
        }
    }
}

pub(crate) unsafe fn do_context_switch() {
    static CS_LOCK: Spinlock = Spinlock::new();

    if CS_LOCK.try_take() {
        scheduler();
        CS_LOCK.give();
        arch::trigger_context_switch();
    }
}

pub(crate) unsafe extern "C" fn sys_tick() {
    TICKS.inc();
    do_context_switch();
}

//Puts the currently running thread to sleep for at least the specified number
//of ticks
pub(crate) fn sleep(sleep_ticks: u64) -> bool {
    unsafe {
        let ret_val = get_cur_task()
            .state
            .try_set(TaskState::Asleep(TICKS.get() + sleep_ticks));
        //Trigger a context switch and wait until that happens
        do_context_switch();

        ret_val
    }
}

//Has the currently running thread block until the semaphore it's blocking on
//is available
pub(crate) fn block(sem: *const Semaphore) -> bool {
    unsafe {
        let ret_val = get_cur_task().state.try_set(TaskState::Blocking(sem));
        do_context_switch();

        ret_val
    }
}

fn get_new_pid() -> usize {
    static PID: AtomicUsize = AtomicUsize::new(1);
    PID.fetch_add(1, Ordering::SeqCst)
}

pub(crate) unsafe fn add_task(
    stack_size: usize,
    entry_point: *const usize,
    param: *mut usize,
) -> Arc<Task> {
    let mut sp = StackPtr { num: 0 };
    let stack = vec![0; stack_size];
    //Convert the adress of the first element of the vector into a ptr for the stack
    sp.ptr = &stack[0] as *const usize;
    let stack_size_bytes = size_of::<usize>() * stack_size;

    let canary: &'static usize = &*arch::set_canary(sp.ptr, stack_size_bytes);
    sp.ptr = arch::set_initial_stack(sp.ptr, stack_size_bytes, entry_point, param);

    let new_task = Task {
        sp,
        canary,
        dynamic_stack: stack,
        task_info: None,
        state: TaskStateStruct::new(),
        pid: get_new_pid(),
    };

    let task_ref = Arc::new(new_task);

    PUSHING_TASK.store(true, Ordering::SeqCst);
    NEW_TASK_QUEUE.push(Arc::clone(&task_ref));
    PUSHING_TASK.store(false, Ordering::SeqCst);

    task_ref
}

pub(crate) unsafe fn add_task_static(
    stack_ptr: &'static usize,
    stack_size: usize,
    entry_point: *const usize,
    param: Option<*mut usize>,
) -> Arc<Task> {
    let mut sp = StackPtr {
        reference: stack_ptr,
    };
    let stack_size_bytes = size_of::<usize>() * stack_size;
    let param_ptr = match param {
        Some(p) => p as *mut usize,
        None => null_mut(),
    };

    let canary: &'static usize = &*arch::set_canary(sp.ptr, stack_size_bytes);
    sp.ptr = arch::set_initial_stack(sp.ptr, stack_size_bytes, entry_point, param_ptr);

    let new_task = Task {
        sp,
        canary,
        dynamic_stack: Vec::new(),
        task_info: None,
        state: TaskStateStruct::new(),
        pid: get_new_pid(),
    };

    let task_ref = Arc::new(new_task);

    PUSHING_TASK.store(true, Ordering::SeqCst);
    NEW_TASK_QUEUE.push(Arc::clone(&task_ref));
    PUSHING_TASK.store(false, Ordering::SeqCst);

    task_ref
}

//This function is called by the task spawn syscall.
//This function handles cleaning up after a task when
//it returns
pub(crate) fn new_task_helper(task_info: Box<NewTaskInfo>) -> ! {
    let task_param: &u32 = unsafe { &*task_info.param };
    let task_ep: fn(&u32) = unsafe { core::mem::transmute(task_info.ep) };
    let task = unsafe { get_cur_task_mut() };

    task.task_info = Some(task_info);

    task_ep(task_param);

    unsafe {
        fe_osi::exit();
    }

    loop {}
}

pub(crate) unsafe fn remove_task() -> bool {
    let ret_val = get_cur_task().state.try_set(TaskState::Zombie);
    do_context_switch();

    ret_val
}

/// Starts the FeRTOS scheduler to begin executing tasks.
///
/// trigger_context_switch is a pointer to a function that forces a context switch
/// enable_systic is a closure that enables the systick interrupt
/// reload_val is the number of counts on the systick counter in a tick
pub fn start_scheduler<F: FnOnce(usize)>(enable_systick: F, reload_val: usize) {
    syscall::link_syscalls();

    unsafe {
        let kernel_task_ref = add_task_static(
            &KERNEL_STACK[0],
            DEFAULT_STACK_SIZE,
            kernel as *const usize,
            None,
        );
        //Add something to the scheduler queue so something can run right away
        KERNEL_TASK = Some(Arc::clone(&kernel_task_ref));
    }

    enable_systick(reload_val);

    loop {}
}

fn kernel(_: &mut u32) {
    let mut first_push = true;
    let mut task_list: LinkedList<Arc<Task>> = LinkedList::new();

    loop {
        let mut task_num: usize = 0;
        let mut deleted_task_num: usize = 0;
        let mut delete_task = false;

        //Make sure the kernel gets added to the scheduler when it starts
        if first_push {
            unsafe {
                arch::disable_interrupts();
            }
        }
        //Add all new tasks to the task list
        while !NEW_TASK_QUEUE.is_empty() && !PUSHING_TASK.load(Ordering::SeqCst) {
            match NEW_TASK_QUEUE.pop() {
                Ok(new_task) => {
                    task_list.push_back(Arc::clone(&new_task));
                    unsafe {
                        SCHEDULER.add_task(new_task);
                    }
                }
                Err(_) => {
                    break;
                }
            }
        }
        if first_push {
            first_push = false;
            unsafe {
                arch::enable_interrupts();
            }
        }

        for task in task_list.iter() {
            let mut new_state = TaskState::Runnable;
            let mut has_new_state = false;

            //Check for stack overflow
            if *task.canary != STACK_CANARY {
                //We can't panic because the task stacks are in the heap
                //so if there's an overflow the heap is all screwed up
                let mut digits: [u8; 20] = [0; 20];
                let start_idx = fe_osi::usize_to_chars(&mut digits, task.pid);
                fe_osi::print_msg("Stack overflow in task ");
                fe_osi::print_msg(core::str::from_utf8(&digits[start_idx..]).unwrap());
                fe_osi::print_msg("!\n");
                unsafe {
                    fe_osi::exit();
                }
            }

            //We want to default to Runnable because if a task is in a transition state,
            //it should be scheduled so it can finish transitioning.
            let task_state = task.state.try_get().unwrap_or(TaskState::Runnable);

            match task_state {
                TaskState::Runnable => {}
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
                }
            }

            if has_new_state {
                //If this fails, the kernel will just take care of it in the next run through
                task.state.try_set(new_state);
            }

            task_num += 1;
        }

        //Delete a task if there's a task to be deleted
        if delete_task {
            //If this task has been removed, remove it from the list
            //and rust will dealloc almost eveything
            let removed_task = task_list.remove(deleted_task_num);

            //We still need to manually dealloc the parameter though
            match &removed_task.task_info {
                Some(info) => {
                    if !info.param.is_null() {
                        unsafe {
                            core::mem::drop(Box::from_raw(info.param));
                        }
                    }
                }
                None => (),
            }

            unsafe {
                SCHEDULER.remove_task(removed_task.pid);
            }
        }

        //Going through the loop multiple times without anything else running is
        //useless since their states will not have changed, so do a context switch
        unsafe {
            do_context_switch();
        }
    }
}

pub(crate) fn idle() {
    loop {}
}
