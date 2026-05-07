use core::fmt::{self, Write};

use pyr_arch::{
    exception::{DataAbortIss, TrapFrame},
    platform::{
        MmioDevice, MmioDeviceError, read_guest_register, write_back_read_value,
    },
};

use crate::qemu;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Pl011Error {
    UnsupportedAccess,
    BadRegister,
    BadSourceRegister,
}

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
    const IMSC: u64 = 0x038;
    const ICR: u64 = 0x044;

    const FR_TXFE: u64 = 1 << 7;
    const FR_RXFE: u64 = 1 << 4;

    pub(crate) fn emulate_putc(byte: u8) {
        let ptr = Self::BASE as *mut u8;

        // SAFETY: QEMU virt exposes PL011 UART MMIO at physical address 0x0900_0000
        unsafe {
            ptr.write_volatile(byte);
        }
    }

    pub(crate) fn emulate_puts(str: &str) {
        for byte in str.bytes() {
            if byte == b'\n' {
                Self::emulate_putc(b'\r');
            }

            Self::emulate_putc(byte);
        }
    }
}

impl MmioDevice for Pl011 {
    fn contains(ipa: u64) -> bool {
        (Self::BASE..Self::BASE + Self::SIZE).contains(&ipa)
    }

    fn emulate(
        ipa: u64,
        frame: &mut TrapFrame,
        iss: DataAbortIss,
    ) -> Result<(), MmioDeviceError> {
        let offset = ipa - Self::BASE;

        if iss.wnr {
            let value = read_guest_register(frame, iss)?;

            match offset {
                Self::DR => Self::emulate_putc(value as u8),
                Self::IBRD => qemu!("pl011 write IBRD"),
                Self::FBRD => qemu!("pl011 write FBRD"),
                Self::LCR_H => qemu!("pl011 write LCR_H"),
                Self::CR => qemu!("pl011 write CR"),
                Self::IMSC => qemu!("pl011 write IMSC"),
                Self::ICR => qemu!("pl011 write ICR"),
                unknown => qemu!("pl011 write unknown offset={unknown:#x}"),
            }

            Ok(())
        } else {
            let value = match offset {
                Self::FR => Self::FR_TXFE | Self::FR_RXFE,
                Self::DR
                | Self::IBRD
                | Self::FBRD
                | Self::LCR_H
                | Self::CR
                | Self::IMSC
                | Self::ICR => 0,
                unknown => {
                    qemu!("pl011 read unknown offset={unknown:#x}");
                    0
                }
            };

            write_back_read_value(frame, iss, value)
        }
    }
}

impl Write for Pl011 {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        Self::emulate_puts(s);

        Ok(())
    }
}

pub fn _print(args: fmt::Arguments<'_>) {
    let mut pl011 = Pl011;
    let _ = pl011.write_fmt(args);
}
