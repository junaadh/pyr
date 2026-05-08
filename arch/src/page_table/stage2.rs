use core::{marker::PhantomData, ptr::NonNull};

use crate::{
    addr::{IpaAddr, PhysAddr},
    barrier::{dsb_ish, isb},
    page::ENTRIES_PER_TABLE,
};

use super::{Descriptor, MemAttr};

pub struct Building;
pub struct Installed;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum MapError {
    UnalignedAddress,
    UnalignedSize,
    AlreadyMapped,
    IndexOutOfRange,
    OutOfPageTables,
    InvalidTable,
}

#[repr(align(4096))]
pub struct PageTable {
    entries: [Descriptor; ENTRIES_PER_TABLE],
}

impl PageTable {
    pub const fn zeroed() -> Self {
        Self {
            entries: [Descriptor::invalid(); ENTRIES_PER_TABLE],
        }
    }

    pub fn clear(&mut self) {
        self.entries.fill(Descriptor::invalid());
    }

    fn paddr(&self) -> PhysAddr {
        PhysAddr::new(self.entries.as_ptr() as u64)
    }

    fn entry(&self, index: usize) -> Result<Descriptor, MapError> {
        self.entries
            .get(index)
            .copied()
            .ok_or(MapError::IndexOutOfRange)
    }

    fn entry_mut(&mut self, index: usize) -> Result<&mut Descriptor, MapError> {
        self.entries.get_mut(index).ok_or(MapError::IndexOutOfRange)
    }
}

pub struct Stage2Tables<S> {
    root: NonNull<PageTable>,
    _state: PhantomData<S>,
}

impl Stage2Tables<Building> {
    /// # Safety
    ///
    /// `root` must point to a valid, 4 KiB-aligned, zeroed page table.
    /// The caller must own the backing frame for atleast as long as this
    /// `Stage2Tables` value lives
    pub unsafe fn new(root: NonNull<PageTable>) -> Self {
        // SAFETY: `root` must be a valid pointer to a writable `PageTable`
        unsafe {
            if let Some(x) = root.as_ptr().as_mut() {
                x.clear()
            }
        }

        Self {
            root,
            _state: PhantomData,
        }
    }

    pub fn install(self) -> Stage2Tables<Installed> {
        Stage2Tables {
            root: self.root,
            _state: PhantomData,
        }
    }

    pub fn map_blocks<F>(
        &mut self,
        ipa: IpaAddr,
        pa: PhysAddr,
        size: usize,
        attr: MemAttr,
        mut alloc_table: F,
    ) -> Result<(), MapError>
    where
        F: FnMut() -> Result<PhysAddr, MapError>,
    {
        self.map_blocks_inner(ipa, pa, size, attr, &mut alloc_table)
    }

    pub fn map_pages<F>(
        &mut self,
        ipa: IpaAddr,
        pa: PhysAddr,
        size: usize,
        attr: MemAttr,
        mut alloc_table: F,
    ) -> Result<(), MapError>
    where
        F: FnMut() -> Result<PhysAddr, MapError>,
    {
        self.map_pages_inner(ipa, pa, size, attr, &mut alloc_table)
    }
}

impl Stage2Tables<Installed> {
    pub fn map_blocks<F>(
        &mut self,
        ipa: IpaAddr,
        pa: PhysAddr,
        size: usize,
        attr: MemAttr,
        mut alloc_table: F,
    ) -> Result<(), MapError>
    where
        F: FnMut() -> Result<PhysAddr, MapError>,
    {
        self.map_blocks_inner(ipa, pa, size, attr, &mut alloc_table)?;
        flush_stage2_tlb();
        Ok(())
    }

    pub fn map_pages<F>(
        &mut self,
        ipa: IpaAddr,
        pa: PhysAddr,
        size: usize,
        attr: MemAttr,
        mut alloc_table: F,
    ) -> Result<(), MapError>
    where
        F: FnMut() -> Result<PhysAddr, MapError>,
    {
        self.map_pages_inner(ipa, pa, size, attr, &mut alloc_table)?;
        flush_stage2_tlb();
        Ok(())
    }
}

impl<S> Stage2Tables<S> {
    const BLOCK_SIZE: u64 = 2 * 1024 * 1024;
    const PAGE_SIZE: u64 = 4096;

    pub fn root_pa(&self) -> PhysAddr {
        // SAFETY:
        // `root` was created from a valid frame-backed PageTable in
        // `Stage2Tables::new` and the owning Stage2Vm keeps that frame alive.
        unsafe { self.root.as_ref().paddr() }
    }

    pub fn root_raw(&self) -> u64 {
        self.root_pa().as_u64()
    }

    fn root_mut(&mut self) -> &mut PageTable {
        // SAFETY:
        // `&mut self` guarantees exclusive access to the table walker. The root
        // pointer is valid for the lifetime of this Stage2Tables value because
        // Stage2Vm owns the backing PhysFrame.
        unsafe { self.root.as_mut() }
    }

    unsafe fn table_at_mut<'a>(
        pa: PhysAddr,
    ) -> Result<&'a mut PageTable, MapError> {
        let ptr = pa.as_u64() as *mut PageTable;

        if ptr.is_null() || !pa.as_u64().is_multiple_of(Self::PAGE_SIZE) {
            return Err(MapError::InvalidTable);
        }

        // SAFETY:
        // `pa` came from a table descriptor installed by this walker. Table
        // descriptors are only created from frame addresses returned by the
        // allocation callback, so the address is valid, 4 KiB-aligned, and points
        // to a live PageTable frame owned by Stage2Vm.
        Ok(unsafe { &mut *ptr })
    }

    fn ensure_table<'a, F>(
        entry: &mut Descriptor,
        alloc_table: &mut F,
    ) -> Result<&'a mut PageTable, MapError>
    where
        F: FnMut() -> Result<PhysAddr, MapError>,
    {
        if entry.is_valid() {
            if !entry.is_table() {
                return Err(MapError::AlreadyMapped);
            }

            // SAFETY:
            // A valid table descriptor in this walker can only have been installed by
            // `ensure_table` from an allocator-provided PageTable frame. Therefore its
            // output address points to a live child table owned by Stage2Vm.
            return unsafe { Self::table_at_mut(entry.output_addr()) };
        }

        let table_pa = alloc_table()?;
        *entry = Descriptor::table(table_pa.as_u64());

        // SAFETY:
        // `alloc_table` returns the physical address of a freshly zeroed PageTable
        // frame and stores ownership in Stage2Vm before returning. The descriptor
        // above now points at that live table.
        unsafe { Self::table_at_mut(table_pa) }
    }

    fn map_pages_inner<F>(
        &mut self,
        ipa: IpaAddr,
        pa: PhysAddr,
        size: usize,
        attr: MemAttr,
        alloc_table: &mut F,
    ) -> Result<(), MapError>
    where
        F: FnMut() -> Result<PhysAddr, MapError>,
    {
        Self::validate_page_mapping(ipa, pa, size)?;

        for offset in (0..size as u64).step_by(Self::PAGE_SIZE as usize) {
            let cur_ipa = ipa.offset(offset);
            let cur_pa = pa.offset(offset);
            self.map_page(cur_ipa, cur_pa, attr, alloc_table)?;
        }

        Ok(())
    }

    fn map_blocks_inner<F>(
        &mut self,
        ipa: IpaAddr,
        pa: PhysAddr,
        size: usize,
        attr: MemAttr,
        alloc_table: &mut F,
    ) -> Result<(), MapError>
    where
        F: FnMut() -> Result<PhysAddr, MapError>,
    {
        Self::validate_block_mapping(ipa, pa, size)?;

        for offset in (0..size as u64).step_by(Self::BLOCK_SIZE as usize) {
            let cur_ipa = ipa.offset(offset);
            let cur_pa = pa.offset(offset);
            self.map_block(cur_ipa, cur_pa, attr, alloc_table)?;
        }

        Ok(())
    }

    fn map_page<F>(
        &mut self,
        ipa: IpaAddr,
        pa: PhysAddr,
        attr: MemAttr,
        alloc_table: &mut F,
    ) -> Result<(), MapError>
    where
        F: FnMut() -> Result<PhysAddr, MapError>,
    {
        let l1 = Self::l1_index(ipa);
        let l2 = Self::l2_index(ipa);
        let l3 = Self::l3_index(ipa);

        let l1_entry = self.root_mut().entry_mut(l1)?;
        let l2_table = Self::ensure_table(l1_entry, alloc_table)?;

        let l2_entry = l2_table.entry_mut(l2)?;
        let l3_table = Self::ensure_table(l2_entry, alloc_table)?;

        let l3_entry = l3_table.entry_mut(l3)?;

        if l3_entry.is_valid() {
            return Err(MapError::AlreadyMapped);
        }

        *l3_entry = Descriptor::page(pa.as_u64(), attr);
        Ok(())
    }

    fn map_block<F>(
        &mut self,
        ipa: IpaAddr,
        pa: PhysAddr,
        attr: MemAttr,
        alloc_table: &mut F,
    ) -> Result<(), MapError>
    where
        F: FnMut() -> Result<PhysAddr, MapError>,
    {
        let l1 = Self::l1_index(ipa);
        let l2 = Self::l2_index(ipa);

        let l1_entry = self.root_mut().entry_mut(l1)?;
        let l2_table = Self::ensure_table(l1_entry, alloc_table)?;

        let l2_entry = l2_table.entry_mut(l2)?;

        if l2_entry.is_valid() {
            return Err(MapError::AlreadyMapped);
        }

        *l2_entry = Descriptor::block(pa.as_u64(), attr);
        Ok(())
    }

    fn validate_page_mapping(
        ipa: IpaAddr,
        pa: PhysAddr,
        size: usize,
    ) -> Result<(), MapError> {
        if size == 0 {
            return Err(MapError::UnalignedSize);
        }

        if !ipa.as_u64().is_multiple_of(Self::PAGE_SIZE)
            || !pa.as_u64().is_multiple_of(Self::PAGE_SIZE)
        {
            return Err(MapError::UnalignedAddress);
        }

        if !(size as u64).is_multiple_of(Self::PAGE_SIZE) {
            return Err(MapError::UnalignedSize);
        }

        Ok(())
    }

    fn validate_block_mapping(
        ipa: IpaAddr,
        pa: PhysAddr,
        size: usize,
    ) -> Result<(), MapError> {
        if size == 0 {
            return Err(MapError::UnalignedSize);
        }

        if !ipa.as_u64().is_multiple_of(Self::BLOCK_SIZE)
            || !pa.as_u64().is_multiple_of(Self::BLOCK_SIZE)
        {
            return Err(MapError::UnalignedAddress);
        }

        if !(size as u64).is_multiple_of(Self::BLOCK_SIZE) {
            return Err(MapError::UnalignedSize);
        }

        Ok(())
    }

    fn l1_index(ipa: IpaAddr) -> usize {
        ((ipa.as_u64() >> 30) & 0x1ff) as usize
    }

    fn l2_index(ipa: IpaAddr) -> usize {
        ((ipa.as_u64() >> 21) & 0x1ff) as usize
    }

    fn l3_index(ipa: IpaAddr) -> usize {
        ((ipa.as_u64() >> 12) & 0x1ff) as usize
    }

    pub fn dump_mapping(
        &mut self,
        ipa: IpaAddr,
    ) -> Result<Stage2MappingDump, MapError> {
        let l1 = Self::l1_index(ipa);
        let l2 = Self::l2_index(ipa);
        let l3 = Self::l3_index(ipa);

        let l1_desc = self.root_mut().entry(l1)?.raw();

        let mut l2_desc = 0;
        let mut l3_desc = None;

        let l1d = self.root_mut().entry(l1)?;
        if l1d.is_table() {
            // SAFETY:
            // `l1d` is a table descriptor read from this walker's root table. Such table
            // descriptors are only installed from allocator-backed child PageTable frames.
            let l2_table = unsafe { Self::table_at_mut(l1d.output_addr())? };
            l2_desc = l2_table.entry(l2)?.raw();

            let l2d = l2_table.entry(l2)?;
            if l2d.is_table() {
                // SAFETY:
                // `l2d` is a table descriptor read from an allocator-backed L2 table. It was
                // installed by this walker and points to a live L3 PageTable frame.
                let l3_table =
                    unsafe { Self::table_at_mut(l2d.output_addr())? };
                l3_desc = Some(l3_table.entry(l3)?.raw())
            }
        }

        Ok(Stage2MappingDump {
            ipa,
            l1_index: l1,
            l2_index: l2,
            l3_index: l3,
            l1_desc,
            l2_desc,
            l3_desc,
        })
    }
}

fn flush_stage2_tlb() {
    dsb_ish();

    // SAFETY: Valid at EL2 after the stage-2 translation is installed
    unsafe {
        core::arch::asm!("tlbi vmalls12e1is", options(nostack, nomem));
    }

    dsb_ish();
    isb();
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Stage2MappingDump {
    pub ipa: IpaAddr,
    pub l1_index: usize,
    pub l2_index: usize,
    pub l3_index: usize,
    pub l1_desc: u64,
    pub l2_desc: u64,
    pub l3_desc: Option<u64>,
}
