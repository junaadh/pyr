use crate::{guest::config::GuestConfig, vm::VmId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VcpuId(pub u64);

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
}

impl Vcpu {
    pub const fn new(id: VcpuId, vm_id: VmId, config: GuestConfig) -> Self {
        Self {
            id,
            vm_id,
            config,
            state: VcpuState::Created,
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

    pub fn mark_running(&mut self) {
        self.state = VcpuState::Running;
    }

    pub fn mark_halted(&mut self) {
        self.state = VcpuState::Halted;
    }
}
