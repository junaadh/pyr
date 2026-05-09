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

    log!("stage2: root={:#018x}", stage2.root_raw());

    let stage2 = stage2.install();

    let mut vm = Vm::new(VmId(0), stage2);
    let mut vcpu = Vcpu::new(VcpuId(0), vm.id(), boot.guest_config());

    run_vcpu(&mut vm, &mut vcpu)
}
