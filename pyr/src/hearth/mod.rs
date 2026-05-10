pub mod abi;
pub mod caps;
pub mod debug_console;
pub mod error;

pub use abi::*;
pub use caps::*;
use pyr_arch::reg::Gpr;

use crate::{hearth::error::HearthError, trap::TrapOutcome, vcpu::Vcpu};

pub fn handle_hvc(vcpu: &mut Vcpu, imm16: u16) -> TrapOutcome {
    let call = HvcCall::from_vcpu(vcpu, imm16);
    let caps = CapSet::debug_guest();

    match dispatch(&call, vcpu, caps) {
        Ok(()) => {
            vcpu.context_mut().write_x(Gpr::X0, 0);
            TrapOutcome::Return
        }
        Err(err) => {
            crate::log!("hearth.error: {err:?}");
            vcpu.context_mut().write_x(Gpr::X0, err.code());
            TrapOutcome::Exit(crate::vcpu::VcpuExitReason::UnhandledTrap)
        }
    }
}

fn dispatch(
    call: &HvcCall,
    vcpu: &mut Vcpu,
    caps: CapSet,
) -> Result<(), HearthError> {
    match call.extension {
        ExtensionId::DebugConsole => debug_console::handle(call, vcpu, caps),
        ExtensionId::Unknown(id) => Err(HearthError::UnknownExtension(id)),
    }
}
