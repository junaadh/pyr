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
    UnsupportedAccess,
    InvalidRegister,
    InvalidSyndrome,
    ReadFault,
    WriteFault,
    DeviceError,
}
