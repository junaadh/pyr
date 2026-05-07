use pyr_arch::platform::{
    MmioAccess, MmioAccessKind, MmioDevice, MmioDeviceError, MmioResult,
};

use crate::qemu;

pub struct Gic;

impl Gic {
    pub const DIST_BASE: u64 = 0x0800_0000;
    pub const CPU_BASE: u64 = 0x0801_0000;
    pub const SIZE: u64 = 0x10000;

    const GICD_CTLR: u64 = 0x0000;
    const GICD_TYPER: u64 = 0x0004;
    const GICD_IIDR: u64 = 0x0008;

    const GICC_CTLR: u64 = 0x0000;
    const GICC_PMR: u64 = 0x0004;
    const GICC_BPR: u64 = 0x0008;
    const GICC_IAR: u64 = 0x000c;
    const GICC_EOIR: u64 = 0x0010;
    const GICC_RPR: u64 = 0x0014;
    const GICC_HPPIR: u64 = 0x0018;

    fn is_dist(ipa: u64) -> bool {
        (Self::DIST_BASE..Self::DIST_BASE + Self::SIZE).contains(&ipa)
    }

    fn is_cpu(ipa: u64) -> bool {
        (Self::CPU_BASE..Self::CPU_BASE + Self::SIZE).contains(&ipa)
    }

    pub fn base_for(ipa: u64) -> Option<u64> {
        if Self::is_cpu(ipa) {
            Some(Self::CPU_BASE)
        } else if Self::is_dist(ipa) {
            Some(Self::DIST_BASE)
        } else {
            None
        }
    }
}

impl MmioDevice for Gic {
    fn contains(ipa: u64) -> bool {
        Self::is_dist(ipa) || Self::is_cpu(ipa)
    }

    fn emulate(access: MmioAccess) -> Result<MmioResult, MmioDeviceError> {
        if Self::is_dist(access.ipa) {
            emulate_dist(access)
        } else if Self::is_cpu(access.ipa) {
            emulate_cpu(access)
        } else {
            Err(MmioDeviceError::BadRegister)
        }
    }
}

fn emulate_dist(access: MmioAccess) -> Result<MmioResult, MmioDeviceError> {
    match access.kind {
        MmioAccessKind::Read { .. } => {
            let value = match access.offset {
                Gic::GICD_CTLR => 0,
                Gic::GICD_TYPER => 0x0000_00ff,
                Gic::GICD_IIDR => 0x0102_0143,

                // ISENABLER / ICENABLER / ICPENDR / IPRIORITYR / ITARGETSR / ICFGR.
                0x100..=0x17f
                | 0x180..=0x1ff
                | 0x300..=0x3ff
                | 0x400..=0x7ff
                | 0xc00..=0xcff => 0,

                // ITARGETSR: single CPU target.
                0x800..=0xbff => 0x0101_0101,

                unknown => {
                    qemu!("gicd read unknown offset={unknown:#x}");
                    return Err(MmioDeviceError::BadRegister);
                }
            };

            Ok(MmioResult::Read(value))
        }

        MmioAccessKind::Write { value, .. } => {
            match access.offset {
                Gic::GICD_CTLR
                | 0x100..=0x17f // ISENABLER
                | 0x180..=0x1ff // ICENABLER
                | 0x200..=0x27f // ISPENDR
                | 0x280..=0x2ff // ICPENDR
                | 0x300..=0x37f // ISACTIVER
                | 0x380..=0x3ff // ICACTIVER
                | 0x400..=0x7ff // IPRIORITYR
                | 0x800..=0xbff // ITARGETSR
                | 0xc00..=0xcff // ICFGR
                => {}

                unknown => {
                    qemu!("gicd write unknown offset={unknown:#x} value={value:#x}");
                    return Err(MmioDeviceError::BadRegister);
                }
            }

            Ok(MmioResult::Done)
        }
    }
}

fn emulate_cpu(access: MmioAccess) -> Result<MmioResult, MmioDeviceError> {
    match access.kind {
        MmioAccessKind::Read { .. } => {
            let value = match access.offset {
                Gic::GICC_CTLR => 0,
                Gic::GICC_PMR => 0xff,
                Gic::GICC_BPR => 0,
                Gic::GICC_IAR => 1023,
                Gic::GICC_RPR => 0xff,
                Gic::GICC_HPPIR => 1023,

                // GICC_IIDR-ish probe area Linux may read.
                0x00fc => 0,

                unknown => {
                    qemu!("gicc read unknown offset={unknown:#x}");
                    return Err(MmioDeviceError::BadRegister);
                }
            };

            Ok(MmioResult::Read(value))
        }

        MmioAccessKind::Write { value, .. } => {
            match access.offset {
                Gic::GICC_CTLR
                | Gic::GICC_PMR
                | Gic::GICC_BPR
                | Gic::GICC_EOIR => {}

                unknown => {
                    qemu!(
                        "gicc write unknown offset={unknown:#x} value={value:#x}"
                    );
                    return Err(MmioDeviceError::BadRegister);
                }
            }

            Ok(MmioResult::Done)
        }
    }
}
