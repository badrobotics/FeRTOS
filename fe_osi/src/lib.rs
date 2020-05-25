#![no_std]
#![feature(alloc_error_handler)]
#![feature(vec_into_raw_parts)]

pub mod allocator;
pub mod interrupt;
pub mod ipc;
pub mod semaphore;
pub mod task;

extern "C" {
    fn do_exit() -> usize;
    fn do_sleep(seconds: u32) -> usize;
    fn do_yield() -> usize;
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
