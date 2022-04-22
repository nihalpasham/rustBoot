/* SPDX-License-Identifier: MIT OR Apache-2.0
 *
 * Copyright (c) 2018-2021 Andre Richter <andre.o.richter@gmail.com>
 */

PAGE_SIZE = 4K;
PAGE_MASK = PAGE_SIZE - 1;

__rpi_phys_dram_start_addr = 0;

/* The physical address at which the the kernel binary will be loaded by the Raspberry's firmware */
__rpi_phys_binary_load_addr = 0x80000;

ENTRY(__rpi_phys_binary_load_addr)

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
    segment_dtb_load        PT_LOAD FLAGS(6);
    segment_kernel_load     PT_LOAD FLAGS(6);
    segment_initramfs_load  PT_LOAD FLAGS(6);
}

SECTIONS
{
    . =  __rpi_phys_dram_start_addr;

    /***********************************************************************************************
    * Boot Core Stack
    ***********************************************************************************************/
    .boot_core_stack (NOLOAD) :
    {
                                             /*   ^             */
                                             /*   | stack       */
        . += __rpi_phys_binary_load_addr;    /*   | growth      */
                                             /*   | direction   */
        __boot_core_stack_end_exclusive = .; /*   |             */
    } :segment_boot_core_stack

    ASSERT((. & PAGE_MASK) == 0, "End of boot core stack is not page aligned")

    /***********************************************************************************************
    * Code + RO Data + Global Offset Table
    ***********************************************************************************************/
    __code_start = .;
    .text :
    {
        KEEP(*(.text._start))      
        *(.text._start_arguments) /* Constants (or statics in Rust speak) read by _start(). */
        *(.text._start_rust)      /* The Rust entry point */
        *(.text*)                 /* Everything else, add all other input .text sections (i.e. from other 
                                  input object files) to our main binary. */
    } :segment_code

    .rodata : ALIGN(8) { *(.rodata*) } :segment_code
    .got    : ALIGN(8) { *(.got)     } :segment_code

     . = ALIGN(PAGE_SIZE);
    __code_end_exclusive = .;

    /***********************************************************************************************
    * Data + BSS
    ***********************************************************************************************/
    .data : ALIGN(65536) { *(.data*) } :segment_data /* align .data to a 64KiB boundary. */
                                                     /* As we enable the MMU and set the paging granularity to 64KiB. */
                                                     /* So, the end of code section and start of data section needs to be 64KiB aligned.*/
    
    /* Section is zeroed in pairs of u64. Align start and end to 16 bytes */
    .bss (NOLOAD): ALIGN(16)
    {
        __bss_start = .;
        *(.bss*);
        . = ALIGN(16);
        __bss_end_exclusive = .;
    } :segment_data /* not a section that's loaded into memory, we just need to allocate as many zeroed bytes.*/
    
}