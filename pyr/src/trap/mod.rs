mod resume;

use crate::{hearth, mmio};
use pyr_arch::{
    exception::{ExceptionClass, TrapFrame},
    sysregs::el2::{EsrEl2, FarEl2},
};
pub use resume::*;

#[unsafe(no_mangle)]
pub extern "C" fn pyr_sync_lower_el64(frame: &mut TrapFrame) {
    let esr = EsrEl2::mrs();

    let resume = match esr.decode() {
        ExceptionClass::Hvc64 { imm16 } => {
            match frame.x[0] {
                0x8400_0000 => {
                    // PSCI_VERSION
                    frame.x[0] = 0x0001_0000; // PSCI 1.0
                    Resume::ReturnToGuest
                }

                0x8400_0003 => {
                    // PSCI_CPU_ON, single CPU prototype: not supported
                    frame.x[0] = (-1i64) as u64; // NOT_SUPPORTED
                    Resume::ReturnToGuest
                }

                0x8400_0008 => {
                    // PSCI_SYSTEM_OFF
                    crate::log!("psci: system_off");
                    Resume::Halt
                }

                0x8400_0009 => {
                    // PSCI_SYSTEM_RESET
                    crate::log!("psci: system_reset");
                    Resume::Halt
                }
                _ => hearth::handle_hvc(frame, imm16),
            }
        }
        ExceptionClass::DataAbortLower { iss } => {
            if iss.dfsc == 0x21 {
                crate::log!("guest alignment fault");
                crate::log!("FAR_EL2 = {:#018x}", FarEl2::mrs().raw());
                Resume::Halt
            } else {
                mmio::handle_data_abort(frame, iss)
            }
        }
        ExceptionClass::SysregTrap { iss } => {
            crate::log!("trap = SysremTrap iss = {iss:#010x}");
            Resume::Halt
        }
        other => {
            crate::log!("unhandled trap: {other:?}");
            Resume::Halt
        }
    };

    match resume {
        Resume::ReturnToGuest => {
            // crate::log!("resuming guest @ {:#018x}", frame.elr_el2);
        }
        Resume::AdvancePcAndReturn => {
            frame.elr_el2 += 4;
            // crate::log!(
            //     "advancing and resuming guest @ {:#018x}",
            //     frame.elr_el2
            // );
        }
        Resume::Halt => {
            crate::log!("halting after trap");
            loop {
                core::hint::spin_loop();
            }
        }
    }
}
