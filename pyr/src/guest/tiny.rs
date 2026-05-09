use pyr_alloc::guest_ram::GuestRam;

use crate::guest::{
    config::GuestConfig, launch::enter_el1_guest, memory::GuestMemory,
    region::GuestRegion,
};
use core::{arch::global_asm, ptr};

global_asm!(include_str!("tiny.S"));

unsafe extern "C" {
    static __tiny_guest_start: u8;
    static __tiny_guest_end: u8;

}

pub fn load_tiny_guest(ram: &GuestRam) -> GuestRegion {
    let src = ptr::addr_of!(__tiny_guest_start);
    let end = ptr::addr_of!(__tiny_guest_end);

    let len = end as usize - src as usize;

    // SAFETY: *const src and len are calulated from __tiny_guest_* extern symbols
    let slice = unsafe { core::slice::from_raw_parts(src, len) };

    GuestMemory::load_image(ram, slice).unwrap_or_else(|err| {
        crate::fatal!("tiny guest too large: {len} bytes: {err:?}")
    })
}

pub fn enter_tiny_guest() -> ! {
    enter_el1_guest(GuestConfig::new(
        GuestMemory::KERNEL_LOAD_IPA.as_u64(),
        GuestMemory::stack_top_ipa(),
    ))
}
