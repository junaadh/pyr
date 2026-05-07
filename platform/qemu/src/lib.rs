#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::undocumented_unsafe_blocks)]

pub mod gic;
pub mod pl011;

use pyr_arch::{
    addr::PhysAddr,
    platform::{MmioAccess, MmioDevice, MmioError, Platform},
};

use crate::{gic::Gic, pl011::Pl011};

pub struct QemuVirt;

impl QemuVirt {}

impl Platform for QemuVirt {
    const UART_BASE: PhysAddr = PhysAddr::new(Pl011::BASE);

    fn early_init() {}

    fn early_putc(byte: u8) {
        Pl011::emulate_putc(byte);
    }

    fn mmio_emulate(
        ipa: pyr_arch::addr::IpaAddr,
        frame: &mut pyr_arch::exception::TrapFrame,
        iss: pyr_arch::exception::DataAbortIss,
    ) -> Result<(), MmioError> {
        let raw = ipa.as_u64();

        if Pl011::contains(raw) {
            let access = MmioAccess::from_abort(ipa, Pl011::BASE, frame, iss)?;
            let result =
                Pl011::emulate(access).map_err(MmioError::DeviceError)?;
            return access.complete(frame, result);
        }

        if let Some(base) = Gic::base_for(raw) {
            let access = MmioAccess::from_abort(ipa, base, frame, iss)?;
            let result =
                Gic::emulate(access).map_err(MmioError::DeviceError)?;
            return access.complete(frame, result);
        }

        Err(MmioError::UnknownDevice)
    }
}

#[macro_export]
macro_rules! qemu {
    ($($args:tt)*) => {
        $crate::pl011::_print(core::format_args!("[pyr-qemu] {}\n", core::format_args!($($args)*)))
    };
}
