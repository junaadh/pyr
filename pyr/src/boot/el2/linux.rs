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
    boot.map_into(&mut stage2).unwrap_or_else(|err| {
        log!("linux stage2 map failed: {err:?}");

        loop {
            core::hint::spin_loop();
        }
    });

    log!("stage2 root = {:#018x}", stage2.root_raw());

    let _stage2 = stage2.install();
    // let mut stage2 = stage2.install();
    // match stage2.dump_mapping(IpaAddr::new(0x417c_89a2)) {
    //     Ok(dump) => {
    //         crate::log!("walk ipa = {:#018x}", dump.ipa.as_u64());
    //         crate::log!("L1[{:#x}] = {:#018x}", dump.l1_index, dump.l1_desc);
    //         crate::log!("L2[{:#x}] = {:#018x}", dump.l2_index, dump.l2_desc);
    //         crate::log!("L3[{:#x}] = {:?}", dump.l3_index, dump.l3_desc);
    //     }
    //     Err(err) => crate::log!("stage2 walk failed: {err:?}"),
    // }

    enter_el1_guest(boot.boot_config().guest_config())
}
