use pyr_arch::page_table::Installed;

use crate::stage2::Stage2Vm;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VmId(pub u64);

pub struct Vm {
    id: VmId,
    stage2: Stage2Vm<Installed>,
}

impl Vm {
    pub const fn new(id: VmId, stage2: Stage2Vm<Installed>) -> Self {
        Self { id, stage2 }
    }

    pub const fn id(&self) -> VmId {
        self.id
    }

    pub const fn stage2(&self) -> &Stage2Vm<Installed> {
        &self.stage2
    }
}
