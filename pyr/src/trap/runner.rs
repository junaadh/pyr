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

        vcpu.record_trap();

        match dispatch::handle_trap(vm, vcpu, frame) {
            TrapOutcome::Return => {}
            TrapOutcome::AdvancePc => {
                vcpu.advance_pc(frame);
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
