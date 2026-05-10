use pyr_arch::{
    exception::{ExceptionClass, WfxKind},
    sysregs::el2::EsrEl2,
};

use crate::{
    trap::{TrapOutcome, data_abort, hvc},
    vcpu::{Vcpu, VcpuBlockReason, VcpuExitReason},
    vm::Vm,
};

pub fn handle_trap(vm: &mut Vm, vcpu: &mut Vcpu) -> TrapOutcome {
    let esr = EsrEl2::mrs();

    match esr.decode() {
        ExceptionClass::Hvc64 { imm16 } => hvc::handle_hvc64(vm, vcpu, imm16),
        ExceptionClass::DataAbortLower { iss } => {
            data_abort::handle(vm, vcpu, iss)
        }

        ExceptionClass::Wfx { kind } => match kind {
            WfxKind::Wfi => {
                TrapOutcome::Block(VcpuBlockReason::WaitForInterrupt)
            }
            WfxKind::Wfe => TrapOutcome::Block(VcpuBlockReason::WaitForEvent),
        },

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
