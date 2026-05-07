use pyr_arch::{
    exception::{DataAbortIss, TrapFrame},
    platform::{
        MmioDevice, MmioDeviceError, read_guest_register, write_back_read_value,
    },
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
    const _GICC_EOIR: u64 = 0x0010;
    const GICC_RPR: u64 = 0x0014;
    const GICC_HPPIR: u64 = 0x0018;
}

impl MmioDevice for Gic {
    fn contains(ipa: u64) -> bool {
        (Self::DIST_BASE..Self::DIST_BASE + Self::SIZE).contains(&ipa)
            || (Self::CPU_BASE..Self::CPU_BASE + Self::SIZE).contains(&ipa)
    }

    fn emulate(
        ipa: u64,
        frame: &mut TrapFrame,
        iss: DataAbortIss,
    ) -> Result<(), MmioDeviceError> {
        if (Self::DIST_BASE..Self::DIST_BASE + Self::SIZE).contains(&ipa) {
            let offset = ipa - Self::DIST_BASE;

            if iss.wnr {
                let value = read_guest_register(frame, iss)?;
                qemu!("gicd write offset={offset:#x} value={value:#x}");
                Ok(())
            } else {
                let value = match offset {
                    Self::GICD_CTLR => 0,
                    Self::GICD_TYPER => 0x0000_00ff,
                    Self::GICD_IIDR => 0x0102_0143,
                    _ => {
                        qemu!("gicd read unknown offset={offset:#x}");
                        0
                    }
                };

                write_back_read_value(frame, iss, value)
            }
        } else {
            let offset = ipa - Self::CPU_BASE;

            if iss.wnr {
                let value = read_guest_register(frame, iss)?;
                qemu!("gicc write offset={offset:#x} value={value:#x}");
                Ok(())
            } else {
                let value = match offset {
                    Self::GICC_CTLR => 0,
                    Self::GICC_PMR => 0xff,
                    Self::GICC_BPR => 0,
                    Self::GICC_IAR => 1023, // spurious interrupt
                    Self::GICC_RPR => 0xff,
                    Self::GICC_HPPIR => 1023, // spurious interrupt
                    _ => {
                        qemu!("gicc read unknown offset={offset:#x}");
                        0
                    }
                };

                write_back_read_value(frame, iss, value)
            }
        }
    }
}
