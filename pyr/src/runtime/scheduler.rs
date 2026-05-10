use crate::vcpu::Vcpu;

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
}

pub enum SchedulerDecision {
    ResumeCurrent,
    NoRunnableVcpu,
}
