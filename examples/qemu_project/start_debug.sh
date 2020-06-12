#!/bin/bash
cargo build
qemu-system-arm -M lm3s6965evb -cpu cortex-m3 -serial stdio -gdb tcp::3333 -S -kernel ../../target/thumbv7m-none-eabi/debug/qemu_project
