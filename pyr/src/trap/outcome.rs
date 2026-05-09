use crate::vcpu::VcpuExitReason;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrapOutcome {
    Return,
    AdvancePc,
    Exit(VcpuExitReason),
}
