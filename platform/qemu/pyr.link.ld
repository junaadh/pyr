ENTRY(_start)

SECTIONS
{
  . = 0x40080000;

  __pyr_image_start = .;

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

  . = ALIGN(4K);
  __pyr_image_end = .;
  __pyr_image_size = __pyr_image_end - __pyr_image_start;

  . = ALIGN(4K);
  __heap_start = .;
  . += 2M;
  __heap_end = .;
  __heap_size = __heap_end - __heap_start;

  . = ALIGN(4K);
  __frame_pool_start = .;
  . += 16M;
  __frame_pool_end = .;
  __frame_pool_size = __frame_pool_end - __frame_pool_start;

  __image_end = __frame_pool_end;

  /DISCARD/ : {
    *(.eh_frame*)
    *(.comment*)
    *(.note*)
  }
}
