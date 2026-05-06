#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::undocumented_unsafe_blocks)]

pub mod console;
pub mod guest;
pub mod hearth;
pub mod mmio;
pub mod stage2;
pub mod trap;

use pyr_arch::{
    barrier::isb,
    exception::install_el2_vectors,
    sysregs::{
        HcrEl2, SctlrEl2, VbarEl2, VtcrEl2, VttbrEl2, current_el::CurrentEl,
    },
};

#[cfg(feature = "platform-qemu-virt")]
use pyr_platform_qemu::QemuVirt;

#[cfg(feature = "platform-qemu-virt")]
pub(crate) type ActivePlatform = QemuVirt;

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
    log!("booting");

    install_el2_vectors();

    log!("VBAR_EL2 = {:#018x}", VbarEl2::mrs().raw());

    let el = CurrentEl::mrs();
    log!("CurrentEL = {:#018x}", el.raw());
    log!("Exception level = EL{}", el.exception_level());

    let hcr = HcrEl2::mrs();
    let sctlr = SctlrEl2::mrs();

    log!("HCR_EL2 = {:#018x}", hcr.raw());
    log!("SCTLR_EL2 = {:#018x}", sctlr.raw());
    log!("SCTLR_EL2.M = {}", sctlr.mmu_enabled());

    HcrEl2::mrs()
        .without_tge()
        .without_e2h()
        .with_rw()
        .with_amo()
        .with_imo()
        .with_fmo()
        .msr();
    isb();

    log!("HCR_EL2 after RW = {:#018x}", HcrEl2::mrs().raw());

    log!("VTCR_EL2 = {:#018x}", VtcrEl2::mrs().raw());
    log!("VTTBR_EL2 = {:#018x}", VttbrEl2::mrs().raw());

    let stage2 = stage2::build_identity_map();
    log!("stage2 root = {:#018x}", stage2.root_raw());

    stage2::enable_stage2(stage2.root_raw());

    guest::tiny::enter_tiny_guest()
}
