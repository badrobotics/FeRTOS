Current build status: ![Build Status](https://github.com/badrobotics/FeRTOS/workflows/Rust/badge.svg)

FeRTOS is a simple "operating system" that currently supports ARM Cortex-M CPUs,
though support for other architectures is planned for the future.
It currently has the following features:
* Ability to run multiple "tasks" simultaneously.
* Support for dynamic memory allocation
* Semaphores to help avoid race conditions
* An IPC interface to allow different tasks to communicate
* Various system calls

This repository contains two different crates: fe_rtos and fe_osi.<br />
fe_rtos is the crate that contains the kernel, scheduler, and syscall implementations.<br />
fe_osi is the crate that is an interface to the syscalls in fe_rtos for the user program. <br />
Currently, you must load one monolithic binary with fe_rtos, fe_osi, and the user program
to the target device, but in the future, we plan on allowing users to load arbitrary
binaries which will make this distinction more significant.

Important things to note when using this crate in your own projects:
* You must use the `#![no_std]` and `#![no_main]` attributes due to the limitations of running without an OS
* Your function must include a function called main that looks like this `pub fn main() -> !`
    * This is not the same main function in standard Rust, but what the runtime calls after it's done
    * This basically functions as a normal main function, but you must make sure it does not return
    * This should have the `#[no_mangle]` attribute to work correctly
* You may need download the rust information for your target before building
    * This is done by running `rustup target add <target_arch>`
    * For the tm4c, that would be `<target_arch>` would either be `thumbv7em-none-eabi` or `thumbv7em-none-eabihf`
    * for the example qemu project `<target_arch>` is `thumbv7m-none-eabi`
* You will likely need to create a custom linker script for your project. This is needed to set up the interupt
table and to tell FeRTOS where and how big the heap is. Look at the example projects for more information.

To add this crate to you project, simply open the project's Cargo.toml file and add the following under `[dependencies]`
* `fe_rtos = { path = "path to fe_rtos" }`
* `fe_osi = { path = "path to fe_osi" }`

To build and load a FeRTOS program using this crate to a microcontroller, you'll need the following programs:
* OpenOCD
* gdb-multiarch
* [gcc-arm-none-eabi cross compiller](https://developer.arm.com/open-source/gnu-toolchain/gnu-rm/downloads) (for the linker and assembler) for cortex-m
* [riscv gnu toolchain](https://github.com/riscv/riscv-gnu-toolchain) for riscv32

FeRTOS depends on unstable rust features, so you must use the nightly rust compiler to build it.
You can enable this by running `rustup default nightly`. To revert back to stable rust, run `rustup default stable`.

## Possible FAQs
### Why should I use FeRTOS over something like FreeRTOS?
You probably shouldn't.

This code started from the examples at https://docs.rust-embedded.org/embedonomicon/preface.html.
