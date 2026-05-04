#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::undocumented_unsafe_blocks)]

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    pyr::pyr_entry()
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    puts("[pyr] panic\n");
    loop {}
}
