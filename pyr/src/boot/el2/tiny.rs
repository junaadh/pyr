#[cfg(feature = "boot-tiny")]
use pyr_alloc::{context::PyrContext, traits::PageAllocator};

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
    A: PageAllocator,
{
    let mut stage2 = Stage2Vm::new(cx)
        .unwrap_or_else(|err| fatal!("stage2 init failed: {err:?}"));

    let _image = guest::tiny::load_tiny_guest();

    GuestMemory::map_region(cx, &mut stage2, GuestMemory::ram_window())
        .unwrap_or_else(|err| fatal!("tiny stage2 map filed: {err:?}"));

    log!("stage2 root = {:#018x}", stage2.root_raw());

    stage2.install();

    guest::tiny::enter_tiny_guest()
}
