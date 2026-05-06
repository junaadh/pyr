use pyr_arch::{
    barrier::isb,
    exception::eret,
    sysregs::el3::{ElrEl3, ScrEl3, SpsrEl3},
};

use crate::log;

pub fn transition_to_el2(el2_entry: u64) -> ! {
    log!("tranitioning EL3 -> EL2");

    ScrEl3::new().with_ns().with_rw().with_hce().msr();

    SpsrEl3::el2h_masked().msr();
    ElrEl3::new(el2_entry).msr();

    isb();

    // SAFETY: ELR_EL3 points to el2_entry and SPSR_EL3 is EL1h masked
    unsafe { eret() }
}
