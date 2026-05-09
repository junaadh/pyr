#[cfg(feature = "boot-tiny")]
use pyr_alloc::{
    context::PyrContext,
    guest_ram::GUEST_RAM_MIN_ALIGN,
    traits::{GuestRamAllocator, PageAllocator},
};

#[cfg(feature = "boot-tiny")]
use crate::{
    fatal,
    guest::{self, memory::GuestMemory},
    log,
    stage2::Stage2Vm,
};

#[cfg(feature = "boot-tiny")]
pub fn boot_tiny<A>(cx: &mut PyrContext<A>) -> !
where
    A: PageAllocator + GuestRamAllocator,
{
    let ram = cx
        .alloc_guest_ram(
            GuestMemory::GUEST_RAM_SIZE as u64,
            GUEST_RAM_MIN_ALIGN,
        )
        .unwrap_or_else(|err| {
            fatal!("tiny guest RAM allocation failed: {err:?}")
        });

    let mut stage2 = Stage2Vm::new(cx)
        .unwrap_or_else(|err| fatal!("stage2 init failed: {err:?}"));

    let _image = guest::tiny::load_tiny_guest(&ram);

    GuestMemory::map_region(cx, &mut stage2, GuestMemory::ram_window(&ram))
        .unwrap_or_else(|err| fatal!("tiny stage2 map filed: {err:?}"));

    log!("stage2 root = {:#018x}", stage2.root_raw());

    stage2.install();

    guest::tiny::enter_tiny_guest()
}
