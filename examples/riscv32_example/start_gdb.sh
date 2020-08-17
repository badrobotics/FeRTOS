#!/bin/bash
gdb-multiarch -x qemu.gdb -q -tui ../../target/riscv32imac-unknown-none-elf/debug/riscv32_example
