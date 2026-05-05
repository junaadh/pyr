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
    // Allocate TrapFrame: 0x110 bytes
    sub sp, sp, #0x110

    stp x0,  x1,  [sp, #0x000]
    stp x2,  x3,  [sp, #0x010]
    stp x4,  x5,  [sp, #0x020]
    stp x6,  x7,  [sp, #0x030]
    stp x8,  x9,  [sp, #0x040]
    stp x10, x11, [sp, #0x050]
    stp x12, x13, [sp, #0x060]
    stp x14, x15, [sp, #0x070]
    stp x16, x17, [sp, #0x080]
    stp x18, x19, [sp, #0x090]
    stp x20, x21, [sp, #0x0A0]
    stp x22, x23, [sp, #0x0B0]
    stp x24, x25, [sp, #0x0C0]
    stp x26, x27, [sp, #0x0D0]
    stp x28, x29, [sp, #0x0E0]

    str x30, [sp, #0x0F0]
    str xzr, [sp, #0x0F8]

    // Save ELR_EL2 and SPSR_EL2
    mrs x1, elr_el2
    mrs x2, spsr_el2
    stp x1, x2, [sp, #0x100]
    
    // rust arg0 = &mut TrapFrame.
    mov x0, sp
    bl pyr_sync_lower_el64

    // Return path

    // Restore ELR_EL2 / SPSR_EL2
    ldp x1, x2, [sp, #0x100]
    msr elr_el2, x1
    msr spsr_el2, x2

    // Restore x0..x30
    ldp x0,  x1,  [sp, #0x000]
    ldp x2,  x3,  [sp, #0x010]
    ldp x4,  x5,  [sp, #0x020]
    ldp x6,  x7,  [sp, #0x030]
    ldp x8,  x9,  [sp, #0x040]
    ldp x10, x11, [sp, #0x050]
    ldp x12, x13, [sp, #0x060]
    ldp x14, x15, [sp, #0x070]
    ldp x16, x17, [sp, #0x080]
    ldp x18, x19, [sp, #0x090]
    ldp x20, x21, [sp, #0x0A0]
    ldp x22, x23, [sp, #0x0B0]
    ldp x24, x25, [sp, #0x0C0]
    ldp x26, x27, [sp, #0x0D0]
    ldp x28, x29, [sp, #0x0E0]

    ldr x30, [sp, #0x0f0]

    add sp, sp, #0x110
    eret

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
