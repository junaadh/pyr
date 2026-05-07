use crate::{
    guest::{launch::enter_el1_guest, linux::boot::load_linux_boot},
    log,
    stage2::Stage2Vm,
};

#[allow(dead_code)]
pub fn boot_linux(image: &[u8], dtb: &[u8]) -> ! {
    let boot = load_linux_boot(image, dtb).unwrap_or_else(|err| {
        log!("linux boot load failed: {err:?}");
        loop {
            core::hint::spin_loop();
        }
    });

    log!(
        "linux dtb IPA     = {:#018x}",
        boot.boot_config().dtb.as_u64()
    );
    log!(
        "linux x0          = {:#018x}",
        boot.boot_config().guest_config().x0
    );

    let mut stage2 = Stage2Vm::new();
    boot.map_into(&mut stage2);

    log!("stage2 root = {:#018x}", stage2.root_raw());

    stage2.install();

    enter_el1_guest(boot.boot_config().guest_config())
}
