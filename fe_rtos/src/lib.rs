#![no_std]
#![feature(const_mut_refs)]
#![feature(linked_list_remove)]
#![feature(const_btree_new)]

#[macro_use]
extern crate lazy_static;
extern crate fe_osi;

mod fe_alloc;
pub mod interrupt;
pub mod ipc;
pub mod syscall;
pub mod task;

use fe_alloc::KernelAllocator;

//Declare the heap allocator so we can use Rust's collections
#[global_allocator]
static mut A: KernelAllocator = KernelAllocator;

extern "C" {
    fn disable_interrupts();
    fn enable_interrupts();
}
