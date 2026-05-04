#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::undocumented_unsafe_blocks)]

use core::panic::PanicInfo;

use pyr::pyr_entry;

type Handle = *mut core::ffi::c_void;
type SytemTable = *mut core::ffi::c_void;

#[unsafe(no_mangle)]
pub extern "efiapi" fn efi_main(_image: Handle, _st: SytemTable) -> usize {
    pyr_entry()
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
