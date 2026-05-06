mod resume;

use crate::{hearth, mmio};
use pyr_arch::{
    exception::{ExceptionClass, TrapFrame},
    sysregs::EsrEl2,
};
pub use resume::*;

#[unsafe(no_mangle)]
pub extern "C" fn pyr_sync_lower_el64(frame: &mut TrapFrame) {
    let esr = EsrEl2::mrs();

    crate::log!("sync lower EL64 trap");
    crate::log!("ESR_EL2 = {:#018x}", esr.raw());
    crate::log!("ELR_EL2 = {:#018x}", frame.elr_el2);
    crate::log!("SPSR_EL2 = {:#018x}", frame.spsr_el2);

    let resume = match esr.decode() {
        ExceptionClass::Hvc64 { imm16 } => {
            crate::log!("trap = HVC64 imm16 = {imm16:#06x}");
            hearth::handle_hvc(frame, imm16)
        }
        ExceptionClass::DataAbortLower { iss } => {
            mmio::handle_data_abort(frame, iss)
        }
        other => {
            crate::log!("unhandled trap: {other:?}");
            Resume::Halt
        }
    };

    match resume {
        Resume::ReturnToGuest => {
            crate::log!("resuming guest @ {:#018x}", frame.elr_el2);
        }
        Resume::AdvancePcAndReturn => {
            frame.elr_el2 += 4;
            crate::log!(
                "advancing and resuming guest @ {:#018x}",
                frame.elr_el2
            );
        }
        Resume::Halt => {
            crate::log!("halting after trap");
            loop {
                core::hint::spin_loop();
            }
        }
    }
}
