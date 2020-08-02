/* Memory layout of the lm3s6965 microcontroller */
/* 1K = 1 KiBi = 1024 bytes */
/* 1M = 1 MiBi = 1024 KiBi  */
MEMORY
{
  FLASH : ORIGIN = 0x00000000, LENGTH = 256K
  RAM : ORIGIN = 0x20000000, LENGTH = 64K
}

/* The entry point is the reset handler */
ENTRY(Reset);

EXTERN(RESET_VECTOR);
EXTERN(EXCEPTIONS);

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

    /* The next entries are the interrupt vectors */
    KEEP(*(vector_table.interrupts));
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
    /* There are 60 interrupts in the lm3s6965 at 4 bytes each */
    _svtable = .;
    . += 4 * 60;
    _evtable = .;
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

  .heap :
  {
    _sheap = .;
    /* Determines the heap size */
    . += 50 * 1024;
    _eheap = .;
  } > RAM

  /DISCARD/ :
  {
    *(.ARM.exidx.*);
    *(.ARM.exidx);
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

/* Define the base addresses of all the peripherals */
SYSTICK        = 0xE000E000;
NVIC           = 0xE000E000;
SCB            = 0xE000E000;
MPU            = 0xE000E000;
FPU            = 0xE000E000;
SYSTEM_CONTROL = 0x400FE000;
