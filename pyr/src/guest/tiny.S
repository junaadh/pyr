    .section .text.guest, "ax"
    .align 4
    .global __tiny_guest_start
    .global __tiny_guest_end

__tiny_guest_start:
    ldr w3, [x0]
    
    mov x0, #'D'
    bl __pyr_putc

    mov x0, #'A'
    bl __pyr_putc

    mov x0, #'B'
    bl __pyr_putc
    
    mov x3, #0x09000000
    mov w4, #'X'
    strb w4, [x3]

    mov x0, #'Z'
    bl __pyr_putc

    mov x0, #'\n'
    bl __pyr_putc

1:
    wfe
    b 1b

// input:
//   x0 = char
// clobbers:
//   x0, x1, x2
__pyr_putc:
    mov x2, x0       // char
    mov x0, #0x7079  // pyr_hvc_ext
    mov x1, #1       // pyr_hvc_fn
    
    hvc #0x0
    ret

__tiny_guest_end:
