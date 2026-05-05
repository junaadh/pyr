#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::undocumented_unsafe_blocks)]

pub mod addr;
pub mod barrier;
pub mod exception;
pub mod platform;
pub mod sysregs;
