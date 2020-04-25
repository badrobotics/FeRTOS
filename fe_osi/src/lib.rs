#![no_std]
#![feature(alloc_error_handler)]
#![feature(vec_into_raw_parts)]

pub mod allocator;
pub mod ipc;
pub mod semaphore;
pub mod task;

extern "C" {
    fn do_exit() -> usize;
    fn do_sleep(seconds: u32) -> usize;
    fn do_yield() -> usize;
}

pub fn exit() -> usize {
    unsafe {
        do_exit();
    }
    //Execution should never reach here
    loop {}
}

pub fn sleep(ms: u32) -> usize {
    unsafe {
        do_sleep(ms);
    }

    0
}
pub fn r#yield() -> usize {
    unsafe {
        do_yield();
    }

    0
}