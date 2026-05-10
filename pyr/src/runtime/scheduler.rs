use crate::{
    irq::InterruptEvent,
    vcpu::{Vcpu, VcpuBlockReason},
};

pub struct Scheduler;

impl Scheduler {
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self
    }

    pub fn on_blocked(&mut self, vcpu: &Vcpu) -> SchedulerDecision {
        crate::log!("sched: blocked {:?}", vcpu.id());
        SchedulerDecision::NoRunnableVcpu
    }

    pub fn on_exited(&mut self, vcpu: &Vcpu) -> SchedulerDecision {
        crate::log!("sched: exited {:?}", vcpu.id());
        SchedulerDecision::NoRunnableVcpu
    }

    pub(crate) fn on_interrupt(
        &self,
        vcpu: &mut Vcpu,
        event: InterruptEvent,
    ) -> SchedulerDecision {
        crate::log!("sched: interrupt {event:?}");
        if matches!(
            vcpu.state(),
            crate::vcpu::VcpuState::Blocked(VcpuBlockReason::WaitForInterrupt)
        ) {
            vcpu.make_runnable();
            return SchedulerDecision::ResumeCurrent;
        }

        SchedulerDecision::ResumeCurrent
    }
}

pub enum SchedulerDecision {
    ResumeCurrent,
    NoRunnableVcpu,
}
