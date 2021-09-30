#![no_std]
#![feature(const_btree_new)]
#![feature(linked_list_remove)]
#![feature(panic_info_message)]
#![allow(clippy::empty_loop)]

#[macro_use]
extern crate lazy_static;
extern crate fe_osi;

#[cfg(target_arch = "arm")]
#[path = "arch/arm/mod.rs"]
pub mod arch;
#[cfg(target_arch = "riscv32")]
#[path = "arch/riscv32/mod.rs"]
pub mod arch;
#[cfg(not(any(target_arch = "arm", target_arch = "riscv32")))]
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
