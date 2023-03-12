
/*
 * Branch if current processor is a Cortex-A53 core.
 */
.macro	branch_if_a53_core, xreg, a53_label
	mrs	\xreg, midr_el1
	lsr	\xreg, \xreg, #4
	and	\xreg, \xreg, #0x00000FFF
	cmp	\xreg, #0xD03		/* Cortex-A53 MPCore processor. */
	b.eq	\a53_label
.endm

.equ _EL2, 0x8

.section .text._start
.global _reset
_reset:
    /*
	 * when ROM code hands over control - system could be in EL3/EL2/EL1, 
     * But generally initial state for rBoot at this point is:
	 * little endian, MMU disabled, i/d cache disabled
	 */
    mrs	x1, currentel
    cmp	x1, _EL2
    b.gt	3f
    b.eq	2f  
    b.lt	1f
3:  
    mrs	x0, scr_el3
    orr	x0, x0, #0xf			        /* SCR_EL3.NS|IRQ|FIQ|EA */
    msr	scr_el3, x0
    msr	cptr_el3, xzr			        /* Enable FP/SIMD */
    b	0f
2:  	
    mrs	x1, hcr_el2
    tbnz x1, #34, 1f                	/* HCR_EL2.E2H */
    orr	x1, x1, #32             	    /* Route SErrors to EL2 */
    msr	hcr_el2, x1
    mov	x0, #0x33ff
    msr	cptr_el2, x0			        /* Enable FP/SIMD */
    b	0f
1:  	
    mov	x0, #3 << 20
    msr	cpacr_el1, x0			        /* Enable FP/SIMD */
0:
    msr	daifclr, #0x4	                /* Enable asynchronous aborts (i.e. exception) only */
    
    isb

    /* Apply ARM core specific erratas */

	bl	apply_core_errata

    // initialize bss to all zeroes
.L__do_bss:
    ldr     x9, =_zero_region_start
    ldr     x10, =_zero_region_end
    sub     x10, x10, x9
    cbz     x10, .L__bss_loop_done
.L__bss_loop:
    sub     x10, x10, #8
    str     xzr, [x9], #8
    cbnz    x10, .L__bss_loop
.L__bss_loop_done:

    msr	SPSel, #1		                /* make sure we use SP_ELx */
    // set the stack pointer
    ldr x0, =_stack_end
    mov sp, x0

    mov x0, #0
    // jump to rust code.
    bl _start_rust
    b .L_parking_loop

/*-----------------------------------------------------------------------*/

apply_core_errata:
	mov	x29, lr			/* Save LR */
	/* For now, we support Cortex-A53 errata */

	/* Check if we are running on a Cortex-A53 core */
	branch_if_a53_core x0, apply_a53_core_errata
0:
	mov	lr, x29			/* Restore LR */
	ret

apply_a53_core_errata:

#ifdef CONFIG_ARM_ERRATA_855873
	mrs	x0, midr_el1
	tst	x0, #(0xf << 20)
	b.ne	0b

	mrs	x0, midr_el1
	and	x0, x0, #0xf
	cmp	x0, #3
	b.lt	0b

	mrs	x0, S3_1_c15_c2_0	/* cpuactlr_el1 */
	/* Enable data cache clean as data cache clean/invalidate */
	orr	x0, x0, #1 << 44
	msr	S3_1_c15_c2_0, x0	/* cpuactlr_el1 */
	isb
#endif
	b 0b

// Infinitely wait for events (aka "park the core").
.L_parking_loop:
	wfe
	b	.L_parking_loop

__zero_region_start:
    .word _zero_region_start
__zero_region_end:
    .word _zero_region_end
