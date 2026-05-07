.section .text._start, "ax"
.align 11
.global _start

_start:
    ldr x0, =__boot_stack_top
    mov sp, x0

    ldr x0, =__pyr_bare_boot_info
    bl pyr_entry

1:
    wfe
    b 1b


.section .rodata.boot_info, "a"
.align 3

.global __pyr_bare_boot_info
__pyr_bare_boot_info:
    .ascii "FCKUEL1\0"
    .word 1
    .word __pyr_bare_boot_info_end - __pyr_bare_boot_info

    .word 1
    .word 1
    .word 2
    .word 0

    .quad 0x3

    .quad __pyr_bare_memory_map
    .quad (__pyr_bare_memory_map_end - __pyr_bare_memory_map) / 32

    .quad __pyr_bare_modules
    .quad (__pyr_bare_modules_end - __pyr_bare_modules) / 64

    .quad 0
    .quad 0

    .quad 0
    .quad 0

    .quad 0
    .quad 0

    .quad 0
    .quad 0
__pyr_bare_boot_info_end:


.align 3
.global __pyr_bare_memory_map
__pyr_bare_memory_map:
    .quad __pyr_image_start
    .quad __pyr_image_size
    .word 4
    .word 0
    .quad 0x5

    .quad __boot_stack
    .quad __boot_stack_top - __boot_stack
    .word 5
    .word 0
    .quad 0x3

    .quad __heap_start
    .quad __heap_size
    .word 6
    .word 0
    .quad 0x43

    .quad __frame_pool_start
    .quad __frame_pool_size
    .word 7
    .word 0
    .quad 0x43

    .quad 0x40000000
    .quad 0x40000000
    .word 8
    .word 0
    .quad 0x43

    .quad 0x08000000
    .quad 0x00020000
    .word 12
    .word 0
    .quad 0x0b

    .quad 0x09000000
    .quad 0x00001000
    .word 12
    .word 0
    .quad 0x0b
__pyr_bare_memory_map_end:


.align 3
.global __pyr_bare_modules
__pyr_bare_modules:
    .quad __linux_image_start
    .quad __linux_image_end - __linux_image_start
    .word 1
    .word 0
    .quad 0x9
    .quad __name_linux
    .quad __name_linux_end - __name_linux
    .quad 0x40000000
    .quad 0x200000

    .quad __dtb_start
    .quad __dtb_end - __dtb_start
    .word 2
    .word 0
    .quad 0x1
    .quad __name_dtb
    .quad __name_dtb_end - __name_dtb
    .quad 0x7f000000
    .quad 0x8

    .quad __initrd_start
    .quad __initrd_end - __initrd_start
    .word 3
    .word 0
    .quad 0x1
    .quad __name_initrd
    .quad __name_initrd_end - __name_initrd
    .quad 0x78000000
    .quad 0x1000
__pyr_bare_modules_end:


__name_linux:
    .ascii "linux"
__name_linux_end:

__name_dtb:
    .ascii "dtb"
__name_dtb_end:

__name_initrd:
    .ascii "initrd"
__name_initrd_end:


.section .rodata.assets, "a"
.align 12

.global __linux_image_start
__linux_image_start:
    .incbin "pyr/assets/img"
.global __linux_image_end
__linux_image_end:

.align 3
.global __dtb_start
__dtb_start:
    .incbin "pyr/assets/qemu-virt.dtb"
.global __dtb_end
__dtb_end:

.align 12
.global __initrd_start
__initrd_start:
    .incbin "pyr/assets/initramfs.cpio"
.global __initrd_end
__initrd_end:


.section .bss.boot_stack, "aw", %nobits
.align 12
.global __boot_stack
__boot_stack:
    .space 0x4000
.global __boot_stack_top
__boot_stack_top:
