use pyr_arch::{
    exception::{ExceptionClass, TrapFrame},
    sysregs::el2::EsrEl2,
};

use crate::{
    trap::{Resume, data_abort, hvc},
    vcpu::Vcpu,
    vm::Vm,
};

pub fn handle_trap(
    vm: &mut Vm,
    vcpu: &mut Vcpu,
    frame: &mut TrapFrame,
) -> Resume {
    let esr = EsrEl2::mrs();

    match esr.decode() {
        ExceptionClass::Hvc64 { imm16 } => {
            hvc::handle_hvc64(vm, vcpu, frame, imm16)
        }
        ExceptionClass::DataAbortLower { iss } => {
            data_abort::handle(vm, vcpu, frame, iss)
        }

        other => {
            crate::log!(
                "trap: unhandled {:?} {:?}: {other:?}",
                vm.id(),
                vcpu.id()
            );

            Resume::Halt
        }
    }
}
