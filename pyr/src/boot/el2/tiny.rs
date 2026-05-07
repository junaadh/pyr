use crate::{
    guest::{self, memory::GuestMemory},
    log,
    stage2::Stage2Vm,
};

pub fn boot_tiny() -> ! {
    let mut stage2 = Stage2Vm::new();

    let _image = guest::tiny::load_tiny_guest();

    let _dtb = guest::linux::dtb::load_dtb_blob(include_bytes!(
        "../../../assets/qemu-virt.dtb"
    ))
    .unwrap_or_else(|err| {
        crate::log!("tiny dtb load failed: {err:?}");

        loop {
            core::hint::spin_loop();
        }
    });

    GuestMemory::map_region(&mut stage2, GuestMemory::ram_window());

    log!("stage2 root = {:#018x}", stage2.root_raw());

    stage2.install();

    guest::tiny::enter_tiny_guest()
}
