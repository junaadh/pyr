use core::cmp::Ordering;

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
    pub fn emulate_abort(
        &self,
        ipa: IpaAddr,
        frame: &mut TrapFrame,
        iss: DataAbortIss,
    ) -> Result<(), MmioError> {
        let raw = ipa.as_u64();

        let region = self.find_region(raw).ok_or(MmioError::UnknownDevice)?;

        let access = MmioAccess::from_abort(ipa, region.base(), frame, iss)?;

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
        let mut regions = Vec::new();
        config
            .mmio
            .into_iter()
            .filter(|_| {
                matches!(
                    config.machine,
                    MachineKind::QemuVirt | MachineKind::GenericArmVirt
                )
            })
            .for_each(|region| {
                push_qemu_virt_devices(
                    &mut regions,
                    region.base.as_u64(),
                    region.len,
                )
            });

        regions.sort_by_key(|region| region.base());

        debug_assert_no_overlaps(&regions);

        Self { regions }
    }

    fn find_region(&self, ipa: u64) -> Option<&DeviceRegion> {
        self.regions
            .binary_search_by(|region| {
                if ipa < region.base() {
                    Ordering::Greater
                } else if ipa >= region.end() {
                    Ordering::Less
                } else {
                    Ordering::Equal
                }
            })
            .ok()
            .and_then(|idx| self.regions.get(idx))
    }
}

fn debug_assert_no_overlaps(regions: &[DeviceRegion]) {
    #[cfg(debug_assertions)]
    for window in regions.windows(2) {
        if let Some(a) = window.first()
            && let Some(b) = window.get(1)
        {
            debug_assert!(
                a.end() <= b.base(),
                "overlapping MMIO regions: {a:?} and {b:?}"
            );
        }
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

    pub const fn end(&self) -> u64 {
        self.base + self.len
    }
}

fn push_qemu_virt_devices(
    regions: &mut Vec<DeviceRegion>,
    base: u64,
    len: u64,
) {
    let end = base + len;

    if base <= 0x0900_0000 && 0x0900_1000 <= end {
        regions.push(DeviceRegion {
            base: 0x0900_0000,
            len: 0x1000,
            kind: DeviceKind::Pl011,
        });
    }

    if base <= 0x0800_0000 && 0x0801_0000 <= end {
        regions.push(DeviceRegion {
            base: 0x0800_0000,
            len: 0x1_0000,
            kind: DeviceKind::Gic,
        });
    }

    if base <= 0x0801_0000 && 0x0802_0000 <= end {
        regions.push(DeviceRegion {
            base: 0x0801_0000,
            len: 0x1_0000,
            kind: DeviceKind::Gic,
        });
    }
}
