use crate::{guest::config::GuestConfig, vm::VmId};
use core::fmt;

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
