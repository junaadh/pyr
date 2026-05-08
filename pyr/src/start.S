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

    .word 1                      // BareEntry
    .word 1                      // QemuVirt
    .word 2                      // EL2
    .word 0

    .quad 0x3                    // HAS_MEMORY_MAP | HAS_RESOURCES

    .quad __pyr_bare_memory_map
    .quad (__pyr_bare_memory_map_end - __pyr_bare_memory_map) / 32

    .quad __pyr_bare_resources
    .quad (__pyr_bare_resources_end - __pyr_bare_resources) / 80

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
    .word 4                      // HypervisorImage
    .word 0
    .quad 0x5                    // READABLE | EXECUTABLE

    .quad __boot_stack
    .quad __boot_stack_top - __boot_stack
    .word 5                      // HypervisorStack
    .word 0
    .quad 0x3                    // READABLE | WRITABLE

    .quad __heap_start
    .quad __heap_size
    .word 6                      // HypervisorHeap
    .word 0
    .quad 0x43                   // READABLE | WRITABLE | ZEROED

    .quad __frame_pool_start
    .quad __frame_pool_size
    .word 7                      // FramePool
    .word 0
    .quad 0x43                   // READABLE | WRITABLE | ZEROED

    .quad 0x40000000
    .quad 0x40000000
    .word 8                      // PyrReserved, temporary dev guest RAM arena
    .word 0
    .quad 0x43

    .quad 0x08000000
    .quad 0x00020000
    .word 11                     // Mmio
    .word 0
    .quad 0x0b

    .quad 0x09000000
    .quad 0x00001000
    .word 11                     // Mmio
    .word 0
    .quad 0x0b
__pyr_bare_memory_map_end:


.align 3
.global __pyr_bare_resources
__pyr_bare_resources:
    // dev-linux-kernel
    .quad __linux_image_start
    .quad __linux_image_end - __linux_image_start
    .word 8                      // DevPayload
    .word 0
    .quad 0x9                    // REQUIRED | EXECUTABLE
    .quad __name_linux
    .quad __name_linux_end - __name_linux
    .word 3                      // Embedded
    .word 0
    .quad 0x200000               // align
    .quad 0                      // metadata_ptr
    .quad 0                      // metadata_len

    // dev-linux-dtb
    .quad __dtb_start
    .quad __dtb_end - __dtb_start
    .word 2                      // PlatformDtb
    .word 0
    .quad 0x11                   // REQUIRED | TRUSTED
    .quad __name_dtb
    .quad __name_dtb_end - __name_dtb
    .word 3                      // Embedded
    .word 0
    .quad 0x8
    .quad 0
    .quad 0

    // dev-linux-initrd
    .quad __initrd_start
    .quad __initrd_end - __initrd_start
    .word 8                      // DevPayload
    .word 0
    .quad 0x1                    // REQUIRED
    .quad __name_initrd
    .quad __name_initrd_end - __name_initrd
    .word 3                      // Embedded
    .word 0
    .quad 0x1000
    .quad 0
    .quad 0
__pyr_bare_resources_end:


__name_linux:
    .ascii "dev-linux-kernel"
__name_linux_end:

__name_dtb:
    .ascii "dev-linux-dtb"
__name_dtb_end:

__name_initrd:
    .ascii "dev-linux-initrd"
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
