#[cfg(feature = "boot-tiny")]
use pyr_alloc::{
    guest_ram::GUEST_RAM_MIN_ALIGN,
    traits::{GuestRamAllocator, PageAllocator},
};

#[cfg(feature = "boot-tiny")]
use crate::{
    context::HypervisorContext,
    device::PlatformDeviceConfig,
    fatal,
    guest::{self, config::GuestConfig, memory::GuestMemory},
    id::{VcpuId, VmId},
    log,
    runtime::El2Context,
    stage2::Stage2Vm,
    vcpu::{Vcpu, runner::VcpuRunner},
    vm::Vm,
};

#[cfg(feature = "boot-tiny")]
pub fn boot_tiny<A>(
    cx: &mut HypervisorContext<A>,
    devices: PlatformDeviceConfig,
) -> !
where
    A: PageAllocator + GuestRamAllocator,
{
    let ram = cx
        .mem
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

    let stage2 = stage2.install();

    let guest_config = GuestConfig::new(
        GuestMemory::KERNEL_LOAD_IPA.as_u64(),
        GuestMemory::stack_top_ipa(),
    );

    let device_map = devices.into_device_map();

    let vm_id = VmId::from_parts(stage2.root_raw(), guest_config.entry);
    let vm = Vm::new(vm_id, stage2, device_map);
    let vcpu = Vcpu::new(VcpuId::from_parts(vm_id, 0), vm_id, guest_config);

    log!(
        "stage2: {:?} root={:#018x}",
        vcpu.id(),
        vm.stage2().root_raw()
    );

    let mut cx = El2Context::from_vm(vm, vcpu);

    VcpuRunner::run(&mut cx)
}
