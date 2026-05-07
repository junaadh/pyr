use pyr_arch::{
    addr::{IpaAddr, PhysAddr},
    page_table::{
        Building, Installed, MapError, MemAttr, Stage2MappingDump, Stage2Tables,
    },
};

use crate::{
    guest::{memory::MapGuestRegion, region::GuestRegion},
    stage2::scratch,
};

pub struct Stage2Vm<S> {
    tables: Stage2Tables<S>,
}

impl<S> Stage2Vm<S> {
    pub fn dump_mapping(
        &mut self,
        ipa: pyr_arch::addr::IpaAddr,
    ) -> Result<Stage2MappingDump, MapError> {
        self.tables.dump_mapping(ipa)
    }

    pub fn root_pa(&self) -> PhysAddr {
        self.tables.root_pa()
    }

    pub fn root_raw(&self) -> u64 {
        self.tables.root_raw()
    }

    fn align_4k(size: usize) -> usize {
        const BLOCK: usize = 4096;
        (size + BLOCK - 1) & !(BLOCK - 1)
    }
}

impl Stage2Vm<Building> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let scratch = scratch::get_mut();

        let tables = Stage2Tables::new(
            &mut scratch.tables.root,
            &mut scratch.tables.l2,
            &mut scratch.tables.l3,
        );

        Self { tables }
    }

    pub fn map_guest_ram(
        &mut self,
        ipa: IpaAddr,
        pa: PhysAddr,
        size: usize,
    ) -> Result<(), MapError> {
        self.map_pages(ipa, pa, Self::align_4k(size), MemAttr::Normal)
    }

    pub fn map_pages(
        &mut self,
        ipa: IpaAddr,
        pa: PhysAddr,
        size: usize,
        attr: MemAttr,
    ) -> Result<(), MapError> {
        self.tables.map_pages(ipa, pa, size, attr)
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
    pub fn map_guest_ram(
        &mut self,
        ipa: IpaAddr,
        pa: PhysAddr,
        size: usize,
    ) -> Result<(), MapError> {
        self.map_pages(ipa, pa, Self::align_4k(size), MemAttr::Normal)
    }

    pub fn map_pages(
        &mut self,
        ipa: IpaAddr,
        pa: PhysAddr,
        size: usize,
        attr: MemAttr,
    ) -> Result<(), MapError> {
        self.tables.map_pages(ipa, pa, size, attr)
    }
}

impl MapGuestRegion for Stage2Vm<Building> {
    fn map_guest_region(
        &mut self,
        region: GuestRegion,
    ) -> Result<(), MapError> {
        self.map_pages(
            region.ipa(),
            region.pa(),
            Self::align_4k(region.size()),
            region.attr(),
        )
    }
}

impl MapGuestRegion for Stage2Vm<Installed> {
    fn map_guest_region(
        &mut self,
        region: GuestRegion,
    ) -> Result<(), MapError> {
        self.map_pages(
            region.ipa(),
            region.pa(),
            Self::align_4k(region.size()),
            region.attr(),
        )
    }
}
