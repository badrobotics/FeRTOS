#!/bin/bash
cargo build
qemu-system-riscv32 -M virt -m 128M -serial stdio -gdb tcp::3333 -S -bios none -kernel ../../target/riscv32imac-unknown-none-elf/debug/riscv32_example
