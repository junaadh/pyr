use crate::{
    ActivePlatform,
    guest::{self, memory::GuestMemory},
    log,
    stage2::Stage2Vm,
};
use pyr_arch::{
    barrier::isb,
    exception::install_el2_vectors,
    sysregs::{
        common::CurrentEl,
        el2::{HcrEl2, SctlrEl2, VbarEl2, VtcrEl2, VttbrEl2},
    },
};

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

    let mut stage2 = Stage2Vm::new();

    let image = guest::tiny::load_tiny_guest();
    let stack = GuestMemory::stack_region();

    GuestMemory::map_region(&mut stage2, image);
    GuestMemory::map_region(&mut stage2, stack);

    log!("stage2 root = {:#018x}", stage2.root_raw());

    stage2.install();

    guest::tiny::enter_tiny_guest()
}
