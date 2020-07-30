extern crate alloc;

use crate::task::get_cur_task;
use alloc::format;
use core::panic::PanicInfo;
use core::ptr;
use fe_osi::print_msg;

/// Registers an interrupt to a specific handler.
/// This should only be used for bootstrapping purposes to setup the system.
/// Once FeRTOS is up and running `fe_osi::interrupt::register_interrupt` should
/// be used.
///
/// # Examples
/// ```
/// fe_rtos::interrupt::int_register(Exception::SysTick.irqn(), fe_rtos::task::sys_tick as *const usize);
/// fe_rtos::interrupt::int_register(Exception::PendSV.irqn(), fe_rtos::task::context_switch as *const usize);
/// fe_rtos::interrupt::int_register(Exception::SVCall.irqn(), fe_rtos::syscall::svc_handler as *const usize);
/// ```
#[cfg(target_arch = "arm")]
pub fn int_register(irqn: i8, int_handler: *const usize) {
    //This is defined in the linker script
    extern "C" {
        static mut _svtable: u32;
        static mut _evtable: u32;
    }

    unsafe {
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
}

#[cfg(not(target_arch = "arm"))]
pub fn int_register(_irqn: i8, _int_handler: *const usize) {}

/// The reset handler. This is the very first thing that runs when the system starts.
///
/// # Safety
/// This function should never be directly called.
#[no_mangle]
unsafe extern "C" fn Reset() -> ! {
    //Initialize the static variables in RAM
    extern "C" {
        static mut _sbss: u8;
        static mut _ebss: u8;

        static mut _sdata: u8;
        static mut _edata: u8;
        static _sidata: u8;
    }

    let count = &_ebss as *const u8 as usize - &_sbss as *const u8 as usize;
    ptr::write_bytes(&mut _sbss as *mut u8, 0, count);

    let count = &_edata as *const u8 as usize - &_sdata as *const u8 as usize;
    ptr::copy_nonoverlapping(&_sidata as *const u8, &mut _sdata as *mut u8, count);

    extern "Rust" {
        fn main() -> !;
    }

    main()
}

#[link_section = ".vector_table.reset_vector"]
#[no_mangle]
pub static RESET_VECTOR: unsafe extern "C" fn() -> ! = Reset;

#[panic_handler]
fn panic(panic_info: &PanicInfo<'_>) -> ! {
    let pid = unsafe { get_cur_task().pid };

    unsafe {
        super::disable_interrupts();
    }

    //Output which process has panicked
    print_msg(format!("Oh no! Process {} has panicked!\n", pid).as_str());

    //Output the file and line number if there is one
    if let Some(loc) = panic_info.location() {
        print_msg(format!("{} line {}\n", loc.file(), loc.line()).as_str());
    }

    //Output the panic message if we can be reasonably sure the heap can handle it
    if let Some(msg) = panic_info.message() {
        print_msg(format!("{}\n", msg).as_str());
    }
    unsafe {
        super::enable_interrupts();
    }

    fe_osi::exit();

    loop {}
}

#[no_mangle]
pub extern "C" fn DefaultExceptionHandler() {
    loop {}
}
