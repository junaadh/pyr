#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::undocumented_unsafe_blocks)]

use pyr_arch::{addr::PhysAddr, platform::Platform};

pub struct QemuVirt;

impl QemuVirt {
    pub const UART_BASE: PhysAddr = PhysAddr::new(0x0900_0000);
}

impl Platform for QemuVirt {
    fn early_init() {}

    fn early_putc(byte: u8) {
        let ptr = Self::UART_BASE.as_u64() as *mut u8;

        // SAFETY: QEMU virt exposes PL011 UART MMIO at physical address 0x0900_0000
        unsafe {
            ptr.write_volatile(byte);
        }
    }
}
