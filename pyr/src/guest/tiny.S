    .section .text.guest, "ax"
    .align 4
    .global __tiny_guest_start
    .global __tiny_guest_end

__tiny_guest_start:
    mov x0, #0x7079
    mov x1, #1
    mov x2, #'A'
    hvc #0x0

    mov x0, #0x7079
    mov x1, #1
    mov x2, #'B'
    hvc #0x0
    
    mov x3, #0x09000000
    mov w4, #'X'
    strb w4, [x3]

    mov x0, #0x7079
    mov x1, #1
    mov x2, #'Z'
    hvc #0x0

1:
    wfe
    b 1b

__tiny_guest_end:
