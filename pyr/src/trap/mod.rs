mod data_abort;
mod dispatch;
mod hvc;
mod psci;
mod resume;

use crate::{fatal::halt, runtime::El2Context};
use pyr_arch::exception::TrapFrame;
pub use resume::*;

#[unsafe(no_mangle)]
pub extern "C" fn pyr_sync_lower_el64(frame: &mut TrapFrame) {
    let cx = El2Context::current();
    let (vm, vcpu) = cx.split_mut();

    vcpu.record_trap();

    match dispatch::handle_trap(vm, vcpu, frame) {
        Resume::ReturnToGuest => {}
        Resume::AdvancePcAndReturn => {
            frame.elr_el2 = frame.elr_el2.wrapping_add(4);
        }
        Resume::Halt => {
            crate::log!(
                "trap: vcpu.halt {:?} traps={}",
                vcpu.id(),
                vcpu.trap_count()
            );
            vcpu.mark_halted();
            halt()
        }
    }
}
