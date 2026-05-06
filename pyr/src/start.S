.section .text._start, "ax"
.align 11
.global _start

_start:
    ldr x0, =__boot_stack_top
    mov sp, x0
    bl pyr_bare_entry

1:
    wfe
    b 1b

.section .bss.boot_stack, "aw", %nobits
.align 12
__boot_stack:
    .space 0x4000
__boot_stack_top:
