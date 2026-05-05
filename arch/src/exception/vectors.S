.section .text.pyr_el2_vectors, "ax"
.align 11
.global __pyr_el2_vector_table

__pyr_el2_vector_table:
    // Current EL with SP0
    b __pyr_el2_sync_current_sp0
    .space 0x80 - 4
    b __pyr_el2_irq_current_sp0
    .space 0x80 - 4
    b __pyr_el2_fiq_current_sp0
    .space 0x80 - 4
    b __pyr_el2_serror_current_sp0
    .space 0x80 - 4

    // Current EL with SPx
    b __pyr_el2_sync_current_spx
    .space 0x80 - 4
    b __pyr_el2_irq_current_spx
    .space 0x80 - 4
    b __pyr_el2_fiq_current_spx
    .space 0x80 - 4
    b __pyr_el2_serror_current_spx
    .space 0x80 - 4

    // Lower EL using AArch64
    b __pyr_el2_sync_lower_aarch64
    .space 0x80 - 4
    b __pyr_el2_irq_lower_aarch64
    .space 0x80 - 4
    b __pyr_el2_fiq_lower_aarch64
    .space 0x80 - 4
    b __pyr_el2_serror_lower_aarch64
    .space 0x80 - 4

    // Lower EL using AArch32
    b __pyr_el2_sync_lower_aarch32
    .space 0x80 - 4
    b __pyr_el2_irq_lower_aarch32
    .space 0x80 - 4
    b __pyr_el2_fiq_lower_aarch32
    .space 0x80 - 4
    b __pyr_el2_serror_lower_aarch32
    .space 0x80 - 4

__pyr_el2_sync_lower_aarch64:
    // Allocate TrapFrame: 0x50 bytes
    sub sp, sp, #0x50

    // Save x0..x7
    stp x0, x1, [sp, #0x00]
    stp x2, x3, [sp, #0x10]
    stp x4, x5, [sp, #0x20]
    stp x6, x7, [sp, #0x30]

    // Save ELR_EL2 and SPSR_EL2
    mrs x1, elr_el2
    mrs x2, spsr_el2
    stp x1, x2, [sp, #0x40]
    
    // rust arg0 = &mut TrapFrame.
    mov x0, sp
    bl pyr_sync_lower_el64

1:
    wfe
    b 1b

__pyr_el2_sync_current_sp0:
__pyr_el2_irq_current_sp0:
__pyr_el2_fiq_current_sp0:
__pyr_el2_serror_current_sp0:
__pyr_el2_sync_current_spx:
__pyr_el2_irq_current_spx:
__pyr_el2_fiq_current_spx:
__pyr_el2_serror_current_spx:
__pyr_el2_irq_lower_aarch64:
__pyr_el2_fiq_lower_aarch64:
__pyr_el2_serror_lower_aarch64:
__pyr_el2_sync_lower_aarch32:
__pyr_el2_irq_lower_aarch32:
__pyr_el2_fiq_lower_aarch32:
__pyr_el2_serror_lower_aarch32:
2:
    wfe
    b 2b
