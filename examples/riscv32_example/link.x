/* 1K = 1 KiBi = 1024 bytes */
/* 1M = 1 MiBi = 1024 KiBi  */
OUTPUT_ARCH(riscv)
MEMORY
{
    RAM (wxa!r) : ORIGIN = 0x80000000, LENGTH = 128M
}

ENTRY(_start)

PHDRS
{
    text PT_LOAD;
    data PT_LOAD;
    bss PT_LOAD;
}

SECTIONS
{
    .text :
    {
        KEEP(*(.init));
        *(.text .text.*);
    } >RAM AT>RAM :text

    .rodata :
    {
        *(.rodata .rodata.*);
    } >RAM AT>RAM :text

    .bss :
    {
        _sbss = .;
        *(.sbss .sbss.* .bss .bss.*);
        _ebss = .;
    } >RAM AT>RAM :bss

    .data :
    {
        _sdata = .;
        *(.sdata .sdata.* .data .data.*);
        _edata = .;
    } >RAM AT>RAM :data

    _sidata = LOADADDR(.data);

    .heap :
    {
        _sheap = .;
        . += 512 * 1024;
        _eheap = .;
    } >RAM

    .eh_frame (INFO) : { KEEP(*(.eh_frame)) }
}

PROVIDE(__stack_top = ORIGIN(RAM) + LENGTH(RAM));
