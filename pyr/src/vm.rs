use crate::{device::DeviceMap, stage2::Stage2Vm, traits::ID};
use core::fmt;
use pyr_arch::page_table::Installed;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VmId(pub u64);

impl VmId {
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn as_u64(self) -> u64 {
        self.0
    }

    pub fn from_parts(stage2_root_pa: u64, guest_entry: u64) -> Self {
        Self(Self::stable_mix64(stage2_root_pa ^ guest_entry))
    }
}

impl ID for VmId {}

impl fmt::Debug for VmId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let short = (self.0 >> 32) as u32;

        write!(f, "vm:{short:08x}")
    }
}

pub struct Vm {
    id: VmId,
    stage2: Stage2Vm<Installed>,
    devices: DeviceMap,
}

impl Vm {
    pub const fn new(
        id: VmId,
        stage2: Stage2Vm<Installed>,
        devices: DeviceMap,
    ) -> Self {
        Self {
            id,
            stage2,
            devices,
        }
    }

    pub const fn id(&self) -> VmId {
        self.id
    }

    pub const fn stage2(&self) -> &Stage2Vm<Installed> {
        &self.stage2
    }

    pub const fn devices(&self) -> &DeviceMap {
        &self.devices
    }
}
