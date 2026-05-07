use crate::{
    guest::{self, memory::GuestMemory},
    log,
    stage2::Stage2Vm,
};

pub fn boot_tiny() -> ! {
    let mut stage2 = Stage2Vm::new();

    let _image = guest::tiny::load_tiny_guest();

    GuestMemory::map_region(&mut stage2, GuestMemory::ram_window());

    log!("stage2 root = {:#018x}", stage2.root_raw());

    stage2.install();

    guest::tiny::enter_tiny_guest()
}
