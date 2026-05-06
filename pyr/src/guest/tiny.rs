use core::arch::global_asm;

use crate::stage2::SCRATCH;
use pyr_arch::{
    barrier::isb,
    exception::eret,
    sysregs::{ElrEl2, SpEl1, SpsrEl2},
};

global_asm!(include_str!("tiny.S"));

unsafe extern "C" {
    fn __tiny_guest_entry() -> !;
}

pub fn enter_tiny_guest() -> ! {
    let entry = __tiny_guest_entry as *const () as u64;

    // SAFETY: We intentionally define a 4096 array in memory and point the top to the base + len
    let stack_top = unsafe {
        let scratch = &raw const SCRATCH;
        let base = core::ptr::addr_of!((*scratch).guest_stack) as u64;
        base + 16 * 1024
    };

    crate::log!("entering tiney EL1 guest at {entry:#018x}");
    crate::log!("SP_EL1 = {stack_top:#018x}");

    ElrEl2::new(entry).msr();
    SpEl1::new(stack_top).msr();
    SpsrEl2::el1h_masked().msr();
    isb();

    crate::log!("ELR_EL2 = {:#018x}", ElrEl2::mrs().raw());
    crate::log!("SPSR_EL2 = {:#018x}", SpsrEl2::mrs().raw());

    // SAFETY: ELR_EL2 points to tiny_guest_entry and SPSR_EL2 is EL1h masked
    unsafe { eret() }
}
