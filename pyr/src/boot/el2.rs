mod linux;
mod tiny;

use crate::{ActivePlatform, log};
use pyr_arch::{
    barrier::isb,
    exception::install_el2_vectors,
    platform::Platform,
    sysregs::{
        common::CurrentEl,
        el2::{HcrEl2, SctlrEl2, VbarEl2, VtcrEl2, VttbrEl2},
    },
};

#[cfg(all(feature = "boot-tiny", feature = "boot-linux"))]
compile_error!("Only one feature can be active at the same time");

#[cfg(not(any(feature = "boot-tiny", feature = "boot-linux")))]
compile_error!("One feature needs to be activated");

#[unsafe(no_mangle)]
pub extern "C" fn pyr_entry() -> ! {
    init_el2();

    #[cfg(feature = "boot-linux")]
    {
        use crate::boot::el2::linux::boot_linux;

        static LINUX_IMAGE: &[u8] = include_bytes!("../../assets/img");
        static DTB: &[u8] = include_bytes!("../../assets/qemu-virt.dtb");

        boot_linux(LINUX_IMAGE, DTB)
    }

    #[cfg(feature = "boot-tiny")]
    {
        tiny::boot_tiny()
    }
}

pub fn init_el2() {
    <ActivePlatform as Platform>::early_init();
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
}
