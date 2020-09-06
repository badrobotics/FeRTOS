use crate::task;
use core::mem::size_of;
use core::ptr;
use cortex_m::peripheral::scb::Exception;

extern "C" {
    fn context_switch();
    fn svc_handler();
    pub(crate) fn disable_interrupts();
    pub(crate) fn enable_interrupts();
}

/// Sets up everything the arm needs before starting FeRTOS.
/// This should be called before doing anything FeRTOS related.
pub fn arch_setup(p: &mut cortex_m::peripheral::Peripherals) {
    unsafe {
        int_register(
            Exception::SysTick.irqn() as isize,
            task::sys_tick as *const usize,
        );
        int_register(
            Exception::PendSV.irqn() as isize,
            context_switch as *const usize,
        );
        int_register(
            Exception::SVCall.irqn() as isize,
            svc_handler as *const usize,
        );
    }

    //It's probably a good idea to have the context switch be the lowest
    //priority interrupt.
    unsafe {
        p.SCB
            .set_priority(cortex_m::peripheral::scb::SystemHandler::PendSV, 7);
    }
}

pub(crate) unsafe fn trigger_context_switch() {
    cortex_m::peripheral::SCB::set_pendsv();
}

pub(crate) unsafe fn set_canary(stack_bottom: *const usize, _stack_size: usize) -> *const usize {
    //In arm, the canary will be the bottom of the stack
    let canary = stack_bottom as *mut usize;
    *canary = task::STACK_CANARY;
    canary
}

//See section 2.5.7.1 and Figure 2-7 in the TM4C123 datasheet for more information
pub(crate) unsafe fn set_initial_stack(
    stack_bottom: *const usize,
    stack_size: usize,
    entry_point: *const usize,
    param: *mut usize,
) -> *const usize {
    const INIT_XPSR: usize = 0x01000000;

    //Arm uses a full descending stack, so we have to start from the top
    let stack_top = (stack_bottom as usize) + stack_size;
    let mut cur_ptr = (stack_top - size_of::<usize>()) as *mut usize;

    //Set the xPSR
    *cur_ptr = INIT_XPSR;
    cur_ptr = (cur_ptr as usize - size_of::<usize>()) as *mut usize;

    //Set the PC
    *cur_ptr = entry_point as usize;
    cur_ptr = (cur_ptr as usize - size_of::<usize>()) as *mut usize;

    //Set the LR
    *cur_ptr = task::idle as usize;
    cur_ptr = (cur_ptr as usize - size_of::<usize>()) as *mut usize;

    for _i in 0..13 {
        *cur_ptr = param as usize;
        cur_ptr = (cur_ptr as usize - size_of::<usize>()) as *mut usize;
    }
    (cur_ptr as usize + size_of::<usize>()) as *const usize
}

/// Registers an interrupt to a specific handler.
/// This should only be used for bootstrapping purposes to setup the system.
/// Once FeRTOS is up and running `fe_osi::interrupt::register_interrupt` should
/// be used.
///
/// # Examples
/// ```
/// fe_rtos::arch::int_register(Exception::SysTick.irqn(), fe_rtos::task::sys_tick as *const usize);
/// fe_rtos::arch::int_register(Exception::PendSV.irqn(), fe_rtos::task::context_switch as *const usize);
/// fe_rtos::arch::int_register(Exception::SVCall.irqn(), fe_rtos::syscall::svc_handler as *const usize);
/// ```
///
/// # Safety
/// int_handler must point to a function
pub(crate) unsafe fn int_register(irqn_isize: isize, int_handler: *const usize) {
    let irqn = irqn_isize as i8;
    //This is defined in the linker script
    extern "C" {
        static mut _svtable: u32;
        static mut _evtable: u32;
    }

    // ARM exceptions are negative numbers. Add 16 to be zero indexed.
    let int_num: usize = (irqn + 16) as usize;
    let int_pos = ((&_svtable) as *const u32).add(int_num) as *mut u32;

    //Collect pointer to SCB peripheral registers
    let regs = cortex_m::peripheral::SCB::ptr();

    //If the interrupt table has not been relocated to RAM, do so
    if (*regs).vtor.read() != &_svtable as *const u32 as u32 {
        // Number of interrupts in vector table
        let count = (&_evtable as *const u32 as usize - &_svtable as *const u32 as usize) / 4;

        // Copy vector table from flash to ram
        let flash_ptr = (*regs).vtor.read() as *const u32;
        let ram_ptr = (&mut _svtable) as *mut u32;
        ptr::copy_nonoverlapping(flash_ptr, ram_ptr, count);

        // point vector table to new locaiton
        (*regs).vtor.write(ram_ptr as u32);
    }

    // Register callback to irqn
    ptr::write(int_pos, int_handler as usize as u32);
}
