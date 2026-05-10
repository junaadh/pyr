use crate::vcpu::{VcpuBlockReason, VcpuExitReason};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrapOutcome {
    Return,
    AdvancePc,
    Block(VcpuBlockReason),
    Exit(VcpuExitReason),
}
