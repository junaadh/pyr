.section .text.el2_vectors, "ax"
.align 11
.global __el2_vector_table

__el2_vector_table:
    // Current EL with SP0
    b __el2_sync_current_sp0
    .space 0x80 - 4
    b __el2_irq_current_sp0
    .space 0x80 - 4
    b __el2_fiq_current_sp0
    .space 0x80 - 4
    b __el2_serror_current_sp0
    .space 0x80 - 4

    // Current EL with SPx
    b __el2_sync_current_spx
    .space 0x80 - 4
    b __el2_irq_current_spx
    .space 0x80 - 4
    b __el2_fiq_current_spx
    .space 0x80 - 4
    b __el2_serror_current_spx
    .space 0x80 - 4

    // Lower EL using AArch64
    b __el2_sync_lower_aarch64
    .space 0x80 - 4
    b __el2_irq_lower_aarch64
    .space 0x80 - 4
    b __el2_fiq_lower_aarch64
    .space 0x80 - 4
    b __el2_serror_lower_aarch64
    .space 0x80 - 4

    // Lower EL using AArch32
    b __el2_sync_lower_aarch32
    .space 0x80 - 4
    b __el2_irq_lower_aarch32
    .space 0x80 - 4
    b __el2_fiq_lower_aarch32
    .space 0x80 - 4
    b __el2_serror_lower_aarch32
    .space 0x80 - 4

__el2_sync_current_sp0:
__el2_irq_current_sp0:
__el2_fiq_current_sp0:
__el2_serror_current_sp0:
__el2_sync_current_spx:
__el2_irq_current_spx:
__el2_fiq_current_spx:
__el2_serror_current_spx:
__el2_sync_lower_aarch64:
__el2_irq_lower_aarch64:
__el2_fiq_lower_aarch64:
__el2_serror_lower_aarch64:
__el2_sync_lower_aarch32:
__el2_irq_lower_aarch32:
__el2_fiq_lower_aarch32:
__el2_serror_lower_aarch32:
1:
    wfe
    b 1b
