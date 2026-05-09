use pyr_arch::exception::TrapFrame;

use crate::{
    fatal::halt,
    runtime::El2Context,
    trap::{Resume, dispatch},
    vcpu::{VcpuExitReason, VcpuState},
};

pub struct TrapRunner;

impl TrapRunner {
    pub fn run(frame: &mut TrapFrame) {
        let cx = El2Context::current();
        let (vm, vcpu) = cx.runtime_mut().split_mut();

        vcpu.record_trap();

        match dispatch::handle_trap(vm, vcpu, frame) {
            Resume::ReturnToGuest => {}
            Resume::AdvancePcAndReturn => {
                frame.elr_el2 = frame.elr_el2.wrapping_add(4);
            }
            Resume::Halt => {
                if vcpu.state() != VcpuState::Halted {
                    vcpu.stop(VcpuExitReason::InternalError);
                }
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
