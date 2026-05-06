use crate::ActivePlatform;
use core::fmt::{self, Write};
use pyr_arch::platform::Platform;

pub struct Console;

impl Console {
    pub fn putc(c: char) {
        ActivePlatform::early_putc(c as u8);
    }

    pub fn puts(s: &str) {
        ActivePlatform::early_print(s);
    }
}

impl Write for Console {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        Self::puts(s);

        Ok(())
    }
}

pub fn _print(args: fmt::Arguments<'_>) {
    let mut console = Console;
    let _ = console.write_fmt(args);
}
