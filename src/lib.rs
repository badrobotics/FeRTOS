#![no_std]

extern crate fe_osi;

mod fe_alloc;
pub mod task;
pub mod syscall;

use fe_osi::allocator::FeAllocator;

//Declare the heap allocator so we can use Rust's collections
#[global_allocator]
static mut A: FeAllocator = FeAllocator;

extern "C" {
    fn disable_interrupts();
    fn enable_interrupts();
}


