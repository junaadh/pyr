use crate::{
    hearth,
    trap::{TrapOutcome, psci::Psci},
    vcpu::Vcpu,
    vm::Vm,
};
use pyr_arch::reg::Gpr;

pub fn handle_hvc64(_vm: &mut Vm, vcpu: &mut Vcpu, imm16: u16) -> TrapOutcome {
    if Psci::is_psci_call(vcpu.context().x(Gpr::X0)) {
        return Psci::handle_call(vcpu);
    }

    hearth::handle_hvc(vcpu, imm16)
}
