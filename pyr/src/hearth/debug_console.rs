use super::abi::{FunctionId, HvcCall};
use super::caps::{CapSet, Scope};
use super::error::HearthError;
use crate::vcpu::Vcpu;

pub fn handle(
    call: &HvcCall,
    _vcpu: &mut Vcpu,
    caps: CapSet,
) -> Result<(), HearthError> {
    if !caps.allows(Scope::GuestConsoleWrite) {
        return Err(HearthError::PermissionDenied);
    }

    match call.function {
        FunctionId::Putc => {
            let byte = call.arg0 as u8;
            crate::print!("{}", byte as char);
            Ok(())
        }
        FunctionId::Unknown(id) => Err(HearthError::UnknownFunction(id)),
    }
}
