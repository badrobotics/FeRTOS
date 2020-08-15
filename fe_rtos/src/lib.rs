#![no_std]
#![feature(const_btree_new)]
#![feature(const_mut_refs)]
#![feature(linked_list_remove)]
#![feature(panic_info_message)]

#[macro_use]
extern crate lazy_static;
extern crate fe_osi;

#[cfg(target_arch = "arm")]
#[path = "arch/arm/mod.rs"]
pub mod arch;
#[cfg(not(target_arch = "arm"))]
#[path = "arch/none/mod.rs"]
pub mod arch;
mod fe_alloc;
pub mod interrupt;
pub mod ipc;
mod spinlock;
pub mod syscall;
pub mod task;

use fe_alloc::KernelAllocator;

//Declare the heap allocator so we can use Rust's collections
#[global_allocator]
static mut A: KernelAllocator = KernelAllocator;
