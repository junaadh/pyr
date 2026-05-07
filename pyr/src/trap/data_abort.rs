use pyr_arch::{
    exception::{DataAbortIss, TrapFrame},
    sysregs::el2::FarEl2,
};

use crate::{mmio, trap::Resume};

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

    pub const fn is_stage2_mmio_candidate(self) -> bool {
        matches!(
            self,
            Self::TranslationFault
                | Self::AccessFlagFault
                | Self::PermissionFault
        )
    }
}

pub fn handle(frame: &mut TrapFrame, iss: DataAbortIss) -> Resume {
    let kind = DataAbortKind::from_dfsc(iss.dfsc);
    let far = FarEl2::mrs();

    if !kind.is_stage2_mmio_candidate() {
        crate::log!("guest data abort: {kind:?}");
        crate::log!("FAR_EL2 = {:#018x}", far.raw());
        return Resume::Halt;
    }

    mmio::handle_data_abort(frame, iss)
}
