
PAGE_SIZE = 4K;
RB_STACK_SIZE_KB = 8;
RB_ENTRY = 0x912000;

OUTPUT_FORMAT("elf64-littleaarch64", "elf64-littleaarch64", "elf64-littleaarch64")
OUTPUT_ARCH(aarch64)
ENTRY(RB_ENTRY)

/* Flags:
 *     4 == R
 *     5 == RX
 *     6 == RW
 *
 * Segments are marked PT_LOAD below so that the ELF file provides virtual and physical addresses.
 * It doesn't mean all of them need actually be loaded.
 */
PHDRS
{
    segment_boot_core_stack PT_LOAD FLAGS(6);
    segment_code            PT_LOAD FLAGS(5);
    segment_data            PT_LOAD FLAGS(6);
}

 MEMORY
{
    ocram (rw)  : ORIGIN = 0x912000, LENGTH = 256k
}

SECTIONS
{
    /***********************************************************************************************
    * Code + RO Data + Global Offset Table
    ***********************************************************************************************/
    .text :
    {
        . = ALIGN(PAGE_SIZE);
        KEEP(*(.text._start))     /* Constants (or statics in Rust speak) read by _start(). */
        *(.text._start_rust)      /* The Rust entry point */
        *(.text*)                 /* Everything else, add all other input .text sections (i.e. from other 
                                  input object files) to our main binary. */
        
        . = ALIGN(PAGE_SIZE);
    } > ocram

    .rodata : ALIGN(PAGE_SIZE) 
    { 
        *(.rodata*) 
        . = ALIGN(PAGE_SIZE);
    } > ocram

    .got    : ALIGN(PAGE_SIZE) 
    { 
        *(.got)     
        . = ALIGN(PAGE_SIZE);
    } > ocram

     . = ALIGN(PAGE_SIZE);
    __code_end_exclusive = .;

    /***********************************************************************************************
    * Data + BSS
    ***********************************************************************************************/
    .data : ALIGN(PAGE_SIZE) 
    { 
        *(.data*)
        . = ALIGN(PAGE_SIZE); 
    } > ocram                                  /* align .data to a 4KB boundary. */
    
    /* Section is zeroed in pairs of u64. Align start and end to 4kB boundary */
    .bss (NOLOAD): ALIGN(PAGE_SIZE)
    {
        _zero_region_start = .;
        *(.bss)
        *(.bss .bss.*)
        . = ALIGN(PAGE_SIZE);
        _zero_region_end = .;
    } > ocram /* not a section that's loaded into memory, we just need to allocate as many zeroed bytes.*/

    /***********************************************************************************************
    * Boot Core Stack
    ***********************************************************************************************/
    .boot_core_stack (NOLOAD) : ALIGN(PAGE_SIZE)
    {
                                             /*   ^             */
                                             /*   | stack       */
                                             /*   | growth      */
        . = . + RB_STACK_SIZE_KB*1024 ;      /*   | direction   */
                                             /*   |             */
        . = ALIGN(PAGE_SIZE);
        _stack_end = .;

    } > ocram
    
}