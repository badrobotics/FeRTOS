extern crate alloc;

use crate::arch;
use crate::task::get_cur_task;
use alloc::format;
use core::panic::PanicInfo;
use core::ptr;
use fe_osi::print_msg;

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
        arch::disable_interrupts();
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
        arch::enable_interrupts();
        fe_osi::exit();
    }

    loop {}
}

#[no_mangle]
pub extern "C" fn DefaultExceptionHandler() {
    loop {}
}
