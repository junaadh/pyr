mod resume;

use crate::hearth;
use pyr_arch::{
    exception::{ExceptionClass, TrapFrame},
    sysregs::{EsrEl2, FarEl2, HpfarEl2},
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
            let far = FarEl2::mrs();
            let hpfar = HpfarEl2::mrs();

            crate::log!("trap = DataAbortLower");
            crate::log!("FAR_EL2 = {:#018x}", far.raw());
            crate::log!("HPFAR_EL2 = {:#018x}", hpfar.raw());
            crate::log!("fault IPA base = {:#018x}", hpfar.ipa_base().as_u64());
            crate::log!(
                "data abort iss: dfsc={:#04x} wnr={} s1ptw={} isv={} sas={} srt={}",
                iss.dfsc,
                iss.wnr,
                iss.s1ptw,
                iss.isv,
                iss.sas,
                iss.srt,
            );
            Resume::AdvancePcAndReturn
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
