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
            const PSCI_SUCCESS: u64 = 0;
            const PSCI_NOT_SUPPORTED: u64 = (-1i64) as u64;

            const PSCI_VERSION: u64 = 0x8400_0000;
            const PSCI_CPU_ON: u64 = 0x8400_0003;
            const PSCI_AFFINITY_INFO: u64 = 0x8400_0004;
            const PSCI_MIGRATE_INFO_TYPE: u64 = 0x8400_0006;
            const PSCI_SYSTEM_OFF: u64 = 0x8400_0008;
            const PSCI_SYSTEM_RESET: u64 = 0x8400_0009;
            const PSCI_FEATURES: u64 = 0x8400_000a;

            match frame.x[0] {
                PSCI_VERSION => {
                    frame.x[0] = 0x0001_0000; // PSCI 1.0
                    Resume::ReturnToGuest
                }

                PSCI_MIGRATE_INFO_TYPE => {
                    frame.x[0] = 2; // Trusted OS migration not required
                    Resume::ReturnToGuest
                }

                PSCI_AFFINITY_INFO => {
                    frame.x[0] = 0; // CPU is on
                    Resume::ReturnToGuest
                }

                PSCI_CPU_ON => {
                    frame.x[0] = PSCI_NOT_SUPPORTED;
                    Resume::ReturnToGuest
                }

                PSCI_SYSTEM_OFF | PSCI_SYSTEM_RESET => {
                    crate::log!("psci: power/reset requested");
                    Resume::Halt
                }

                PSCI_FEATURES => {
                    let queried = frame.x[1];

                    frame.x[0] = match queried {
                        PSCI_VERSION
                        | PSCI_MIGRATE_INFO_TYPE
                        | PSCI_AFFINITY_INFO
                        | PSCI_SYSTEM_OFF
                        | PSCI_SYSTEM_RESET => PSCI_SUCCESS,
                        PSCI_CPU_ON => PSCI_NOT_SUPPORTED,
                        _ => PSCI_NOT_SUPPORTED,
                    };

                    Resume::ReturnToGuest
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
