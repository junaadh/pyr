#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::undocumented_unsafe_blocks)]

use core::panic::PanicInfo;
use pyr::{
    boot::{el2, el3},
    fatal,
};
use pyr_arch::{
    boot::{abi::RawBootInfo, info::BootInfo},
    sysregs::common::CurrentEl,
};
core::arch::global_asm!(include_str!("start.S"));

/// # Safety
///
/// Tbis function is the entry point for the bare route of the `pyr` hypervisor
/// The `start.S` trampoline ensures that the `raw` satisfies the RawBootInfo ABI
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pyr_entry(raw: *const RawBootInfo) -> ! {
    match CurrentEl::mrs().exception_level() {
        3 => el3::transition_to_el2_with_arg(
            el2::pyr_el2_entry_raw as *const () as u64,
            raw as u64,
        ),
        // SAFETY: The caller guarantees that `raw` satisfies the RawBootInfo ABI
        2 => unsafe { el2::pyr_el2_entry_raw(raw) },
        level => crate::fatal!("unsupported exception level: EL{level}"),
    }
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    fatal!("{info:?}")
}
