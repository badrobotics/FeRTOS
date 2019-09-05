extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

#[repr(C)]
union StackPtr {
    reference: &'static u32,
    ptr: *const u32,
    num: u32
}

enum TaskState {
    Runnable,
    Asleep,
    Zombie,
}

#[repr(C)]
pub struct Task {
    //Stack pointer
    sp: StackPtr,
    //Entry point into task
    ep: fn(*const u32),
    //Holds the smart pointer to a dynamically allocated stack is applicable
    //It needs to be a smart type to easily allow for deallocation of the stack
    //if a task is deleted
    dynamic_stack: Vec<u32>,
    state: TaskState,
    //When to wake up the task
    wake_up_tick: u64,
}

const INIT_XPSR: u32 = 0x01000000;
pub const DEFAULT_STACK_SIZE: usize = 512;

static mut KERNEL_STACK: [u32; DEFAULT_STACK_SIZE] = [0; DEFAULT_STACK_SIZE];
static mut TASKS: Vec<Task> = Vec::new();
static mut TICKS: u64 = 0;
static mut TASK_INDEX: usize = 0;
static mut KERNEL_TASK: Task = Task {
                                sp: StackPtr { num: 0 },
                                ep: kernel,
                                dynamic_stack: Vec::new(),
                                state: TaskState::Runnable,
                                wake_up_tick: 0
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
            },
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
        TASKS[TASK_INDEX].wake_up_tick = TICKS + sleep_ticks;
        TASKS[TASK_INDEX].state = TaskState::Asleep;
        //Trigger a context switch and wait until that happens
        do_context_switch();
    }
}

//See section 2.5.7.1 and Figure 2-7 in the TM4C123 datasheet for more information
unsafe fn set_initial_stack(stack_ptr: *const u32, entry_point: fn(*const u32), param: Option<*const u32>) -> *const u32 {
    let mut cur_ptr = ((stack_ptr as u32) - 4) as *mut u32;
    let param_val : u32 = match param {
        Some(val) => val as u32,
        None => 0
    };

    //Set the xPSR
    *cur_ptr = INIT_XPSR;
    cur_ptr = (cur_ptr as u32 - 4) as *mut u32;

    //Set the PC
    *cur_ptr = entry_point as u32;
    cur_ptr = (cur_ptr as u32 - 4) as *mut u32;

    //Set the LR
    *cur_ptr = idle as u32;
    cur_ptr = (cur_ptr as u32 - 4) as *mut u32;

    //Set all of the registers
    //Since we need R0 to be the parameter, and the other registers will be
    //cleared on use, we'll set them all to the parameter
    for _i in 0..13 {
        *cur_ptr = param_val;
        cur_ptr = (cur_ptr as u32 - 4) as *mut u32;
    }

    //In the for loop we subtracted 4 too many, so we have to add that back
    (cur_ptr as u32 + 4) as *const u32
}

pub unsafe fn add_task(stack_size: usize, entry_point: fn(*const u32), param: Option<*const u32>) -> bool {
    let mut new_task = Task {
        sp: StackPtr {
                num: 0
            },
        ep: entry_point,
        dynamic_stack: vec![0; stack_size],
        state: TaskState::Runnable,
        wake_up_tick: 0,
    };

    //Convert the adress of the first element of the vector into a ptr for the stack
    new_task.sp.ptr = &new_task.dynamic_stack[0] as *const u32;
    //Arm uses a full descending stack so we have to start from the top
    new_task.sp.num += 4 * (stack_size as u32);

    new_task.sp.ptr = set_initial_stack(new_task.sp.ptr, entry_point, param);

    TASKS.push(new_task);

    true
}

pub unsafe fn add_task_static(stack_ptr: &'static u32, stack_size: usize, entry_point: fn(*const u32), param: Option<*const u32>) -> bool {
    let mut new_task = Task {
        sp: StackPtr {
                reference: stack_ptr
            },
        ep: entry_point,
        dynamic_stack: Vec::new(),
        state: TaskState::Runnable,
        wake_up_tick: 0,
    };

    //Arm uses a full descending stack so we have to start from the top
    new_task.sp.num += 4 * (stack_size as u32);

    new_task.sp.ptr = set_initial_stack(new_task.sp.ptr, entry_point, param);

    TASKS.push(new_task);

    true
}

pub unsafe fn remove_task() {
    TASKS[TASK_INDEX].state = TaskState::Zombie;
    do_context_switch();
}

pub fn start_scheduler(trigger_context_switch: fn())  {

    unsafe {
        add_task_static(&KERNEL_STACK[0], DEFAULT_STACK_SIZE, kernel, None);
        TRIGGER_CONTEXT_SWITCH = trigger_context_switch;
    }
}

fn kernel(_ : *const u32) {
    unsafe {
        loop {
            let mut task_num: usize = 0;

            while task_num < TASKS.len() {
                let task = &mut TASKS[task_num];

                match task.state {
                    TaskState::Runnable => {},
                    TaskState::Asleep => {
                        //If this task is asleep, see if we should wake it up
                        if task.wake_up_tick < TICKS {
                            //If we wake the task up, we can run it
                            task.state = TaskState::Runnable;
                        }
                    },
                    TaskState::Zombie => {
                        //If this task has been removed, remove it from the list
                        //and rust will dealloc everything
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
    loop{}
}
