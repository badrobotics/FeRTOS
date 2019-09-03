#![no_std]
#![feature(alloc_error_handler)]
#![feature(const_vec_new)]

pub mod fe_alloc;
pub mod task;
pub mod syscall;

use core::panic::PanicInfo;
use core::ptr;
use fe_alloc::FeAllocator;

//Declare the heap allocator so we can use Rust's collections
#[global_allocator]
static mut A: FeAllocator = FeAllocator;

extern "C" {
    fn disable_interrupts();
    fn enable_interrupts();
}


