use pyr_arch::{exception::ExceptionClass, sysregs::el2::EsrEl2};

use crate::{
    trap::{TrapOutcome, data_abort, hvc},
    vcpu::{Vcpu, VcpuExitReason},
    vm::Vm,
};

pub fn handle_trap(vm: &mut Vm, vcpu: &mut Vcpu) -> TrapOutcome {
    let esr = EsrEl2::mrs();

    match esr.decode() {
        ExceptionClass::Hvc64 { imm16 } => hvc::handle_hvc64(vm, vcpu, imm16),
        ExceptionClass::DataAbortLower { iss } => {
            data_abort::handle(vm, vcpu, iss)
        }

        other => {
            crate::log!(
                "trap: unhandled {:?} {:?}: {other:?}",
                vm.id(),
                vcpu.id()
            );

            TrapOutcome::Exit(VcpuExitReason::UnhandledTrap)
        }
    }
}
