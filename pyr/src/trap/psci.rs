use crate::trap::Resume;
use pyr_arch::exception::TrapFrame;

pub struct Psci;

impl Psci {
    const PSCI_SUCCESS: u64 = 0;
    const PSCI_NOT_SUPPORTED: u64 = (-1i64) as u64;

    const PSCI_VERSION: u64 = 0x8400_0000;
    const PSCI_CPU_ON: u64 = 0x8400_0003;
    const PSCI_AFFINITY_INFO: u64 = 0x8400_0004;
    const PSCI_MIGRATE_INFO_TYPE: u64 = 0x8400_0006;
    const PSCI_SYSTEM_OFF: u64 = 0x8400_0008;
    const PSCI_SYSTEM_RESET: u64 = 0x8400_0009;
    const PSCI_FEATURES: u64 = 0x8400_000a;

    pub const fn is_psci_call(x0: u64) -> bool {
        matches!(
            x0,
            Self::PSCI_VERSION
                | Self::PSCI_MIGRATE_INFO_TYPE
                | Self::PSCI_AFFINITY_INFO
                | Self::PSCI_CPU_ON
                | Self::PSCI_SYSTEM_OFF
                | Self::PSCI_SYSTEM_RESET
                | Self::PSCI_FEATURES
        )
    }

    pub fn handle_call(frame: &mut TrapFrame) -> Resume {
        match frame.x[0] {
            Self::PSCI_VERSION => {
                frame.x[0] = 0x0001_0000; // PSCI 1.0
                Resume::ReturnToGuest
            }

            Self::PSCI_MIGRATE_INFO_TYPE => {
                frame.x[0] = 2; // Trusted OS migration not required
                Resume::ReturnToGuest
            }

            Self::PSCI_AFFINITY_INFO => {
                frame.x[0] = 0; // CPU is on
                Resume::ReturnToGuest
            }

            Self::PSCI_CPU_ON => {
                frame.x[0] = Self::PSCI_NOT_SUPPORTED;
                Resume::ReturnToGuest
            }

            Self::PSCI_SYSTEM_OFF | Self::PSCI_SYSTEM_RESET => {
                crate::log!("psci: power/reset requested");
                Resume::Halt
            }

            Self::PSCI_FEATURES => {
                Self::handle_psci_features(frame);
                Resume::ReturnToGuest
            }

            unreachable => unreachable!(
                "frame.x[0] = {unreachable} should not be caught in Psci::handle_call"
            ),
        }
    }

    const fn handle_psci_features(frame: &mut TrapFrame) {
        let queried = frame.x[1];

        frame.x[0] = match queried {
            Self::PSCI_VERSION
            | Self::PSCI_MIGRATE_INFO_TYPE
            | Self::PSCI_AFFINITY_INFO
            | Self::PSCI_SYSTEM_OFF
            | Self::PSCI_SYSTEM_RESET => Self::PSCI_SUCCESS,
            Self::PSCI_CPU_ON => Self::PSCI_NOT_SUPPORTED,
            _ => Self::PSCI_NOT_SUPPORTED,
        };
    }
}
