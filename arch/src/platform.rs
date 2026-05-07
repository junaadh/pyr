use crate::{
    addr::{IpaAddr, PhysAddr},
    exception::{DataAbortIss, TrapFrame},
};

pub trait Platform {
    const UART_BASE: PhysAddr;

    fn early_init();

    fn early_putc(byte: u8);
    fn early_print(s: &str) {
        s.bytes().for_each(Self::early_putc);
    }

    fn mmio_emulate(
        ipa: IpaAddr,
        frame: &mut TrapFrame,
        iss: DataAbortIss,
    ) -> Result<(), MmioError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MmioError {
    UnknownDevice,
    InvalidSyndrome,
    ReadFault,
    WriteFault,
    DeviceError(MmioDeviceError),
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum MmioDeviceError {
    UnsupportedAccess,
    BadRegister,
    BadSourceRegister,
}

pub trait MmioDevice {
    fn contains(ipa: u64) -> bool;

    fn emulate(
        ipa: u64,
        frame: &mut TrapFrame,
        iss: DataAbortIss,
    ) -> Result<(), MmioDeviceError>;
}

pub fn write_back_read_value(
    frame: &mut TrapFrame,
    iss: DataAbortIss,
    value: u64,
) -> Result<(), MmioDeviceError> {
    if !iss.isv {
        return Err(MmioDeviceError::UnsupportedAccess);
    }

    let reg = iss.srt as usize;
    let slot = frame
        .x
        .get_mut(reg)
        .ok_or(MmioDeviceError::BadSourceRegister)?;

    *slot = match iss.sas {
        0 => value & 0xff,
        1 => value & 0xffff,
        2 => value & 0xffff_ffff,
        3 => value,
        _ => return Err(MmioDeviceError::UnsupportedAccess),
    };

    Ok(())
}

pub fn read_guest_register(
    frame: &TrapFrame,
    iss: DataAbortIss,
) -> Result<u64, MmioDeviceError> {
    if !iss.isv {
        return Err(MmioDeviceError::UnsupportedAccess);
    }

    frame
        .x
        .get(iss.srt as usize)
        .copied()
        .ok_or(MmioDeviceError::BadSourceRegister)
}
