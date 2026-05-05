use pyr_arch::sysregs::{ElrEl2, EsrEl2, SpsrEl2};

#[unsafe(no_mangle)]
pub extern "C" fn pyr_sync_lower_el64() {
    let esr = EsrEl2::mrs();
    let elr = ElrEl2::mrs();
    let spsr = SpsrEl2::mrs();

    crate::log!("sync lower EL64 trap");
    crate::log!("ESR_EL2 = {:#018x}", esr.raw());
    crate::log!("ELR_EL2 = {:#018x}", elr.raw());
    crate::log!("SPSR_EL2 = {:#018x}", spsr.raw());
    crate::log!("EC = {:#04x}", esr.ec());
    crate::log!("ISS = {:#010x}", esr.iss());

    if esr.is_hvc64() {
        crate::log!("HVC64 trapped");
    } else {
        crate::log!("unexpected trap EC");
    }

    loop {
        core::hint::spin_loop();
    }
}
