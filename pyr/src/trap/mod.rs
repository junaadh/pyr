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

            let ipa = hpfar.ipa_base().as_u64() | (far.raw() & 0xfff);
            crate::log!("fault IPA = {ipa:#018x}");

            if iss.wnr && iss.sas == 0 && ipa == 0x0900_0000 {
                let reg = iss.srt as usize;

                if let Some(byte) = frame.x.get(reg) {
                    let byte = *byte as u8;

                    crate::log!("mmio: PL011 write byte {}", byte as char);
                    crate::print!("{}", byte as char);

                    Resume::AdvancePcAndReturn
                } else {
                    crate::log!("invalid syndrome register: {reg}");
                    Resume::Halt
                }
            } else {
                crate::log!("unhandled data abort");
                Resume::Halt
            }
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
