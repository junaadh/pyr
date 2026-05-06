use pyr_arch::{
    addr::PhysAddr,
    barrier::{dsb_ish, isb},
    sysregs::el2::{HcrEl2, VtcrEl2, VttbrEl2},
};

pub fn enable_stage2(root_pa: u64) {
    configure_translation(root_pa);
    enable_hcr_vm();

    crate::log!("VTCR_EL2 after = {:#018x}", VtcrEl2::mrs().raw());
    crate::log!("VTTBR_EL2 after = {:#018x}", VttbrEl2::mrs().raw());
    crate::log!("HCR_EL2 stage2 = {:#018x}", HcrEl2::mrs().raw());
}

fn configure_translation(root_pa: u64) {
    VtcrEl2::new()
        .with_t0sz(25)
        .with_sl0_level1()
        .with_tg0_4k()
        .with_sh0_inner()
        .with_orgn0_write_back()
        .with_irgn0_write_back()
        .msr();

    VttbrEl2::from_baddr(PhysAddr::new(root_pa)).msr();

    dsb_ish();
    isb();
}

fn enable_hcr_vm() {
    HcrEl2::mrs()
        .without_tge()
        .without_e2h()
        .with_rw()
        .with_amo()
        .with_imo()
        .with_fmo()
        .with_vm()
        .msr();

    isb();
}
