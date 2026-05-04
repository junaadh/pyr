ENTRY(_start)

SECTIONS
{
  . = 0x40080000

  .text : ALIGN(4K) {
    KEEP(*(.text._start))
    *(.text*)
  }

  .rodata : ALIGN(4K) {
    *(.rodata*)
  }

  .data : ALIGN(4K) {
    *(.data*)
  }

  .bss : ALIGN(4K) {
    __bss_start = .;
    *(.bss*)
    *(COMMON)
    __bss_end = .;
  }

  __image_end = .;

  /DISCARD/ : {
    *(.eh_frame*)
    *(.comment*)
    *(.note*)
  }
}

