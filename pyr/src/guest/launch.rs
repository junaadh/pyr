use crate::guest::config::GuestConfig;
use pyr_arch::{
    barrier::isb,
    sysregs::{
        el1::{MairEl1, SctlrEl1, SpEl1, TcrEl1, Ttbr0El1, VbarEl1},
        el2::{CnthctlEl2, CntvoffEl2, ElrEl2, SpsrEl2},
    },
};

pub fn enter_el1_guest(config: GuestConfig) -> ! {
    crate::log!("entering EL1 guest at {:#018x}", config.entry);
    crate::log!("SP_EL1 = {:#018x}", config.stack_top);

    MairEl1::minimal().msr();
    TcrEl1::disabled_mmu_minimal().msr();
    Ttbr0El1::new(0).msr();
    VbarEl1::new(0).msr();
    SctlrEl1::linux_reset().msr();

    ElrEl2::new(config.entry).msr();
    SpEl1::new(config.stack_top).msr();

    crate::log!("ELR_EL2 = {:#018x}", ElrEl2::mrs().raw());

    #[cfg(feature = "boot-tiny")]
    {
        SpsrEl2::el1h_masked().msr();
    }

    #[cfg(feature = "boot-linux")]
    {
        SpsrEl2::el1h_linux().msr();
    }

    isb();

    crate::log!("SPSR_EL2 = {:#018x}", SpsrEl2::mrs().raw());
    crate::log!("SCTLR_EL1 = {:#018x}", SctlrEl1::mrs().raw());

    CntvoffEl2::new(0).msr();

    CnthctlEl2::mrs().with_el1pcten().with_el1pcen().msr();

    crate::log!("CNTHCTL_EL2 = {:#018x}", CnthctlEl2::mrs().raw());
    crate::log!("CNTVOFF_EL2 = {:#018x}", CntvoffEl2::mrs().raw());

    let base = crate::stage2::scratch::guest_ram_base();

    for i in 0..4 {
        // SAFETY: Intentional read unaligned for debugging linux image load
        let word =
            unsafe { core::ptr::read_unaligned((base as *const u32).add(i)) };
        crate::log!("kernel[{i}] = {word:#010x}");
    }

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
