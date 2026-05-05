use pyr_arch::{
    barrier::isb,
    exception::eret,
    sysregs::{ElrEl2, SpsrEl2},
};

#[unsafe(no_mangle)]
pub extern "C" fn tiny_guest_entry() -> ! {
    // SAFETY: This intentionally traps from EL1 to EL2
    unsafe {
        core::arch::asm!("hvc #0");
    }

    loop {
        core::hint::spin_loop();
    }
}

pub fn enter_tiny_guest() -> ! {
    let entry = tiny_guest_entry as *const () as u64;

    crate::log!("entering tiney EL1 guest at {entry:#018x}");

    ElrEl2::new(entry).msr();
    SpsrEl2::el1h_masked().msr();
    isb();

    crate::log!("ELR_EL2 = {:#018x}", ElrEl2::mrs().raw());
    crate::log!("SPSR_EL2 = {:#018x}", SpsrEl2::mrs().raw());

    // SAFETY: ELR_EL2 points to tiny_guest_entry and SPSR_EL2 is EL1h masked
    unsafe { eret() }
}
