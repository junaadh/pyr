use crate::{
    guest::config::GuestConfig,
    id::{VcpuId, VmId},
};
use pyr_arch::{exception::TrapFrame, platform::GuestReg};

pub mod runner;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VcpuState {
    Created,
    Runnable,
    Running,
    Blocked,
    Halted(VcpuExitReason),
}

impl VcpuState {
    pub const fn is_halted(self) -> bool {
        matches!(self, Self::Halted(_))
    }

    pub const fn is_running(self) -> bool {
        matches!(self, Self::Running)
    }

    pub const fn is_runnable(self) -> bool {
        matches!(self, Self::Runnable)
    }

    pub const fn exit_reason(&self) -> VcpuExitReason {
        if let VcpuState::Halted(reason) = self {
            *reason
        } else {
            VcpuExitReason::None
        }
    }
}

pub struct Vcpu {
    id: VcpuId,
    vm_id: VmId,
    config: GuestConfig,
    state: VcpuState,
    traps: u64,
}

impl Vcpu {
    pub const fn new(id: VcpuId, vm_id: VmId, config: GuestConfig) -> Self {
        Self {
            id,
            vm_id,
            config,
            state: VcpuState::Created,
            traps: 0,
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
        self.state.exit_reason()
    }

    pub fn make_runnable(&mut self) {
        if matches!(self.state, VcpuState::Created | VcpuState::Blocked) {
            self.state = VcpuState::Runnable;
        }
    }

    pub fn enter_running(&mut self) {
        debug_assert!(
            matches!(self.state, VcpuState::Created | VcpuState::Runnable),
            "invalid vCPU transition into Running from {:?}",
            self.state
        );

        self.state = VcpuState::Running;
    }

    pub fn block(&mut self) {
        debug_assert_eq!(self.state, VcpuState::Running);
        self.state = VcpuState::Blocked;
    }

    pub fn halt(&mut self, reason: VcpuExitReason) {
        self.state = VcpuState::Halted(reason);
    }

    pub const fn is_halted(&self) -> bool {
        matches!(self.state, VcpuState::Halted(_))
    }

    pub const fn is_running(&self) -> bool {
        matches!(self.state, VcpuState::Running)
    }

    pub fn record_trap(&mut self) {
        self.traps = self.traps.wrapping_add(1);
    }

    pub const fn advance_pc(&mut self, frame: &mut TrapFrame) {
        frame.elr_el2 = frame.elr_el2.wrapping_add(4);
    }

    pub fn read_gpr(&self, frame: &TrapFrame, reg: GuestReg) -> Option<u64> {
        match reg {
            GuestReg::Gpr(index) => frame.x.get(index as usize).copied(),
            GuestReg::Zero => Some(0),
        }
    }

    pub fn write_gpr(
        &mut self,
        frame: &mut TrapFrame,
        reg: GuestReg,
        value: u64,
    ) -> Option<()> {
        match reg {
            GuestReg::Zero => Some(()),
            GuestReg::Gpr(index) => {
                let slot = frame.x.get_mut(index as usize)?;
                *slot = value;
                Some(())
            }
        }
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
