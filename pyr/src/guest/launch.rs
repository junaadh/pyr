use crate::guest::config::GuestConfig;
use pyr_arch::{
    barrier::isb,
    sysregs::{
        el1::{MairEl1, SctlrEl1, SpEl1, TcrEl1, Ttbr0El1, VbarEl1},
        el2::{CnthctlEl2, CntvoffEl2, ElrEl2, SpsrEl2},
    },
};

pub(crate) fn enter_el1_guest(config: GuestConfig) -> ! {
    crate::log!(
        "el1: entry={:#018x} sp={:#018x} x0={:#x}",
        config.entry,
        config.stack_top,
        config.x0
    );

    MairEl1::minimal().msr();
    TcrEl1::disabled_mmu_minimal().msr();
    Ttbr0El1::new(0).msr();
    VbarEl1::new(0).msr();
    SctlrEl1::linux_reset().msr();

    ElrEl2::new(config.entry).msr();
    SpEl1::new(config.stack_top).msr();

    #[cfg(feature = "boot-tiny")]
    {
        SpsrEl2::el1h_masked().msr();
    }

    #[cfg(feature = "boot-linux")]
    {
        SpsrEl2::el1h_linux().msr();
    }

    isb();

    CntvoffEl2::new(0).msr();
    CnthctlEl2::mrs().with_el1pcten().with_el1pcen().msr();

    // SAFETY: ELR_EL2 points to guest entry, SP_EL1 is initialized, and SPSR_EL2 selects EL1h
    unsafe { eret_with_args(config) }
}

/// # SAFETY
///
/// Caller must ensure that `ELR_EL2` and `SPSR_EL2` are configured before calling `eret`
/// x0-x3 are intentionally loaded as guest initial ABI registers.
#[inline(always)]
unsafe fn eret_with_args(config: GuestConfig) -> ! {
    // SAFETY: Caller must have configured ELR_EL2 and SPSR_EL2 correctly
    unsafe {
        core::arch::asm!(
            "mov x0, {x0}",
            "mov x1, {x1}",
            "mov x2, {x2}",
            "mov x3, {x3}",
            "eret",
            x0 = in(reg) config.x0,
            x1 = in(reg) config.x1,
            x2 = in(reg) config.x2,
            x3 = in(reg) config.x3,
            options(noreturn)
        )
    }
}
