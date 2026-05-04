#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::undocumented_unsafe_blocks)]

const UART: *mut u8 = 0x0900_0000 as *mut u8;

#[unsafe(no_mangle)]
pub extern "C" fn pyr_entry() -> ! {
    puts("[pyr] booting\n");

    let el = current_el();
    puts("[pyr] CurrentEL = ");
    put_hex(el);
    puts("\n");

    loop {
        core::hint::spin_loop();
    }
}

fn putc(c: u8) {
    // SAFETY: QEMU virt PL011 UART base for early debug output.
    unsafe {
        UART.write_volatile(c);
    }
}

fn puts(s: &str) {
    for b in s.bytes() {
        putc(b);
    }
}

fn current_el() -> u64 {
    let el: u64;

    // SAFETY: Reading CurrentEL is valid at all AArch64 exception levels.
    unsafe {
        core::arch::asm!(
            "mrs {out}, CurrentEL",
            out = out(reg) el,
            options(nomem, nostack)
        );
    }

    el
}

fn put_hex(x: u64) {
    puts("0x");

    for shift in (0..64).step_by(4).rev() {
        let n = ((x >> shift) & 0xf) as u8;
        let c = match n {
            0..=9 => b'0' + n,
            _ => b'a' + (n - 10),
        };
        putc(c);
    }
}
