use crate::{device::DeviceMap, id::VmId, stage2::Stage2Vm};
use pyr_arch::page_table::Installed;

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

    pub fn devices_mut(&mut self) -> &mut DeviceMap {
        &mut self.devices
    }

    pub fn inject_irq(&mut self, irq: u32) {
        self.devices_mut().inject_irq(irq);
    }
}

impl core::fmt::Debug for Vm {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Vm")
            .field("id", &self.id)
            .field("vmid", &self.stage2.vmid())
            .field("stage2_root", &self.stage2.root_pa())
            .finish_non_exhaustive()
    }
}
