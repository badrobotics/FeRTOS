use core::ptr;
use core::panic::PanicInfo;

pub fn int_register(int_num: u32, int_handler: unsafe extern "C" fn()) {
    //This is defined in the linker script
    extern "C" {
        static mut _svtable: u32;
        static mut _evtable: u32;
    }

    unsafe {
        let ram_table_start = &_svtable as *const u32;
        let ram_table_end = &_evtable as *const u32;
        let num_interrupts = ((ram_table_end as u32) - (ram_table_start as u32)) / 4;
        let int_pos = (ram_table_start as u32 + (4 * int_num)) as *mut u32;
        let regs = cortex_m::peripheral::SCB::ptr();

        //If the interrupt table has not been relocated to RAM, do so
        if (*regs).vtor.read() != ram_table_start as u32 {
            //I tried to use ptr::copy_nonoverlapping, but for some reason
            //that didn't work...
            for i in 0..num_interrupts {
                let ram_pos = (ram_table_start as u32 + (4 * i as u32)) as *mut u32;
                let flash_pos = ((*regs).vtor.read() + (4 * i as u32)) as *mut u32;
                ptr::write(ram_pos, *flash_pos);
            }
            (*regs).vtor.write(ram_table_start as u32);
        }
        ptr::write(int_pos, int_handler as u32);
    }
}

// The reset handler
#[no_mangle]
pub unsafe extern "C" fn Reset() -> ! {
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
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn DefaultExceptionHandler() {
    loop {}
}
