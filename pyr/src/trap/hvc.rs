use crate::{
    hearth,
    trap::{Resume, psci::Psci},
};
use pyr_arch::exception::TrapFrame;

pub fn handle_hvc64(frame: &mut TrapFrame, imm16: u16) -> Resume {
    if Psci::is_psci_call(frame.x[0]) {
        return Psci::handle_call(frame);
    }

    hearth::handle_hvc(frame, imm16)
}
