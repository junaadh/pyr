use pyr_arch::{
    addr::{IpaAddr, PhysAddr},
    page_table::{Building, Installed, MemAttr, Stage2Tables},
};

use crate::stage2::scratch;

pub struct Stage2Vm<S> {
    tables: Stage2Tables<S>,
}

impl<S> Stage2Vm<S> {
    pub fn root_pa(&self) -> PhysAddr {
        self.tables.root_pa()
    }

    pub fn root_raw(&self) -> u64 {
        self.tables.root_raw()
    }

    // pub fn map_guest_ram(&mut self, ipa: IpaAddr, pa: PhysAddr, size: usize) {
    //     self.tables
    //         .map_range(ipa, pa, Self::align_2m(size), MemAttr::Normal)
    //         .unwrap_or_else(|_| panic_stage2_map_failed());
    // }

    fn align_2m(size: usize) -> usize {
        const BLOCK: usize = 2 * 1024 * 1024;
        (size + BLOCK - 1) & !(BLOCK - 1)
    }
}

impl Stage2Vm<Building> {
    pub fn new() -> Self {
        let scratch = scratch::get_mut();

        let tables =
            Stage2Tables::new(&mut scratch.tables.root, &mut scratch.tables.l2);

        Self { tables }
    }

    pub fn map_guest_ram(&mut self, ipa: IpaAddr, pa: PhysAddr, size: usize) {
        self.map_range(ipa, pa, Self::align_2m(size), MemAttr::Normal);
    }

    pub fn map_range(
        &mut self,
        ipa: IpaAddr,
        pa: PhysAddr,
        size: usize,
        attr: MemAttr,
    ) {
        self.tables
            .map_range(ipa, pa, size, attr)
            .unwrap_or_else(|_| panic_stage2_map_failed());
    }

    pub fn install(self) -> Stage2Vm<Installed> {
        self.enable();

        Stage2Vm {
            tables: self.tables.install(),
        }
    }

    pub fn enable(&self) {
        super::enable::enable_stage2(self.root_raw());
    }
}

impl Stage2Vm<Installed> {
    pub fn map_guest_ram(&mut self, ipa: IpaAddr, pa: PhysAddr, size: usize) {
        self.map_range(ipa, pa, Self::align_2m(size), MemAttr::Normal);
    }

    pub fn map_range(
        &mut self,
        ipa: IpaAddr,
        pa: PhysAddr,
        size: usize,
        attr: MemAttr,
    ) {
        self.tables
            .map_range(ipa, pa, size, attr)
            .unwrap_or_else(|_| panic_stage2_map_failed());
    }
}

fn panic_stage2_map_failed() -> ! {
    crate::log!("stage2: map_range failed");
    loop {
        core::hint::spin_loop();
    }
}
