This is the crate that contains the setup code to allow Rust programs to run on a bare-metal ARM CPU. It includes a few important things:
* src/lib.sr
    * Contains the reset point of the application as well as the panic handler
* link.x
    * A script file for the linker to make sure everything is where it's supposed to be in memory
* build.rs
    * Rust program that helps configure the build environment

Important things to note when using this crate in your own projects:
* You must use the `#![no_std]` and `#![no_main]` attributes due to the limitations of running without an OS
* Your function must include a function called main that looks like this `pub fn main() -> !`
    * Note: This is not the same main function in standard Rust, but what the runtime calls after it's done
    * It basically functions as a normal main function, but you must make sure it does not return
    * You should use the macro `entry!(main);` at the before the declaration of main to make sure it's of the right type
* You may need download the rust information for your target before building
    * This is done by running `rustup target add <target_arch>`
    * For the tm4c, that would be `<target_arch>` would either be `thumbv7em-none-eabi` or `thumbv7em-none-eabihf`

To add this crate to you project, simply open the project's Cargo.toml file and add
`fe_rtos = { path = "path to crate"}` under `[dependencies]`

To build and load a program using this crate to a microcontroller, you'll need the following programs:
* OpenOCD
* [gcc-arm-none-eabi cross compiller](https://developer.arm.com/open-source/gnu-toolchain/gnu-rm/downloads) (for the linker and GDB)

To build the project, simply run `cargo build` in the project's root directory.

If you want to use the heap allocator with Rust's built in collections, you will need to use the nightly rust build. You can enable
this by running `rustup default nightly`. To revert back to stable rust, run `rustup default stable`.

To load the binary onto the board:
1. Run `openocd -f <path to config file>`
    * For me, the file was located at /usr/share/openocd/scripts/board/ek-tm4c123gxl.cfg
1. In a separate terminal, run `arm-none-eabi-gdb -q  <path to binary>`
1. In GDB, type in `target extended-remote :3333` to connect to the openocd server
1. Finally, type `load` and hopefully that will flash the binary onto the board.

This code started from the examples at https://docs.rust-embedded.org/embedonomicon/preface.html.
