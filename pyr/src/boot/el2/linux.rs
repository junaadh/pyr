use pyr_alloc::{
    context::PyrContext,
    traits::{GuestRamAllocator, PageAllocator},
};

use crate::{
    fatal,
    guest::{
        launch::run_vcpu,
        linux::{boot::load_linux_boot, boot_config::LinuxBootConfig},
    },
    log,
    runtime::El2Context,
    stage2::Stage2Vm,
    vcpu::{Vcpu, VcpuId},
    vm::{Vm, VmId},
};

#[allow(dead_code)]
pub fn boot_linux<A>(cx: &mut PyrContext<A>, config: LinuxBootConfig<'_>) -> !
where
    A: PageAllocator + GuestRamAllocator,
{
    let image = config.kernel;
    let dtb = config.dtb;
    let initrd = config.initrd;

    let boot = load_linux_boot(cx, image, dtb, initrd)
        .unwrap_or_else(|err| fatal!("linux boot load failed: {err:?}"));

    let mut stage2 = Stage2Vm::new(cx)
        .unwrap_or_else(|err| fatal!("tage2 init failed: {err:?}"));
    boot.map_into(cx, &mut stage2)
        .unwrap_or_else(|err| fatal!("linux stage2 map failed: {err:?}"));

    let stage2 = stage2.install();
    let guest = boot.guest_config();

    let vm_id = VmId::from_parts(stage2.root_pa().as_u64(), guest.entry);
    let vm = Vm::new(vm_id, stage2);
    let vcpu = Vcpu::new(VcpuId::from_parts(vm_id, 0), vm_id, guest);

    log!(
        "stage2: {:?} root={:#018x}",
        vcpu.id(),
        vm.stage2().root_raw()
    );

    let mut cx = El2Context::new(vm, vcpu);

    run_vcpu(&mut cx)
}
