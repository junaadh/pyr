#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::undocumented_unsafe_blocks)]

use pyr_arch::addr::PhysAddr;

pub struct QemuVirt;

impl QemuVirt {
    pub const UART_BASE: PhysAddr = PhysAddr::new(0x0900_0000);
}
