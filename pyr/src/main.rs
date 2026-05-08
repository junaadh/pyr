#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::undocumented_unsafe_blocks)]

use core::panic::PanicInfo;
use pyr::{boot::pyr_entry_raw, fatal};
use pyr_arch::boot::abi::RawBootInfo;
core::arch::global_asm!(include_str!("start.S"));

/// # Safety
///
/// Tbis function is the entry point for the bare route of the `pyr` hypervisor
/// The `start.S` trampoline ensures that the `raw` satisfies the RawBootInfo ABI
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pyr_entry(raw: *const RawBootInfo) -> ! {
    // SAFETY: The caller guarentees the validity of `raw` according to the RawBootInfo ABI
    unsafe { pyr_entry_raw(raw) }
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    fatal!("{info:?}")
}
