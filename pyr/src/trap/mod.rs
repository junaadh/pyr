mod data_abort;
mod hvc;
mod psci;
mod resume;

use crate::{fatal::halt, trap::hvc::handle_hvc64};
use pyr_arch::{
    exception::{ExceptionClass, TrapFrame},
    sysregs::el2::EsrEl2,
};
pub use resume::*;

#[unsafe(no_mangle)]
pub extern "C" fn pyr_sync_lower_el64(frame: &mut TrapFrame) {
    let esr = EsrEl2::mrs();

    let resume = match esr.decode() {
        ExceptionClass::Hvc64 { imm16 } => handle_hvc64(frame, imm16),
        ExceptionClass::DataAbortLower { iss } => {
            data_abort::handle(frame, iss)
        }
        other => {
            crate::log!("unhandled trap: {other:?}");
            Resume::Halt
        }
    };

    match resume {
        Resume::ReturnToGuest => {}
        Resume::AdvancePcAndReturn => {
            frame.elr_el2 += 4;
        }
        Resume::Halt => {
            crate::log!("halting after trap");
            halt()
        }
    }
}
