use crate::{
    fatal::halt,
    guest::timer,
    irq::{InterruptEvent, InterruptSource, IrqNumber},
    runtime::{El2Context, scheduler::SchedulerDecision},
    trap::{TrapOutcome, dispatch, interrupt::InterruptKind},
};
use pyr_arch::{
    exception::TrapFrame, platform::PhysicalInterruptController,
    sysregs::el2::HcrEl2,
};
#[cfg(feature = "platform-qemu-virt")]
use pyr_platform_qemu::InterruptController as ActiveInterruptController;

pub struct TrapRunner;

impl TrapRunner {
    pub fn run_sync(frame: &mut TrapFrame) {
        let cx = El2Context::current();

        let outcome = {
            let (vm, vcpu) = cx.runtime_mut().split_mut();

            vcpu.context_mut().sync_from_trap_frame(frame);
            vcpu.record_trap();

            dispatch::handle_trap(vm, vcpu)
        };

        Self::apply_outcome(cx, frame, outcome);
    }

    pub fn run_interrupt(frame: &mut TrapFrame, kind: InterruptKind) {
        match kind {
            InterruptKind::Irq => Self::run_irq(frame),
            InterruptKind::Fiq => Self::run_fiq(frame),
        }
    }

    fn apply_outcome(
        cx: &mut El2Context,
        frame: &mut TrapFrame,
        outcome: TrapOutcome,
    ) {
        match outcome {
            TrapOutcome::Return => prepare_guest_return(cx, frame),
            TrapOutcome::AdvancePc => {
                cx.runtime_mut().vcpu_mut().context_mut().advance_pc();
                prepare_guest_return(cx, frame);
            }

            TrapOutcome::Block(reason) => {
                let id = {
                    let vcpu = cx.runtime_mut().vcpu_mut();
                    vcpu.block(reason);
                    vcpu.id()
                };
                let descision = cx.on_vcpu_blocked(id);
                match descision {
                    SchedulerDecision::ResumeCurrent => {
                        {
                            let vcpu = cx.runtime_mut().vcpu_mut();
                            vcpu.make_runnable();
                            vcpu.enter_running();
                        }
                        prepare_guest_return(cx, frame);
                    }
                    SchedulerDecision::NoRunnableVcpu => halt(),
                }
            }

            TrapOutcome::Exit(reason) => {
                let id = {
                    let vcpu = cx.runtime_mut().vcpu_mut();
                    vcpu.halt(reason);
                    vcpu.id()
                };
                let descision = cx.on_vcpu_exited(id);
                match descision {
                    SchedulerDecision::ResumeCurrent => {
                        {
                            let vcpu = cx.runtime_mut().vcpu_mut();
                            vcpu.make_runnable();
                            vcpu.enter_running();
                        }
                        prepare_guest_return(cx, frame);
                    }
                    SchedulerDecision::NoRunnableVcpu => halt(),
                }
            }
        }
    }

    pub fn run_irq(frame: &mut TrapFrame) {
        let cx = El2Context::current();

        {
            let vcpu = cx.runtime_mut().vcpu_mut();
            vcpu.context_mut().sync_from_trap_frame(frame);
            vcpu.record_trap();
        }

        let irq = ActiveInterruptController::acknowledge();

        if !ActiveInterruptController::is_spurious(irq) {
            let source =
                InterruptSource::from_physical_irq(IrqNumber::new(irq.0));

            if let Some(irq) = source.guest_irq() {
                cx.runtime_mut().vm_mut().inject_irq(irq);
            }

            ActiveInterruptController::complete(irq);
        }

        let decision =
            cx.on_vcpu_interrupt(InterruptEvent::Irq(IrqNumber::new(irq.0)));

        match decision {
            SchedulerDecision::ResumeCurrent => {
                let vcpu = cx.runtime_mut().vcpu_mut();

                if !vcpu.is_running() {
                    vcpu.make_runnable();
                    vcpu.enter_running();
                }

                prepare_guest_return(cx, frame);
            }
            SchedulerDecision::NoRunnableVcpu => halt(),
        }
    }

    fn run_fiq(frame: &mut TrapFrame) {
        let cx = El2Context::current();

        {
            let vcpu = cx.runtime_mut().vcpu_mut();
            vcpu.context_mut().sync_from_trap_frame(frame);
            vcpu.record_trap();
        }

        let decision = cx.on_vcpu_interrupt(InterruptEvent::Fiq);

        match decision {
            SchedulerDecision::ResumeCurrent => {
                let vcpu = cx.runtime_mut().vcpu_mut();

                if !vcpu.is_running() {
                    vcpu.make_runnable();
                    vcpu.enter_running();
                }

                prepare_guest_return(cx, frame);
            }
            SchedulerDecision::NoRunnableVcpu => halt(),
        }
    }
}

fn prepare_guest_return(cx: &mut El2Context, frame: &mut TrapFrame) {
    {
        let (vm, vcpu) = cx.runtime_mut().split_mut();
        timer::evaluate_guest_timers(vm, vcpu);
    }

    let pending_irq = cx.runtime().vm().devices().has_pending_irq();

    if pending_irq {
        HcrEl2::mrs().with_vi().msr();
    } else {
        HcrEl2::mrs().without_vi().msr();
    }

    cx.runtime_mut()
        .vcpu_mut()
        .context()
        .sync_to_trap_frame(frame);
}
