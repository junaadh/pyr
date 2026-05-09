#[cfg(feature = "boot-tiny")]
use pyr_alloc::{
    context::PyrContext,
    guest_ram::GUEST_RAM_MIN_ALIGN,
    traits::{GuestRamAllocator, PageAllocator},
};

#[cfg(feature = "boot-tiny")]
use crate::{
    fatal,
    guest::{self, config::GuestConfig, launch::run_vcpu, memory::GuestMemory},
    log,
    stage2::Stage2Vm,
    vcpu::{Vcpu, VcpuId},
    vm::{Vm, VmId},
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

    log!(
        "mem: guest_ram base={:#x} size={}",
        ram.base().as_u64(),
        ram.size()
    );

    let mut stage2 = Stage2Vm::new(cx)
        .unwrap_or_else(|err| fatal!("stage2 init failed: {err:?}"));

    let image = guest::tiny::load_tiny_guest(&ram);

    log!(
        "tiny: image ipa={:#x} pa={:#x} size={}",
        image.ipa().as_u64(),
        image.pa().as_u64(),
        image.size()
    );

    GuestMemory::map_region(cx, &mut stage2, GuestMemory::ram_window(&ram))
        .unwrap_or_else(|err| fatal!("tiny stage2 map failed: {err:?}"));

    log!("stage2: root={:#018x}", stage2.root_raw());

    let stage2 = stage2.install();

    let guest_config = GuestConfig::new(
        GuestMemory::KERNEL_LOAD_IPA.as_u64(),
        GuestMemory::stack_top_ipa(),
    );

    let mut vm = Vm::new(VmId(0), stage2);
    let mut vcpu = Vcpu::new(VcpuId(0), vm.id(), guest_config);

    run_vcpu(&mut vm, &mut vcpu);
}
