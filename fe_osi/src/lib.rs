#![no_std]
#![feature(alloc_error_handler)]

pub mod allocator;
pub mod semaphore;
pub mod task;

extern "C" {
    fn do_exit() -> usize;
    fn do_sleep(seconds: u32) -> usize;
    fn do_yield() -> usize;
}

pub fn exit() -> usize {
    unsafe {
        while do_exit() != 0 {}
    }
    //Execution should never reach here
    loop {}
}

pub fn sleep(ms: u32) -> usize {
    unsafe {
        while do_sleep(ms) != 0 {}
    }

    0
}
pub fn r#yield() -> usize {
    unsafe {
        do_yield();
    }

    0
}
