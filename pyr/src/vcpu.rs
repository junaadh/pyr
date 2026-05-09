use crate::{guest::config::GuestConfig, vm::VmId};
use core::fmt;

pub mod runner;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct VcpuId {
    vm: VmId,
    index: u16,
}

impl VcpuId {
    pub const fn from_parts(vm: VmId, index: u16) -> Self {
        Self { vm, index }
    }

    pub const fn vm(self) -> VmId {
        self.vm
    }

    pub const fn index(self) -> u16 {
        self.index
    }
}

impl fmt::Debug for VcpuId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let short = (self.vm.as_u64() >> 32) as u32;

        write!(f, "vcpu:{short:08x}:{:04x}", self.index)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VcpuState {
    Created,
    Running,
    Halted,
}

pub struct Vcpu {
    id: VcpuId,
    vm_id: VmId,
    config: GuestConfig,
    state: VcpuState,
    traps: u64,
    exit_reason: VcpuExitReason,
}

impl Vcpu {
    pub const fn new(id: VcpuId, vm_id: VmId, config: GuestConfig) -> Self {
        Self {
            id,
            vm_id,
            config,
            state: VcpuState::Created,
            traps: 0,
            exit_reason: VcpuExitReason::None,
        }
    }

    pub const fn id(&self) -> VcpuId {
        self.id
    }

    pub const fn vm_id(&self) -> VmId {
        self.vm_id
    }

    pub const fn config(&self) -> GuestConfig {
        self.config
    }

    pub const fn state(&self) -> VcpuState {
        self.state
    }

    pub const fn trap_count(&self) -> u64 {
        self.traps
    }

    pub const fn exit_reason(&self) -> VcpuExitReason {
        self.exit_reason
    }

    pub fn stop(&mut self, reason: VcpuExitReason) {
        self.state = VcpuState::Halted;
        self.exit_reason = reason;
    }

    pub fn record_trap(&mut self) {
        self.traps = self.traps.wrapping_add(1);
    }

    pub fn mark_running(&mut self) {
        self.state = VcpuState::Running;
    }

    pub fn mark_halted(&mut self) {
        self.state = VcpuState::Halted;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VcpuExitReason {
    None,
    Halted,
    UnhandledTrap,
    MmioError,
    InternalError,
}
