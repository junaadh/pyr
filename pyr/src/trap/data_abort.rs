use crate::{
    trap::TrapOutcome,
    vcpu::{Vcpu, VcpuExitReason},
    vm::Vm,
};
use pyr_arch::{
    addr::IpaAddr,
    exception::{DataAbortIss, TrapFrame},
    sysregs::el2::{FarEl2, HpfarEl2},
};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum DataAbortKind {
    AddressSizeFault,
    TranslationFault,
    AccessFlagFault,
    PermissionFault,
    AlignmentFault,
    ExternalAbort,
    Unknown(u8),
}

impl DataAbortKind {
    pub const fn from_dfsc(dfsc: u8) -> Self {
        match dfsc {
            0x00..=0x03 => Self::AddressSizeFault,
            0x04..=0x07 => Self::TranslationFault,
            0x08..=0x0b => Self::AccessFlagFault,
            0x0c..=0x0f => Self::PermissionFault,
            0x10 => Self::ExternalAbort,
            0x21 => Self::AlignmentFault,
            other => Self::Unknown(other),
        }
    }

    pub const fn is_mmio_candidate(self) -> bool {
        matches!(
            self,
            Self::TranslationFault
                | Self::AccessFlagFault
                | Self::PermissionFault
        )
    }
}

pub fn handle(
    vm: &mut Vm,
    vcpu: &mut Vcpu,
    frame: &mut TrapFrame,
    iss: DataAbortIss,
) -> TrapOutcome {
    let kind = DataAbortKind::from_dfsc(iss.dfsc);
    let far = FarEl2::mrs();

    if !kind.is_mmio_candidate() {
        crate::log!(
            "trap: data abort {:?} {:?}: kind={kind:?} far={:#018x}",
            vm.id(),
            vcpu.id(),
            far.raw()
        );

        return TrapOutcome::Exit(VcpuExitReason::UnhandledTrap);
    }

    let hpfar = HpfarEl2::mrs();
    let ipa = hpfar.ipa_base().as_u64() | (far.raw() & 0xfff);

    match vm.devices().emulate_abort(IpaAddr::new(ipa), frame, iss) {
        Ok(()) => TrapOutcome::AdvancePc,

        Err(err) => {
            crate::log!(
                "mmio: error {:?} {:?}: {err:?} @ {ipa:#018x}",
                vm.id(),
                vcpu.id()
            );

            TrapOutcome::Exit(VcpuExitReason::MmioError)
        }
    }
}
