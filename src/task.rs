extern crate alloc;

use crate::syscall;
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use fe_osi;
use fe_osi::semaphore::Semaphore;

#[repr(C)]
union StackPtr {
    reference: &'static u32,
    ptr: *const u32,
    num: u32,
}

enum TaskState {
    Runnable,
    Asleep(u64),
    Blocking(*const Semaphore),
    Zombie,
}

#[repr(C)]
pub struct Task {
    //Stack pointer
    sp: StackPtr,
    //Entry point into task
    //Holds the smart pointer to a dynamically allocated stack is applicable
    //It needs to be a smart type to easily allow for deallocation of the stack
    //if a task is deleted
    dynamic_stack: Vec<u32>,
    //Stores the param of the task if it was created with the task_spawn syscall
    //This will be a raw pointer to a Box that will need to be freed
    param: *mut u32,
    state: TaskState,
}

pub struct NewTaskInfo {
    pub ep: *const u32,
    pub param: *mut u32,
}

const INIT_XPSR: u32 = 0x01000000;
pub const DEFAULT_STACK_SIZE: usize = 512;

static mut KERNEL_STACK: [u32; DEFAULT_STACK_SIZE] = [0; DEFAULT_STACK_SIZE];
static mut TASKS: Vec<Task> = Vec::new();
static mut TICKS: u64 = 0;
static mut TASK_INDEX: usize = 0;
static mut KERNEL_TASK: Task = Task {
    sp: StackPtr { num: 0 },
    dynamic_stack: Vec::new(),
    param: core::ptr::null_mut(),
    state: TaskState::Runnable,
};
static mut NEXT_TASK: &Task = unsafe { &KERNEL_TASK };
static mut CUR_TASK: &Task = unsafe { &KERNEL_TASK };

static mut TRIGGER_CONTEXT_SWITCH: fn() = idle;

extern "C" {
    pub fn context_switch();
}

#[no_mangle]
pub unsafe extern "C" fn get_cur_task() -> &'static Task {
    CUR_TASK
}

#[no_mangle]
pub unsafe extern "C" fn set_cur_task(new_val: &'static Task) {
    CUR_TASK = new_val;
}

#[no_mangle]
pub unsafe extern "C" fn get_next_task() -> &'static Task {
    NEXT_TASK
}

unsafe fn scheduler() {
    let cur_index = TASK_INDEX;

    //Loop through the tasks to find the next runnable task to execute.
    //We don't have to worry about no tasks being runnable since the kernel task
    //should never block
    loop {
        TASK_INDEX += 1;
        if TASK_INDEX >= TASKS.len() {
            TASK_INDEX = 0;
        }

        let task = &mut TASKS[TASK_INDEX];

        match task.state {
            TaskState::Runnable => {
                //If the task is awake, great! let's run it
                break;
            }
            _ => {}
        }

        //If TASK_INDEX is equal to cur_index that means all of the threads
        //are sleeping
        if TASK_INDEX == cur_index {
            break;
        }
    }

    NEXT_TASK = &TASKS[TASK_INDEX];
}

unsafe fn do_context_switch() {
    scheduler();
    TRIGGER_CONTEXT_SWITCH();
}

pub unsafe extern "C" fn sys_tick() {
    TICKS += 1;
    do_context_switch();
}

//Puts the currently running thread to sleep for at least the specified number
//of ticks
pub fn sleep(sleep_ticks: u64) {
    unsafe {
        TASKS[TASK_INDEX].state = TaskState::Asleep(TICKS + sleep_ticks);
        //Trigger a context switch and wait until that happens
        do_context_switch();
    }
}

//Has the currently running thread block until the semaphore it's blocking on
//is available
pub fn block(sem: *const Semaphore) {
    unsafe {
        TASKS[TASK_INDEX].state = TaskState::Blocking(sem);
        do_context_switch();
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

pub unsafe fn add_task(stack_size: usize, entry_point: *const u32, param: *mut u32,) -> bool {
    let mut new_task = Task {
        sp: StackPtr { num: 0 },
        dynamic_stack: vec![0; stack_size],
        param: core::ptr::null_mut(),
        state: TaskState::Runnable,
    };

    //Convert the adress of the first element of the vector into a ptr for the stack
    new_task.sp.ptr = &new_task.dynamic_stack[0] as *const u32;
    //Arm uses a full descending stack so we have to start from the top
    new_task.sp.num += 4 * (stack_size as u32);

    new_task.sp.ptr = set_initial_stack(new_task.sp.ptr, entry_point, param);

    TASKS.push(new_task);

    true
}

pub unsafe fn add_task_static(stack_ptr: &'static u32, stack_size: usize, entry_point: *const u32, param: Option<*mut u32>) -> bool {
    let mut new_task = Task {
        sp: StackPtr { reference: stack_ptr, },
        dynamic_stack: Vec::new(),
        param: core::ptr::null_mut(),
        state: TaskState::Runnable,
    };

    //Arm uses a full descending stack so we have to start from the top
    new_task.sp.num += 4 * (stack_size as u32);
    let param_ptr = match param {
        Some(p) => p as *mut u32,
        None => 0 as *mut u32
    };

    new_task.sp.ptr = set_initial_stack(new_task.sp.ptr, entry_point, param_ptr);

    TASKS.push(new_task);

    true
}

//This function is called by the task spawn syscall.
//This function handles cleaning up after a task when
//it returns
pub fn new_task_helper(task_info: Box<NewTaskInfo>) -> ! {
    let task_param : &u32 = unsafe { &*task_info.param };
    let task_ep : fn(&u32) = unsafe { core::mem::transmute(task_info.ep) };

    unsafe {
        TASKS[TASK_INDEX].param = task_info.param;
    }

    task_ep(task_param);

    //Free task_info
    {
        let _pass_ownership = task_info;
    }

    fe_osi::exit();

    loop{}
}

pub unsafe fn remove_task() {
    TASKS[TASK_INDEX].state = TaskState::Zombie;
    do_context_switch();
}

pub fn start_scheduler(
    trigger_context_switch: fn(),
    mut systick: cortex_m::peripheral::SYST,
    reload_val: u32,
) {
    syscall::link_syscalls();

    unsafe {
        add_task_static(&KERNEL_STACK[0], DEFAULT_STACK_SIZE, kernel as *const u32, None);
        TRIGGER_CONTEXT_SWITCH = trigger_context_switch;
    }

    //Basically, wait for the scheduler to start
    systick.set_reload(reload_val);
    systick.clear_current();
    systick.enable_counter();
    systick.enable_interrupt();

    loop {}
}

fn kernel(_: &mut u32) {
    unsafe {
        loop {
            let mut task_num: usize = 0;

            while task_num < TASKS.len() {
                let task = &mut TASKS[task_num];

                match task.state {
                    TaskState::Runnable => {}
                    TaskState::Asleep(wake_up_ticks) => {
                        //If this task is asleep, see if we should wake it up
                        if wake_up_ticks < TICKS {
                            //If we wake the task up, we can run it
                            task.state = TaskState::Runnable;
                        }
                    }
                    TaskState::Blocking(sem) => {
                        let sem_ref: &Semaphore = &*sem;
                        //If the semaphore this task is blocking on is available
                        //to be taken, wake up the task so it can attempt to take it
                        if sem_ref.is_available() {
                            task.state = TaskState::Runnable;
                        }
                    }
                    TaskState::Zombie => {
                        //If this task has been removed, remove it from the list
                        //and rust will dealloc everything
                        if !task.param.is_null() {
                            Box::from_raw(task.param);
                        }
                        TASKS.swap_remove(task_num);
                        task_num -= 1;
                        continue;
                    }
                }

                task_num += 1;
            }

            //Going through the loop multiple times without anything else running is
            //useless since their states will not have changed, so do a context switch
            do_context_switch();
        }
    }
}

fn idle() {
    loop {}
}
