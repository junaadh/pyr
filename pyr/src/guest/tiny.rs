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

pub fn load_tiny_guest() -> GuestRegion {
    let src = ptr::addr_of!(__tiny_guest_start);
    let end = ptr::addr_of!(__tiny_guest_end);

    let len = end as usize - src as usize;

    let region = GuestMemory::load_image(src, len).unwrap_or_else(|err| {
        crate::log!("tiny guest too large: {len} bytes: {err:?}");
        loop {
            core::hint::spin_loop();
        }
    });

    crate::log!("tiny guest loaded: {} bytes", len);
    crate::log!("tiny guest IPA = {:#018x}", region.ipa().as_u64());
    crate::log!("tiny guest PA  = {:#018x}", region.pa().as_u64());

    region
}

pub fn enter_tiny_guest() -> ! {
    enter_el1_guest(GuestConfig::new(
        GuestMemory::ENTRY_IPA.as_u64(),
        GuestMemory::stack_top_ipa(),
    ))
}
