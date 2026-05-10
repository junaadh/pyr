use core::fmt::{self, Write};

use pyr_arch::platform::{
    MmioAccess, MmioAccessKind, MmioDevice, MmioDeviceError, MmioResult,
};

use crate::qemu;

pub struct Pl011;

impl Pl011 {
    pub const BASE: u64 = 0x0900_0000;
    pub const SIZE: u64 = 0x1000;

    const DR: u64 = 0x000;
    const FR: u64 = 0x018;
    const IBRD: u64 = 0x024;
    const FBRD: u64 = 0x028;
    const LCR_H: u64 = 0x02c;
    const CR: u64 = 0x030;
    const IFLS: u64 = 0x034;
    const IMSC: u64 = 0x038;
    const ICR: u64 = 0x044;
    const RSR_ECR: u64 = 0x004;
    const ILPR: u64 = 0x020;
    const RIS: u64 = 0x03c;
    const MIS: u64 = 0x040;
    const DMACR: u64 = 0x048;

    const FR_TXFE: u64 = 1 << 7;
    const FR_RXFE: u64 = 1 << 4;

    pub(crate) fn emulate_putc(byte: u8) {
        let ptr = Self::BASE as *mut u8;

        // SAFETY: QEMU virt exposes host PL011 UART MMIO at physical address 0x0900_0000.
        unsafe {
            ptr.write_volatile(byte);
        }
    }

    pub(crate) fn emulate_puts(s: &str) {
        for byte in s.bytes() {
            if byte == b'\n' {
                Self::emulate_putc(b'\r');
            }

            Self::emulate_putc(byte);
        }
    }

    fn read_id(offset: u64) -> Option<u64> {
        Some(match offset {
            0xfe0 => 0x11,
            0xfe4 => 0x10,
            0xfe8 => 0x14,
            0xfec => 0x00,
            0xff0 => 0x0d,
            0xff4 => 0xf0,
            0xff8 => 0x05,
            0xffc => 0xb1,
            _ => return None,
        })
    }
}

impl MmioDevice for Pl011 {
    fn contains(ipa: u64) -> bool {
        (Self::BASE..Self::BASE + Self::SIZE).contains(&ipa)
    }

    fn emulate(
        &mut self,
        access: MmioAccess,
    ) -> Result<MmioResult, MmioDeviceError> {
        match access.kind {
            MmioAccessKind::Read { .. } => {
                if let Some(value) = Self::read_id(access.offset) {
                    return Ok(MmioResult::Read(value));
                }

                let value = match access.offset {
                    Self::FR => Self::FR_TXFE | Self::FR_RXFE,

                    Self::DR
                    | Self::RSR_ECR
                    | Self::ILPR
                    | Self::IBRD
                    | Self::FBRD
                    | Self::LCR_H
                    | Self::CR
                    | Self::IFLS
                    | Self::IMSC
                    | Self::RIS
                    | Self::MIS
                    | Self::ICR
                    | Self::DMACR => return stub_read_zero(access.offset),

                    unknown => {
                        qemu!("pl011 read unknown offset={unknown:#x}");
                        return Err(MmioDeviceError::BadRegister);
                    }
                };

                Ok(MmioResult::Read(value))
            }

            MmioAccessKind::Write { value, .. } => {
                match access.offset {
                    Self::DR => Self::emulate_putc(value as u8),

                    Self::RSR_ECR
                    | Self::ILPR
                    | Self::IBRD
                    | Self::FBRD
                    | Self::LCR_H
                    | Self::CR
                    | Self::IFLS
                    | Self::IMSC
                    | Self::RIS
                    | Self::MIS
                    | Self::ICR
                    | Self::DMACR => {
                        return stub_ignore_write(access.offset, value);
                    }

                    unknown => {
                        qemu!(
                            "pl011 write unknown offset={unknown:#x} value={value:#x}"
                        );
                        return Err(MmioDeviceError::BadRegister);
                    }
                }

                Ok(MmioResult::Done)
            }
        }
    }
}

fn stub_read_zero(offset: u64) -> Result<MmioResult, MmioDeviceError> {
    #[cfg(not(feature = "trace_stubs"))]
    {
        _ = offset;
    }
    #[cfg(feature = "trace_stubs")]
    qemu!("pl011 stub read-as-zero offset={offset:#x}");
    Ok(MmioResult::Read(0))
}

fn stub_ignore_write(
    offset: u64,
    value: u64,
) -> Result<MmioResult, MmioDeviceError> {
    #[cfg(not(feature = "trace_stubs"))]
    {
        _ = offset;
        _ = value;
    }
    #[cfg(feature = "trace_stubs")]
    qemu!("pl011 stub ignore-write offset={offset:#x} value={value:#x}");
    Ok(MmioResult::Done)
}

impl Write for Pl011 {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        Self::emulate_puts(s);
        Ok(())
    }
}

pub fn _print(args: fmt::Arguments<'_>) {
    let mut pl011 = Pl011;
    let _ = pl011.write_fmt(args);
}
