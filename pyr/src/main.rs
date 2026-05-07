#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::undocumented_unsafe_blocks)]

use core::panic::PanicInfo;
use pyr::fatal;
core::arch::global_asm!(include_str!("start.S"));

#[unsafe(no_mangle)]
pub extern "C" fn pyr_bare_entry() -> ! {
    pyr::boot::enter()
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    fatal!("{info:?}")
}
