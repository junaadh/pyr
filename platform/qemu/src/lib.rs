#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![allow(clippy::new_without_default)]

pub mod gic;
pub mod irq;
pub mod pl011;

use crate::pl011::Pl011;
use pyr_arch::{addr::PhysAddr, platform::Platform};

pub type InterruptController = irq::QemuGic;

pub struct QemuVirt;

impl QemuVirt {}

impl Platform for QemuVirt {
    const UART_BASE: PhysAddr = PhysAddr::new(Pl011::BASE);

    fn early_init() {}

    fn early_putc(byte: u8) {
        Pl011::emulate_putc(byte);
    }
}

#[macro_export]
macro_rules! qemu {
    ($($args:tt)*) => {
        $crate::pl011::_print(core::format_args!("[pyr-qemu] {}\n", core::format_args!($($args)*)))
    };
}
