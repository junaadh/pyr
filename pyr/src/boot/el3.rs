use pyr_arch::{
    barrier::isb,
    sysregs::el3::{ElrEl3, ScrEl3, SpsrEl3},
};

use crate::log;

pub fn transition_to_el2_with_arg(el2_entry: u64, arg0: u64) -> ! {
    log!("el3: transitioning to EL2={:#x}", el2_entry);

    ScrEl3::new().with_ns().with_rw().with_hce().msr();

    SpsrEl3::el2h_masked().msr();
    ElrEl3::new(el2_entry).msr();

    isb();

    // SAFETY:
    // - ELR_EL3 points to a valid EL2 entry function.
    // - SPSR_EL3 selects EL2h with interrupts masked.
    // - x0 is loaded with the raw boot-info pointer before eret.
    unsafe {
        core::arch::asm!(
            "mov x0, {arg0}",
            "eret",
            arg0 = in(reg) arg0,
            options(noreturn)
        )
    }
}
