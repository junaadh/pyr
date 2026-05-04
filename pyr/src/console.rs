use core::fmt::{self, Write};

use pyr_arch::platform::Platform;

static mut EARLY_PUTC: Option<fn(u8)> = None;

pub fn init<P>()
where
    P: Platform,
{
    // SAFETY: single-core early boot only. No concurrency yet.
    unsafe {
        EARLY_PUTC = Some(P::early_putc);
    }
}

pub struct EarlyConsole;

impl Write for EarlyConsole {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        // SAFETY: single-core early boot only. Initialised before logging
        let putc = unsafe { EARLY_PUTC };

        if let Some(putc) = putc {
            s.bytes().for_each(putc);
        }

        Ok(())
    }
}

pub fn _print(args: fmt::Arguments<'_>) {
    let mut console = EarlyConsole;
    let _ = console.write_fmt(args);
}
