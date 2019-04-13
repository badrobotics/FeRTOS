extern crate alloc;
extern crate rust_tm4c123;

use alloc::vec::Vec;
use rust_tm4c123::interrupt;

#[repr(C)]
#[derive(Copy, Clone)]
union StackPtr {
    reference: &'static u32,
    ptr: *const u32,
    num: u32
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Task {
    //Stack pointer
    sp: StackPtr,
    //Entry point into task
    ep: fn()
}

const INIT_XPSR: u32 = 0x01000000;
const IDLE_STACK_SIZE: usize = 64;

static mut IDLE_STACK: [u32; IDLE_STACK_SIZE] = [0; IDLE_STACK_SIZE];
static mut TASKS: Vec<Task> = Vec::new();
static mut TASK_INDX: usize = 0;
static mut IDLE_TASK: Task = Task { sp: StackPtr { num: 0 }, ep: idle };
static mut NEXT_TASK: &Task = unsafe { &IDLE_TASK };
static mut CUR_TASK: &Task = unsafe { &IDLE_TASK };

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

pub unsafe extern "C" fn sys_tick() {
    NEXT_TASK = if TASK_INDX + 1 <  TASKS.len() {
        TASK_INDX += 1;
        &TASKS[TASK_INDX]
    } else {
        TASK_INDX = 0;
        &TASKS[TASK_INDX]
    };

    interrupt::trigger_pendsv();
}

//See section 2.5.7.1 and Figure 2-7 in the TM4C123 datasheet for more information
unsafe fn set_initial_stack(stack_ptr: *const u32, entry_point: fn()) -> *const u32 {
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

    //Clear all of the registers
    for _i in 0..13 {
        *cur_ptr = 0;
        cur_ptr = (cur_ptr as u32 - 4) as *mut u32;
    }

    //In the for loop we subtracted 4 too many, so we have to add that back
    (cur_ptr as u32 + 4) as *const u32
}

pub unsafe fn add_task(stack_ptr: &'static u32, stack_size: usize, entry_point: fn()) -> bool {
    let mut new_task = Task {
        sp: StackPtr {
                reference: stack_ptr
            },
        ep: entry_point
    };

    //Arm uses a full descending stack so we have to start from the top
    new_task.sp.num += 4 * (stack_size as u32);

    new_task.sp.ptr = set_initial_stack(new_task.sp.ptr, entry_point);

    TASKS.push(new_task);

    true
}

pub fn start_scheduler() {

    unsafe {
        IDLE_TASK.sp.reference = &IDLE_STACK[0];
        IDLE_TASK.sp.num += 4 * (IDLE_STACK_SIZE as u32);
        IDLE_TASK.sp.ptr = set_initial_stack(IDLE_TASK.sp.ptr, idle);
    }

    //Basically, wait for the scheduler to start
    interrupt::enable_systick(1000000);

    loop {}
}

fn idle() {
    loop{}
}
