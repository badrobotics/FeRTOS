use crate::interrupt;
use crate::task;
use core::mem::{size_of, transmute};
use core::ptr::{null, null_mut};

const INT_MASK: usize = 0x80000000;
const TIMER_INT: usize = INT_MASK | 7;
static mut MTIME: *const u64 = null();
static mut MTIMECMP: *mut u64 = null_mut();
static mut INT_HANDLER: [extern "C" fn(); 16] = [interrupt::DefaultExceptionHandler; 16];
static mut EXC_HANDLER: [extern "C" fn(); 16] = [interrupt::DefaultExceptionHandler; 16];
static mut RELOAD_VAL: u64 = 0;

extern "C" {
    pub(crate) fn disable_interrupts();
    pub(crate) fn enable_interrupts();
    pub(crate) fn trigger_context_switch();
    fn set_mtie();
    fn get_mepc() -> usize;
    fn set_mepc(mepc: usize);
    fn setup_interrupts();
}

/// Sets up everything the arm needs before starting FeRTOS.
/// This should be called before doing anything FeRTOS related.
///
/// # Safety
/// mtime and mtimecmp must actually point to those registers on the processor
pub unsafe fn arch_setup(mtime: *const u64, mtimecmp: *mut u64) {
    MTIME = mtime;
    MTIMECMP = mtimecmp;

    int_register(TIMER_INT as isize, task::sys_tick as *const usize);
    setup_interrupts();
}

pub(crate) unsafe fn set_canary(stack_bottom: *const usize, _stack_size: usize) -> *const usize {
    //In riscv, the canary will be the bottom of the stack
    let canary = stack_bottom as *mut usize;
    *canary = task::STACK_CANARY;
    canary
}

pub(crate) unsafe fn set_initial_stack(
    stack_bottom: *const usize,
    stack_size: usize,
    entry_point: *const usize,
    param: *mut usize,
) -> *const usize {
    let stack_top = stack_bottom.add(stack_size / size_of::<usize>()) as *mut usize;

    //Initially set all registers to 0
    for i in 1..32 {
        *stack_top.offset(-i) = 0;
    }

    //Set the return address
    *stack_top.offset(-1) = task::idle as usize;

    //Set the correct stack pointer (x2) value
    *stack_top.offset(-2) = stack_top.offset(-32) as usize;

    //Set the parameter
    *stack_top.offset(-10) = param as usize;

    //Store the entry point at the bottom
    //This is also where we will store the mepc CSR to help with
    //context switching
    *stack_top.offset(-32) = entry_point as usize;

    stack_top.offset(-32)
}

pub(crate) unsafe fn int_register(irqn_isize: isize, int_handler: *const usize) {
    let irqn = irqn_isize as usize;
    let cause = irqn & !INT_MASK;
    // Man, this better be a function pointer
    let handler = transmute::<*const usize, extern "C" fn()>(int_handler);

    if irqn & INT_MASK == INT_MASK {
        INT_HANDLER[cause] = handler;
    } else {
        EXC_HANDLER[cause] = handler;
    }
}

pub fn enable_systick(reload_val: usize) {
    unsafe {
        RELOAD_VAL = reload_val as u64;
        *MTIMECMP = *MTIME + RELOAD_VAL;
        set_mtie();
    }
}

#[no_mangle]
unsafe fn interrupt_switch(mcause: usize) {
    let cause = mcause & !INT_MASK;
    let mut mepc: usize = 0;

    if mcause & INT_MASK == INT_MASK {
        //Becuase the timer handler calls ecall to force a context
        //switch, we need to save mepc to return to the right state
        if mcause == TIMER_INT {
            *MTIMECMP += RELOAD_VAL;
            mepc = get_mepc();
        }
        INT_HANDLER[cause]();
        if mcause == TIMER_INT {
            set_mepc(mepc);
        }
    } else {
        EXC_HANDLER[cause]();
    }
}
