use pyr_arch::{
    barrier::{dsb_ish, isb},
    page_table::Installed,
    sysregs::el2::{HcrEl2, VtcrEl2, VttbrEl2},
};

use crate::stage2::Stage2Vm;

pub fn enable_stage2(stage2: &Stage2Vm<Installed>) {
    configure_translation(stage2);
    enable_hcr_vm();
}

fn configure_translation(stage2: &Stage2Vm<Installed>) {
    VtcrEl2::new()
        .with_t0sz(25)
        .with_sl0_level1()
        .with_tg0_4k()
        .with_sh0_inner()
        .with_orgn0_write_back()
        .with_irgn0_write_back()
        .msr();

    VttbrEl2::from_vmid_baddr(stage2.vmid().as_u16(), stage2.root_pa()).msr();

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
