use pyr_arch::{
    exception::ExceptionClass,
    sysregs::{ElrEl2, EsrEl2, SpsrEl2},
};

mod resume;

pub use resume::*;

use crate::hearth;

#[unsafe(no_mangle)]
pub extern "C" fn pyr_sync_lower_el64() {
    let esr = EsrEl2::mrs();
    let elr = ElrEl2::mrs();
    let spsr = SpsrEl2::mrs();

    crate::log!("sync lower EL64 trap");
    crate::log!("ESR_EL2 = {:#018x}", esr.raw());
    crate::log!("ELR_EL2 = {:#018x}", elr.raw());
    crate::log!("SPSR_EL2 = {:#018x}", spsr.raw());

    match esr.decode() {
        ExceptionClass::Hvc64 { imm16 } => {
            crate::log!("trap = HVC64 imm16 = {imm16:#06x}");

            match hearth::handle_hvc(imm16) {
                Resume::ReturnToGuest => {
                    crate::log!("resume requested but not implemented yet");
                }
                Resume::Halt => {
                    crate::log!("halting after HVC")
                }
            }
        }
        other => crate::log!("unhandled trap: {other:?}"),
    }

    loop {
        core::hint::spin_loop();
    }
}
