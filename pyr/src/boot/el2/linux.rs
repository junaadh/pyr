use crate::{
    fatal,
    guest::{launch::enter_el1_guest, linux::boot::load_linux_boot},
    log,
    stage2::Stage2Vm,
};

#[allow(dead_code)]
pub fn boot_linux(image: &[u8], dtb: &[u8], initrd: Option<&[u8]>) -> ! {
    let boot = load_linux_boot(image, dtb, initrd)
        .unwrap_or_else(|err| fatal!("linux boot load failed: {err:?}"));

    log!(
        "linux dtb IPA     = {:#018x}",
        boot.boot_config().dtb.as_u64()
    );
    log!(
        "linux x0          = {:#018x}",
        boot.boot_config().guest_config().x0
    );
    crate::log!(
        "linux x1          = {:#018x}",
        boot.boot_config().guest_config().x1
    );
    crate::log!(
        "linux x2          = {:#018x}",
        boot.boot_config().guest_config().x2
    );
    crate::log!(
        "linux x3          = {:#018x}",
        boot.boot_config().guest_config().x3
    );

    let mut stage2 = Stage2Vm::new();
    boot.map_into(&mut stage2)
        .unwrap_or_else(|err| fatal!("linux stage2 map failed: {err:?}"));

    log!("stage2 root = {:#018x}", stage2.root_raw());

    let _stage2 = stage2.install();

    enter_el1_guest(boot.boot_config().guest_config())
}
