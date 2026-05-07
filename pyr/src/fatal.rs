pub fn halt() -> ! {
    loop {
        core::hint::spin_loop();
    }
}

pub fn fatal(reason: &str) -> ! {
    crate::log!("fatal: {reason}");
    halt()
}

#[macro_export]
macro_rules! fatal {
    ($($arg:tt)*) => {{
        $crate::log!("fatal: {}", core::format_args!($($arg)*));
        $crate::fatal::halt()
    }};
}
