#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::undocumented_unsafe_blocks)]

pub mod console;

use pyr_arch::{
    barrier::isb,
    sysregs::{HcrEl2, SctlrEl2, current_el::CurrentEl},
};

#[cfg(feature = "platform-qemu-virt")]
use pyr_platform_qemu::QemuVirt;

#[cfg(feature = "platform-qemu-virt")]
type ActivePlatform = QemuVirt;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::console::_print(core::format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n")
    };

    ($fmt:expr) => {
        $crate::print!(core::concat!($fmt, "\n"))
    };

    ($fmt:expr, $($arg:tt)*) => {
        $crate::print!(core::concat!($fmt, "\n"), $($arg)*)
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::println!("[debug] {}", core::format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        $crate::println!("[pyr] {}", core::format_args!($($arg)*))
    };
}

#[unsafe(no_mangle)]
pub extern "C" fn pyr_entry() -> ! {
    pyr::<ActivePlatform>()
}

pub fn pyr<P>() -> !
where
    P: pyr_arch::platform::Platform,
{
    P::early_init();
    console::init::<P>();
    log!("booting");

    let el = CurrentEl::mrs();
    log!("CurrentEL = {:#018x}", el.raw());
    log!("Exception level = EL{}", el.exception_level());

    let hcr = HcrEl2::mrs();
    let sctlr = SctlrEl2::mrs();

    log!("HCR_EL2 = {:#018x}", hcr.raw());
    log!("SCTLR_EL2 = {:#018x}", sctlr.raw());
    log!("SCTLR_EL2.M = {}", sctlr.mmu_enabled());

    HcrEl2::mrs().with_rw().msr();
    isb();

    log!("HCR_EL2 after RW = {:#018x}", HcrEl2::mrs().raw());

    loop {
        core::hint::spin_loop();
    }
}
