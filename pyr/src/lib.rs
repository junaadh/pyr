#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::undocumented_unsafe_blocks)]

pub mod boot;
pub mod console;
pub mod guest;
pub mod hearth;
pub mod mmio;
pub mod stage2;
pub mod trap;

#[cfg(feature = "platform-qemu-virt")]
use pyr_platform_qemu::QemuVirt;

#[cfg(feature = "platform-qemu-virt")]
pub(crate) type ActivePlatform = QemuVirt;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::console::_print(core::format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n")
    };

    ($fmt:expr) => {
        $crate::print!(core::concat!($fmt, "\n"))
    };

    ($fmt:expr, $($arg:tt)*) => {
        $crate::print!(core::concat!($fmt, "\n"), $($arg)*)
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::println!("[debug] {}", core::format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        $crate::println!("[pyr] {}", core::format_args!($($arg)*))
    };
}
