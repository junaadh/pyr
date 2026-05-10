use crate::{
    guest::{config::GuestConfig, context::GuestContext, timer::GuestTimers},
    id::{VcpuId, VmId},
};

pub mod runner;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VcpuState {
    Created,
    Runnable,
    Running,
    Blocked(VcpuBlockReason),
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

    pub const fn block_reason(&self) -> VcpuBlockReason {
        if let VcpuState::Blocked(reason) = self {
            *reason
        } else {
            VcpuBlockReason::None
        }
    }
}

pub struct Vcpu {
    id: VcpuId,
    vm_id: VmId,
    config: GuestConfig,
    context: GuestContext,
    state: VcpuState,
    traps: u64,
    timers: GuestTimers,
}

impl Vcpu {
    pub fn new(id: VcpuId, vm_id: VmId, config: GuestConfig) -> Self {
        Self {
            id,
            vm_id,
            config,
            context: GuestContext::from_config(config),
            state: VcpuState::Created,
            traps: 0,
            timers: GuestTimers::new(),
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

    pub const fn context(&self) -> &GuestContext {
        &self.context
    }

    pub const fn context_mut(&mut self) -> &mut GuestContext {
        &mut self.context
    }

    pub const fn state(&self) -> VcpuState {
        self.state
    }

    pub const fn trap_count(&self) -> u64 {
        self.traps
    }

    pub const fn timers(&self) -> &GuestTimers {
        &self.timers
    }

    pub fn timers_mut(&mut self) -> &mut GuestTimers {
        &mut self.timers
    }

    pub const fn exit_reason(&self) -> VcpuExitReason {
        self.state.exit_reason()
    }

    pub const fn block_reason(&self) -> VcpuBlockReason {
        self.state.block_reason()
    }

    pub fn make_runnable(&mut self) {
        if matches!(self.state, VcpuState::Created | VcpuState::Blocked(_)) {
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

    pub fn block(&mut self, reason: VcpuBlockReason) {
        debug_assert_eq!(self.state, VcpuState::Running);
        self.state = VcpuState::Blocked(reason);
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VcpuExitReason {
    None,
    Halted,
    UnhandledTrap,
    MmioError,
    InternalError,
    InstructionAbort,
    UnknownSysReg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VcpuBlockReason {
    None,
    WaitForInterrupt,
    WaitForEvent,
}
