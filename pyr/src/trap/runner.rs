use pyr_arch::exception::TrapFrame;

use crate::{
    fatal::halt,
    runtime::El2Context,
    trap::{TrapOutcome, dispatch},
};

pub struct TrapRunner;

impl TrapRunner {
    pub fn run(frame: &mut TrapFrame) {
        let cx = El2Context::current();
        let (vm, vcpu) = cx.runtime_mut().split_mut();

        vcpu.context_mut().sync_from_trap_frame(frame);
        vcpu.record_trap();

        match dispatch::handle_trap(vm, vcpu) {
            TrapOutcome::Return => vcpu.context().sync_to_trap_frame(frame),
            TrapOutcome::AdvancePc => {
                vcpu.context_mut().advance_pc();
                vcpu.context().sync_to_trap_frame(frame);
            }
            TrapOutcome::Exit(reason) => {
                vcpu.halt(reason);
                crate::log!(
                    "trap: vcpu.halt {:?} reason={:?} traps={}",
                    vcpu.id(),
                    vcpu.exit_reason(),
                    vcpu.trap_count()
                );
                halt()
            }
        }
    }
}
