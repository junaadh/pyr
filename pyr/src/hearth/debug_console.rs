use pyr_arch::exception::TrapFrame;

use super::abi::{FunctionId, HvcCall};
use super::caps::{CapSet, Scope};
use super::error::HearthError;

pub fn handle(
    call: &HvcCall,
    _frame: &mut TrapFrame,
    caps: CapSet,
) -> Result<(), HearthError> {
    if !caps.allows(Scope::GuestConsoleWrite) {
        return Err(HearthError::PermissionDenied);
    }

    match call.function {
        FunctionId::Putc => {
            let byte = call.arg0 as u8;

            crate::log!("hearth.debug_console.putc: {}", byte as char);
            crate::print!("{}", byte as char);

            Ok(())
        }
        FunctionId::Unknown(id) => Err(HearthError::UnknownFunction(id)),
    }
}
