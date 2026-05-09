use crate::{trap::Resume, vcpu::Vcpu, vm::Vm};
use pyr_arch::{
    addr::IpaAddr,
    exception::{DataAbortIss, TrapFrame},
    sysregs::el2::{FarEl2, HpfarEl2},
};

pub fn handle(
    vm: &mut Vm,
    vcpu: &mut Vcpu,
    frame: &mut TrapFrame,
    iss: DataAbortIss,
) -> Resume {
    let far = FarEl2::mrs();
    let hpfar = HpfarEl2::mrs();
    let ipa = hpfar.ipa_base().as_u64() | (far.raw() & 0xfff);

    match vm.devices().emulate_abort(IpaAddr::new(ipa), frame, iss) {
        Ok(()) => Resume::AdvancePcAndReturn,

        Err(err) => {
            crate::log!(
                "mmio: error {:?} {:?}: {err:?} @ {ipa:#018x}",
                vm.id(),
                vcpu.id()
            );

            Resume::Halt
        }
    }
}
