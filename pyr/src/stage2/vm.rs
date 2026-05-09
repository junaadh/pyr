use crate::{
    guest::{memory::MapGuestRegion, region::GuestRegion},
    stage2::{invalidate::Stage2Invalidation, vmid::Vmid},
};
use alloc::vec::Vec;
use core::ptr::NonNull;
use pyr_alloc::{context::PyrContext, frame::PhysFrame, traits::PageAllocator};
use pyr_arch::{
    addr::{IpaAddr, PhysAddr},
    page_table::{
        Building, Installed, MapError, MemAttr, PageTable, Stage2MappingDump,
        Stage2Tables,
    },
};

pub struct Stage2Vm<S> {
    vmid: Vmid,
    root: PhysFrame,
    child_tables: Vec<PhysFrame>,
    tables: Stage2Tables<S>,
}

impl<S> Stage2Vm<S> {
    pub const fn vmid(&self) -> Vmid {
        self.vmid
    }

    pub fn dump_mapping(
        &mut self,
        ipa: IpaAddr,
    ) -> Result<Stage2MappingDump, MapError> {
        self.tables.dump_mapping(ipa)
    }

    pub fn root_pa(&self) -> PhysAddr {
        self.root.addr()
    }

    pub fn root_raw(&self) -> u64 {
        self.root_pa().as_u64()
    }

    fn align_4k(size: usize) -> usize {
        const BLOCK: usize = 4096;
        (size + BLOCK - 1) & !(BLOCK - 1)
    }

    fn alloc_table<A>(
        cx: &mut PyrContext<A>,
        child_tables: &mut Vec<PhysFrame>,
    ) -> Result<PhysAddr, MapError>
    where
        A: PageAllocator,
    {
        let frame = cx
            .alloc_zeroed_frame()
            .map_err(|_| MapError::OutOfPageTables)?;

        let pa = frame.addr();

        child_tables.push(frame);

        Ok(pa)
    }
}

impl Stage2Vm<Building> {
    pub fn new<A>(cx: &mut PyrContext<A>) -> Result<Self, MapError>
    where
        A: PageAllocator,
    {
        let root = cx
            .alloc_zeroed_frame()
            .map_err(|_| MapError::OutOfPageTables)?;

        // SAFETY:
        // `root` is a freshly allocated zeroed 4 KiB frame from the frame allocator.
        // `PhysFrame` guarantees the pointer is non-null, page-aligned, writable,
        // and uniquely owned by this Stage2Vm.
        let root_ptr = unsafe {
            NonNull::new_unchecked(root.as_ptr().as_ptr() as *mut PageTable)
        };

        // SAFETY:
        // `root_ptr` points to a valid zeroed PageTable frame. This Stage2Vm owns
        // `root`, so the backing memory stays alive for the lifetime of `tables`.
        let tables = unsafe { Stage2Tables::new(root_ptr) };
        Ok(Self {
            vmid: Vmid::BOOT,
            root,
            child_tables: Vec::new(),
            tables,
        })
    }

    pub fn map_guest_ram<A>(
        &mut self,
        cx: &mut PyrContext<A>,
        ipa: IpaAddr,
        pa: PhysAddr,
        size: usize,
    ) -> Result<(), MapError>
    where
        A: PageAllocator,
    {
        self.map_pages(cx, ipa, pa, Self::align_4k(size), MemAttr::Normal)
    }

    pub fn map_pages<A>(
        &mut self,
        cx: &mut PyrContext<A>,
        ipa: IpaAddr,
        pa: PhysAddr,
        size: usize,
        attr: MemAttr,
    ) -> Result<(), MapError>
    where
        A: PageAllocator,
    {
        let child_tables = &mut self.child_tables;

        self.tables.map_pages(ipa, pa, size, attr, || {
            Self::alloc_table(cx, child_tables)
        })
    }

    pub fn map_blocks<A>(
        &mut self,
        cx: &mut PyrContext<A>,
        ipa: IpaAddr,
        pa: PhysAddr,
        size: usize,
        attr: MemAttr,
    ) -> Result<(), MapError>
    where
        A: PageAllocator,
    {
        let child_tables = &mut self.child_tables;

        self.tables.map_blocks(ipa, pa, size, attr, || {
            Self::alloc_table(cx, child_tables)
        })
    }

    pub fn install(self) -> Stage2Vm<Installed> {
        self.enable();

        Stage2Vm {
            vmid: self.vmid,
            root: self.root,
            child_tables: self.child_tables,
            tables: self.tables.install(),
        }
    }

    pub fn enable(&self) {
        super::enable::enable_stage2(self.root_raw());
    }
}

impl Stage2Vm<Installed> {
    pub fn flush_all_translations(&self) {
        Stage2Invalidation::flush_all();
    }
}

impl<A: PageAllocator> MapGuestRegion<A> for Stage2Vm<Building> {
    fn map_guest_region(
        &mut self,
        cx: &mut PyrContext<A>,
        region: GuestRegion,
    ) -> Result<(), MapError> {
        self.map_pages(
            cx,
            region.ipa(),
            region.pa(),
            Self::align_4k(region.size()),
            region.attr(),
        )
    }
}
