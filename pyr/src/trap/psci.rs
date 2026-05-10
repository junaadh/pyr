use crate::{trap::TrapOutcome, vcpu::Vcpu};
use pyr_arch::reg::Gpr;

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

    pub fn handle_call(vcpu: &mut Vcpu) -> TrapOutcome {
        match vcpu.context().x(Gpr::X0) {
            Self::PSCI_VERSION => {
                vcpu.context_mut().write_x(Gpr::X0, 0x0001_0000); // PSCI 1.0
                TrapOutcome::Return
            }

            Self::PSCI_MIGRATE_INFO_TYPE => {
                vcpu.context_mut().write_x(Gpr::X0, 2); // Trusted OS migration not required
                TrapOutcome::Return
            }

            Self::PSCI_AFFINITY_INFO => {
                vcpu.context_mut().write_x(Gpr::X0, 0); // CPU is on
                TrapOutcome::Return
            }

            Self::PSCI_CPU_ON => {
                vcpu.context_mut()
                    .write_x(Gpr::X0, Self::PSCI_NOT_SUPPORTED);
                TrapOutcome::Return
            }

            Self::PSCI_SYSTEM_OFF | Self::PSCI_SYSTEM_RESET => {
                crate::log!("psci: power/reset requested");
                TrapOutcome::Exit(crate::vcpu::VcpuExitReason::InternalError)
            }

            Self::PSCI_FEATURES => {
                Self::handle_psci_features(vcpu);
                TrapOutcome::Return
            }

            unreachable => unreachable!(
                "frame.x[0] = {unreachable} should not be caught in Psci::handle_call"
            ),
        }
    }

    fn handle_psci_features(vcpu: &mut Vcpu) {
        let queried = vcpu.context().x(Gpr::X1);

        vcpu.context_mut().write_x(
            Gpr::X0,
            match queried {
                Self::PSCI_VERSION
                | Self::PSCI_MIGRATE_INFO_TYPE
                | Self::PSCI_AFFINITY_INFO
                | Self::PSCI_SYSTEM_OFF
                | Self::PSCI_SYSTEM_RESET => Self::PSCI_SUCCESS,
                Self::PSCI_CPU_ON => Self::PSCI_NOT_SUPPORTED,
                _ => Self::PSCI_NOT_SUPPORTED,
            },
        );
    }
}
