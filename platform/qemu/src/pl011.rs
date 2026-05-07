use core::fmt::{self, Write};

use pyr_arch::exception::{DataAbortIss, TrapFrame};

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

    pub fn contains(ipa: u64) -> bool {
        (Self::BASE..Self::BASE + Self::SIZE).contains(&ipa)
    }

    pub fn emulate(
        ipa: u64,
        frame: &mut TrapFrame,
        iss: DataAbortIss,
    ) -> Result<(), Pl011Error> {
        let offset = ipa - Self::BASE;

        if iss.wnr {
            Self::write(offset, frame, iss)
        } else {
            Self::read(offset, frame, iss)
        }
    }

    fn read(
        offset: u64,
        frame: &mut TrapFrame,
        iss: DataAbortIss,
    ) -> Result<(), Pl011Error> {
        if !iss.isv {
            return Err(Pl011Error::UnsupportedAccess);
        }

        let value = match offset {
            Self::FR => Self::FR_TXFE | Self::FR_RXFE,

            // Linux may probe/read these. Returning zero is fine for now.
            Self::DR
            | Self::IBRD
            | Self::FBRD
            | Self::LCR_H
            | Self::CR
            | Self::IMSC
            | Self::ICR => 0,

            _ => 0,
        };

        Self::write_back_read_value(frame, iss, value)
    }

    fn write(
        offset: u64,
        frame: &mut TrapFrame,
        iss: DataAbortIss,
    ) -> Result<(), Pl011Error> {
        if !iss.isv {
            return Err(Pl011Error::UnsupportedAccess);
        }

        let reg = iss.srt as usize;
        let value = frame
            .x
            .get(reg)
            .copied()
            .ok_or(Pl011Error::BadSourceRegister)?;

        match offset {
            Self::DR => {
                Self::emulate_putc(value as u8);
            }

            Self::IBRD => qemu!("pl1011 read IBRD"),
            Self::FBRD => qemu!("pl1011 read FBRD"),
            Self::LCR_H => qemu!("pl1011 read LCR_H"),
            Self::CR => qemu!("pl1011 read CR"),
            Self::IMSC => qemu!("pl1011 read IMSC"),
            Self::ICR => qemu!("pl011 read ICR"),

            unknown => {
                // Unknown PL011 offsets are ignored for bring-up.
                qemu!("pl011 access offset={unknown:#x} wnr={}", iss.wnr);
            }
        }

        Ok(())
    }

    fn write_back_read_value(
        frame: &mut TrapFrame,
        iss: DataAbortIss,
        value: u64,
    ) -> Result<(), Pl011Error> {
        let reg = iss.srt as usize;
        let slot = frame.x.get_mut(reg).ok_or(Pl011Error::BadSourceRegister)?;

        *slot = match iss.sas {
            0 => value & 0xff,
            1 => value & 0xffff,
            2 => value & 0xffff_ffff,
            3 => value,
            _ => return Err(Pl011Error::UnsupportedAccess),
        };

        Ok(())
    }

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
