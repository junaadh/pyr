#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::undocumented_unsafe_blocks)]

use pyr_arch::{
    addr::PhysAddr,
    exception::{DataAbortIss, TrapFrame},
    platform::{MmioError, Platform},
};

pub struct QemuVirt;

impl QemuVirt {
    fn emulate_pl011_write(
        frame: &mut TrapFrame,
        iss: DataAbortIss,
    ) -> Result<(), MmioError> {
        if !iss.isv {
            return Err(MmioError::InvalidSyndrome);
        }

        if !iss.wnr {
            return Err(MmioError::ReadFault);
        }

        if iss.sas != 0 {
            return Err(MmioError::UnsupportedAccess);
        }

        let reg = iss.srt as usize;
        let Some(value) = frame.x.get(reg) else {
            return Err(MmioError::InvalidRegister);
        };

        let byte = *value as u8;

        Self::early_putc(byte);

        Ok(())
    }
}

impl Platform for QemuVirt {
    const UART_BASE: PhysAddr = PhysAddr::new(0x0900_0000);

    fn early_init() {}

    fn early_putc(byte: u8) {
        let ptr = Self::UART_BASE.as_u64() as *mut u8;

        // SAFETY: QEMU virt exposes PL011 UART MMIO at physical address 0x0900_0000
        unsafe {
            ptr.write_volatile(byte);
        }
    }

    fn mmio_emulate(
        ipa: pyr_arch::addr::IpaAddr,
        frame: &mut pyr_arch::exception::TrapFrame,
        iss: pyr_arch::exception::DataAbortIss,
    ) -> Result<(), pyr_arch::platform::MmioError> {
        match ipa.as_u64() {
            0x0900_0000 => Self::emulate_pl011_write(frame, iss),
            _ => Err(MmioError::UnknownDevice),
        }
    }
}
