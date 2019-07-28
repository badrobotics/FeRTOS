/* Memory layout of the TM4C123 microcontroller */
/* 1K = 1 KiBi = 1024 bytes */
MEMORY
{
  FLASH : ORIGIN = 0x00000000, LENGTH = 256K
  RAM : ORIGIN = 0x20000000, LENGTH = 32K
}

/* The entry point is the reset handler */
ENTRY(Reset);

/* Needed to avoid linker from rudely throwing away these symbols*/
EXTERN(RESET_VECTOR);
EXTERN(EXCEPTIONS);
EXTERN(sys_exit);
EXTERN(sys_sleep);

SECTIONS
{
  .vector_table ORIGIN(FLASH) :
  {
    /* First entry: initial Stack Pointer value */
    LONG(ORIGIN(RAM) + LENGTH(RAM));

    /* Second entry: reset vector */
    KEEP(*(.vector_table.reset_vector));

    /* The next 14 entries are exception vectors */
    KEEP(*(.vector_table.exceptions));
  } > FLASH

  .text :
  {
    *(.text .text.*);
  } > FLASH

  .rodata :
  {
    *(.rodata .rodata.*);
  } > FLASH

  /* Create a section in RAM for where the Vector Table will be relocated  */
  .ram_vtable ORIGIN(RAM) :
  {
    /* Make sure there is enough reserved space for the table */
    /* There are 155 interrupts in the TM4C123 at 4 bytes each */
    _svtable = .;
    . += 4 * 155;
  } > RAM

  .bss :
  {
    _sbss = .;
    *(.bss .bss.*);
    _ebss = .;
  } > RAM

  .data : AT(ADDR(.rodata) + SIZEOF(.rodata))
  {
    _sdata = .;
    *(.data .data.*);
    _edata = .;
  } > RAM

  _sidata = LOADADDR(.data);

  /* Make sure the heap is 8 byte aligned */
  .heap ALIGN(0x8):
  {
    _sheap = .;
    /* Make the heap 10k in size */
    . += 10 * 1024;
    _eheap = .;
  } > RAM

  /DISCARD/ :
  {
    *(.ARM.exidx.*);
  }
}

/* Give the handlers default values */
PROVIDE(NMI = DefaultExceptionHandler);
PROVIDE(HardFault = DefaultExceptionHandler);
PROVIDE(MemManage = DefaultExceptionHandler);
PROVIDE(BusFault = DefaultExceptionHandler);
PROVIDE(UsageFault = DefaultExceptionHandler);
PROVIDE(SVCall = DefaultExceptionHandler);
PROVIDE(PendSV = DefaultExceptionHandler);
PROVIDE(SysTick = DefaultExceptionHandler);
