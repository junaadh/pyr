mod resume;

use crate::hearth::{self, HvcCall};
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
        other => {
            crate::log!("unhandled trap: {other:?}");
            Resume::Halt
        }
    };

    match resume {
        Resume::ReturnToGuest => crate::log!(
            "resume requested but vector resume is not implemented yet"
        ),
        Resume::Halt => crate::log!("halting after trap"),
    }

    loop {
        core::hint::spin_loop();
    }
}
