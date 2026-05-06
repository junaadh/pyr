use core::arch::global_asm;

use crate::guest::launch::enter_el1_guest;

global_asm!(include_str!("tiny.S"));

unsafe extern "C" {
    fn __tiny_guest_entry() -> !;
}

pub fn enter_tiny_guest() -> ! {
    let entry = __tiny_guest_entry as *const () as u64;

    // SAFETY: We intentionally define a 4096 array in memory and point the top to the base + len
    let stack_top = crate::stage2::scratch::guest_stack_top();

    enter_el1_guest(super::config::GuestConfig {
        entry,
        stack_top,
        x0: 0,
        x1: 0,
        x2: 0,
        x3: 0,
    })
}
