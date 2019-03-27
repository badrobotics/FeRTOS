extern crate rust_tm4c123;

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
struct Task {
    //Stack pointer
    sp: StackPtr,
    //Entry point into task
    ep: fn()
}

const MAX_TASKS: usize = 10;
const INIT_XPSR: u32 = 0x01000000;
const IDLE_STACK_SIZE: usize = 64;

static mut IDLE_STACK: [u32; IDLE_STACK_SIZE] = [0; IDLE_STACK_SIZE];
static mut TASKS: [Task; MAX_TASKS] = [Task { sp: StackPtr {num: 0}, ep: idle }; MAX_TASKS];
static mut NUM_TASKS: usize = 1;
static mut NEXT_TASK: usize = 0;
static mut CUR_TASK: usize = 0;

extern "C" {
    pub fn context_switch();
}

#[no_mangle]
pub unsafe extern "C" fn get_cur_task() -> usize {
    CUR_TASK
}

#[no_mangle]
pub unsafe extern "C" fn set_cur_task(new_val: usize) {
    CUR_TASK = new_val;
}

#[no_mangle]
pub unsafe extern "C" fn get_next_task() -> usize {
    NEXT_TASK
}

#[no_mangle]
pub unsafe extern "C" fn get_task_table() -> *const u32 {
    &TASKS[0] as *const Task as *const u32
}

#[no_mangle]
pub unsafe extern "C" fn get_task_size() -> usize {
    core::mem::size_of::<Task>()
}

pub unsafe extern "C" fn sys_tick() {
    NEXT_TASK = if CUR_TASK + 1 < NUM_TASKS {
        CUR_TASK + 1
    } else {
        //Reset to 1 because TASK[0] is for the initial thread of execution
        1
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
    if NUM_TASKS < MAX_TASKS {
        TASKS[NUM_TASKS] = Task {
            sp: StackPtr {
                    reference: stack_ptr
                },
            ep: entry_point
        };

        //Arm uses a full descending stack so we have to start from the top
        TASKS[NUM_TASKS].sp.num += 4 * (stack_size as u32);

        TASKS[NUM_TASKS].sp.ptr = set_initial_stack(TASKS[NUM_TASKS].sp.ptr, entry_point);

        NUM_TASKS += 1;

        true
    } else {
        false
    }
}

pub fn start_scheduler() {

    unsafe {
        if NUM_TASKS == 0 {
            add_task(&IDLE_STACK[0], IDLE_STACK_SIZE, idle);
        }
    }

    //Basically, wait for the scheduler to start
    interrupt::enable_systick(1000000);

    loop {}
}

fn idle() {
    loop{}
}
