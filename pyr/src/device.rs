use alloc::vec::Vec;
use pyr_arch::{
    addr::{IpaAddr, PhysAddr},
    boot::info::{BootInfo, MachineKind, MemoryKind},
    exception::{DataAbortIss, TrapFrame},
    platform::{MmioAccess, MmioDevice, MmioError},
};
use pyr_platform_qemu::{gic::Gic, pl011::Pl011};

#[derive(Clone, Copy, Debug)]
pub struct MmioRegion {
    pub base: PhysAddr,
    pub len: u64,
}

pub struct PlatformDeviceConfig {
    machine: MachineKind,
    mmio: Vec<MmioRegion>,
}

impl PlatformDeviceConfig {
    pub fn from_boot_info(boot: &BootInfo<'_>) -> Self {
        let mmio = boot
            .memory()
            .regions_of(MemoryKind::Mmio)
            .map(|region| MmioRegion {
                base: region.start,
                len: region.len,
            })
            .collect();

        Self {
            machine: boot.machine(),
            mmio,
        }
    }

    pub fn into_device_map(self) -> DeviceMap {
        DeviceMap::from_platform_config(self)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceKind {
    Pl011,
    Gic,
    UnknownMmio,
}

#[derive(Clone, Copy, Debug)]
pub struct DeviceRegion {
    base: u64,
    len: u64,
    kind: DeviceKind,
}

pub struct DeviceMap {
    regions: Vec<DeviceRegion>,
}

impl DeviceMap {
    pub fn from_boot_info(boot: &BootInfo<'_>) -> Self {
        let mut regions = Vec::new();

        for region in boot.memory().regions_of(MemoryKind::Mmio) {
            let kind = classify_mmio_region(
                boot.machine(),
                region.start.as_u64(),
                region.len,
            );

            regions.push(DeviceRegion {
                base: region.start.as_u64(),
                len: region.len,
                kind,
            });
        }

        Self { regions }
    }

    pub fn emulate_abort(
        &self,
        ipa: IpaAddr,
        frame: &mut TrapFrame,
        iss: DataAbortIss,
    ) -> Result<(), MmioError> {
        let raw = ipa.as_u64();

        let region = self
            .regions
            .iter()
            .find(|region| region.contains(raw))
            .ok_or(MmioError::UnknownDevice)?;

        let access_base = match region.kind {
            DeviceKind::Pl011 => Pl011::BASE,
            DeviceKind::Gic => {
                Gic::base_for(raw).ok_or(MmioError::UnknownDevice)?
            }
            DeviceKind::UnknownMmio => return Err(MmioError::UnknownDevice),
        };

        let access = MmioAccess::from_abort(ipa, access_base, frame, iss)?;

        let result = match region.kind {
            DeviceKind::Pl011 => {
                Pl011::emulate(access).map_err(MmioError::DeviceError)?
            }

            DeviceKind::Gic => {
                Gic::emulate(access).map_err(MmioError::DeviceError)?
            }

            DeviceKind::UnknownMmio => {
                return Err(MmioError::UnknownDevice);
            }
        };

        access.complete(frame, result)
    }

    pub fn from_platform_config(config: PlatformDeviceConfig) -> Self {
        let regions = config
            .mmio
            .into_iter()
            .map(|region| DeviceRegion {
                base: region.base.as_u64(),
                len: region.len,
                kind: classify_mmio_region(
                    config.machine,
                    region.base.as_u64(),
                    region.len,
                ),
            })
            .collect();

        Self { regions }
    }
}

impl DeviceRegion {
    pub const fn contains(&self, ipa: u64) -> bool {
        ipa >= self.base && ipa < self.base + self.len
    }

    pub const fn base(&self) -> u64 {
        self.base
    }

    pub const fn len(&self) -> u64 {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub const fn kind(&self) -> DeviceKind {
        self.kind
    }
}

fn classify_mmio_region(
    machine: MachineKind,
    base: u64,
    _len: u64,
) -> DeviceKind {
    match machine {
        MachineKind::QemuVirt | MachineKind::GenericArmVirt => match base {
            0x0900_0000 => DeviceKind::Pl011,
            0x0800_0000 => DeviceKind::Gic,
            _ => DeviceKind::UnknownMmio,
        },

        _ => DeviceKind::UnknownMmio,
    }
}
