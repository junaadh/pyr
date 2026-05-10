use pyr_arch::exception::TrapFrame;

use crate::{
    fatal::halt,
    runtime::{El2Context, scheduler::SchedulerDecision},
    trap::{TrapOutcome, dispatch},
};

pub struct TrapRunner;

impl TrapRunner {
    pub fn run(frame: &mut TrapFrame) {
        let cx = El2Context::current();

        let outcome = {
            let (vm, vcpu) = cx.runtime_mut().split_mut();

            vcpu.context_mut().sync_from_trap_frame(frame);
            vcpu.record_trap();

            dispatch::handle_trap(vm, vcpu)
        };

        match outcome {
            TrapOutcome::Return => cx
                .runtime_mut()
                .vcpu_mut()
                .context()
                .sync_to_trap_frame(frame),
            TrapOutcome::AdvancePc => {
                let vcpu = cx.runtime_mut().vcpu_mut();
                vcpu.context_mut().advance_pc();
                vcpu.context().sync_to_trap_frame(frame);
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
                        let vcpu = cx.runtime_mut().vcpu_mut();
                        vcpu.make_runnable();
                        vcpu.enter_running();
                        vcpu.context().sync_to_trap_frame(frame);
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
                        let vcpu = cx.runtime_mut().vcpu_mut();
                        vcpu.make_runnable();
                        vcpu.enter_running();
                        vcpu.context().sync_to_trap_frame(frame);
                    }
                    SchedulerDecision::NoRunnableVcpu => halt(),
                }
            }
        }
    }
}
