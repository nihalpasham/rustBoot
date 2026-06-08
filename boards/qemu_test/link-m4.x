MEMORY
{
  FLASH (rx)  : ORIGIN = 0x08000000, LENGTH = 1M
  RAM (rwx)   : ORIGIN = 0x20000000, LENGTH = 128K
}

SECTIONS
{
  .text ORIGIN(FLASH) : {
    KEEP(*(.vector_table))
    *(.text .text.* .rodata .rodata.*)
  } > FLASH

  /DISCARD/ : { *(.ARM.exidx*) *(.ARM.extab*) }
}