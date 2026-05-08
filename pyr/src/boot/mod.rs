use pyr_arch::{boot::abi::RawBootInfo, sysregs::common::CurrentEl};

pub mod el2;
pub mod el3;

/// # Safety
///
/// Tbis function is the entry point for the bare route of the `pyr` hypervisor
/// The `start.S` trampoline ensures that the `raw` satisfies the RawBootInfo ABI
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pyr_entry_raw(raw: *const RawBootInfo) -> ! {
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
