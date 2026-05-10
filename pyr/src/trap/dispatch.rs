use crate::{
    trap::{TrapOutcome, data_abort, hvc, sysreg},
    vcpu::{Vcpu, VcpuBlockReason, VcpuExitReason},
    vm::Vm,
};
use pyr_arch::{
    exception::{ExceptionClass, WfxKind},
    sysregs::el2::EsrEl2,
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

        ExceptionClass::InstructionAbortLower { iss } => {
            crate::log!(
                "trap: instruction_abort {:?} {:?}: iss={iss:#x}",
                vm.id(),
                vcpu.id(),
            );

            TrapOutcome::Exit(VcpuExitReason::InstructionAbort)
        }

        ExceptionClass::SysregTrap { iss } => sysreg::handle(vm, vcpu, iss),
        ExceptionClass::Smc64 { imm16 } => {
            crate::log!(
                "trap: unsupported_smc {:?} {:?}: imm={imm16:#x}",
                vm.id(),
                vcpu.id(),
            );

            TrapOutcome::Exit(VcpuExitReason::UnhandledTrap)
        }

        ExceptionClass::Unknown { ec, iss } => {
            crate::log!(
                "trap: unknown {:?} {:?}: ec={ec:#x} iss={iss:#x}",
                vm.id(),
                vcpu.id(),
            );

            TrapOutcome::Exit(VcpuExitReason::UnhandledTrap)
        }
    }
}
