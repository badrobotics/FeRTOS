#![no_std]
#![feature(alloc_error_handler)]
#![feature(vec_into_raw_parts)]

pub mod allocator;
pub mod interrupt;
pub mod ipc;
pub mod semaphore;
pub mod task;

static mut PUTC: Option<fn(char)> = None;

extern "C" {
    fn do_exit() -> usize;
    fn do_sleep(seconds: u32) -> usize;
    fn do_yield() -> usize;
}

/// Sets what is internally used for putc to print out error messages
pub fn set_putc(putc: fn(char)) {
    unsafe {
        PUTC = Some(putc);
    }
}

/// If a putc function has been set using set_putc, print a character
/// using the putc function.
pub fn putc(c: char) {
    unsafe {
        if let Some(callback) = PUTC {
            callback(c);
        }
    }
}

/// If a putc function has been set using set_putc, print a string
/// using the putc function
pub fn print_msg(msg: &str) {
    for c in msg.chars() {
        putc(c);
    }
}

/// Causes the calling task to exit.
pub fn exit() -> usize {
    unsafe {
        do_exit();
    }
    //Execution should never reach here
    loop {}
}

/// Causes the calling task to block for at least the specified number of
/// milliseconds.
pub fn sleep(ms: u32) -> usize {
    unsafe {
        do_sleep(ms);
    }

    0
}

/// Triggers a context switch.
pub fn r#yield() -> usize {
    unsafe {
        do_yield();
    }

    0
}
