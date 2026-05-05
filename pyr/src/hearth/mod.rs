pub mod abi;
pub mod caps;
pub mod debug_console;
pub mod error;

pub use abi::*;
pub use caps::*;
use pyr_arch::exception::TrapFrame;

use crate::{hearth::error::HearthError, trap::Resume};

pub fn handle_hvc(frame: &mut TrapFrame, imm16: u16) -> Resume {
    let call = HvcCall::from_frame(frame, imm16);
    let caps = CapSet::debug_guest();

    match dispatch(&call, frame, caps) {
        Ok(()) => {
            frame.x[0] = 0;
            Resume::Halt
        }
        Err(err) => {
            crate::log!("hearth.error: {err:?}");
            frame.x[0] = err.code();
            Resume::Halt
        }
    }
}

fn dispatch(
    call: &HvcCall,
    frame: &mut TrapFrame,
    caps: CapSet,
) -> Result<(), HearthError> {
    match call.extension {
        ExtensionId::DebugConsole => debug_console::handle(call, frame, caps),
        ExtensionId::Unknown(id) => Err(HearthError::UnknownExtension(id)),
    }
}
