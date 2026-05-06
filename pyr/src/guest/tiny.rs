use core::{arch::global_asm, ptr};

use pyr_arch::addr::{IpaAddr, PhysAddr};

use crate::{guest::launch::enter_el1_guest, stage2::scratch};

global_asm!(include_str!("tiny.S"));

unsafe extern "C" {
    static __tiny_guest_start: u8;
    static __tiny_guest_end: u8;

}

const TINY_GUEST_IPA: u64 = 0x4000_0000;

pub fn load_tiny_guest() -> (IpaAddr, PhysAddr, usize) {
    let src = ptr::addr_of!(__tiny_guest_start);
    let end = ptr::addr_of!(__tiny_guest_end);

    let len = end as usize - src as usize;

    let scratch = scratch::get_mut();

    if len > scratch.guest_ram.len() {
        crate::log!("tiny guest too large: {len}");
        loop {
            core::hint::spin_loop();
        }
    }

    // SAFETY:
    // - src/end delimit the embedded tiny guest payload.
    // - destination is dedicated guest RAM scratch storage.
    // - source and destination do not overlap.
    unsafe {
        ptr::copy_nonoverlapping(src, scratch.guest_ram.as_mut_ptr(), len);
    }

    let host_pa = scratch::guest_ram_base();

    crate::log!("tiny guest loaded: {} bytes", len);
    crate::log!("tiny guest IPA = {:#018x}", TINY_GUEST_IPA);
    crate::log!("tiny guest PA  = {:#018x}", host_pa);

    (IpaAddr::new(TINY_GUEST_IPA), PhysAddr::new(host_pa), len)
}

pub fn enter_tiny_guest() -> ! {
    let stack_top = crate::stage2::scratch::guest_stack_top();

    enter_el1_guest(super::config::GuestConfig::new(TINY_GUEST_IPA, stack_top))
}
