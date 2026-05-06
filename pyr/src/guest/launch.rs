use pyr_arch::{
    barrier::isb,
    exception::eret,
    sysregs::{
        el1::{MairEl1, SctlrEl1, SpEl1, TcrEl1, Ttbr0El1, VbarEl1},
        el2::{ElrEl2, SpsrEl2},
    },
};

use crate::guest::config::GuestConfig;

pub fn enter_el1_guest(config: GuestConfig) -> ! {
    crate::log!("entering EL1 guest at {:#018x}", config.entry);
    crate::log!("SP_EL1 = {:#018x}", config.stack_top);

    MairEl1::minimal().msr();
    TcrEl1::disabled_mmu_minimal().msr();
    Ttbr0El1::new(0).msr();
    SctlrEl1::mmu_disabled().msr();
    VbarEl1::new(0).msr();

    SpEl1::new(config.stack_top).msr();
    ElrEl2::new(config.entry).msr();
    SpsrEl2::el1h_masked().msr();

    // TODO: load x0..x3 before ERET when Linux path starts.
    // Current tiny guest sets its own registers.

    isb();

    crate::log!("ELR_EL2 = {:#018x}", ElrEl2::mrs().raw());
    crate::log!("SPSR_EL2 = {:#018x}", SpsrEl2::mrs().raw());

    // SAFETY: ELR_EL2 points to guest entry, SP_EL1 is initialized, and SPSR_EL2 selects EL1h
    unsafe { eret() }
}
