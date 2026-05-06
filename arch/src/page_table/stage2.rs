use core::marker::PhantomData;

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
    CrossesL1Boundary,
    AlreadyMapped,
    IndexOutOfRange,
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
    root: &'static mut PageTable,
    l2: &'static mut PageTable,
    l3: &'static mut PageTable,
    _state: PhantomData<S>,
}

impl Stage2Tables<Building> {
    pub fn new(
        root: &'static mut PageTable,
        l2: &'static mut PageTable,
        l3: &'static mut PageTable,
    ) -> Self {
        root.clear();
        l2.clear();
        l3.clear();

        Self {
            root,
            l2,
            l3,
            _state: PhantomData,
        }
    }

    pub fn map_range(
        &mut self,
        ipa: IpaAddr,
        pa: PhysAddr,
        size: usize,
        attr: MemAttr,
    ) -> Result<(), MapError> {
        self.map_range_inner(ipa, pa, size, attr)
    }

    pub fn install(self) -> Stage2Tables<Installed> {
        Stage2Tables {
            root: self.root,
            l2: self.l2,
            l3: self.l3,
            _state: PhantomData,
        }
    }

    pub fn map_pages(
        &mut self,
        ipa: IpaAddr,
        pa: PhysAddr,
        size: usize,
        attr: MemAttr,
    ) -> Result<(), MapError> {
        self.map_pages_inner(ipa, pa, size, attr)
    }
}

impl Stage2Tables<Installed> {
    pub fn map_range(
        &mut self,
        ipa: IpaAddr,
        pa: PhysAddr,
        size: usize,
        attr: MemAttr,
    ) -> Result<(), MapError> {
        self.map_range_inner(ipa, pa, size, attr)?;
        flush_stage2_tlb();
        Ok(())
    }

    pub fn map_pages(
        &mut self,
        ipa: IpaAddr,
        pa: PhysAddr,
        size: usize,
        attr: MemAttr,
    ) -> Result<(), MapError> {
        self.map_pages_inner(ipa, pa, size, attr)?;
        flush_stage2_tlb();
        Ok(())
    }
}

impl<S> Stage2Tables<S> {
    const BLOCK_SIZE: u64 = 2 * 1024 * 1024;
    const PAGE_SIZE: u64 = 4096;

    pub fn root_pa(&self) -> PhysAddr {
        self.root.paddr()
    }

    pub fn root_raw(&self) -> u64 {
        self.root_pa().as_u64()
    }

    fn map_range_inner(
        &mut self,
        ipa: IpaAddr,
        pa: PhysAddr,
        size: usize,
        attr: MemAttr,
    ) -> Result<(), MapError> {
        Self::validate_mapping(ipa, pa, size)?;

        let l1 = Self::l1_index(ipa);
        self.ensure_l2_table(l1)?;

        for offset in (0..size as u64).step_by(Self::BLOCK_SIZE as usize) {
            let cur_ipa = ipa.offset(offset);
            let cur_pa = pa.offset(offset);
            self.map_block(cur_ipa, cur_pa, attr)?;
        }

        Ok(())
    }

    fn validate_mapping(
        ipa: IpaAddr,
        pa: PhysAddr,
        size: usize,
    ) -> Result<(), MapError> {
        if !ipa.as_u64().is_multiple_of(Self::BLOCK_SIZE)
            || !pa.as_u64().is_multiple_of(Self::BLOCK_SIZE)
        {
            return Err(MapError::UnalignedAddress);
        }

        if !(size as u64).is_multiple_of(Self::BLOCK_SIZE) {
            return Err(MapError::UnalignedSize);
        }

        let start_l1 = Self::l1_index(ipa);
        let end_ipa = ipa.offset(size as u64 - 1);
        let end_l1 = Self::l1_index(end_ipa);

        if start_l1 != end_l1 {
            return Err(MapError::CrossesL1Boundary);
        }

        Ok(())
    }

    fn ensure_l2_table(&mut self, l1_index: usize) -> Result<(), MapError> {
        if self.root.entry(l1_index)?.is_valid() {
            return Ok(());
        }

        let l2_pa = self.l2.paddr().as_u64();
        *self.root.entry_mut(l1_index)? = Descriptor::table(l2_pa);

        Ok(())
    }

    fn map_block(
        &mut self,
        ipa: IpaAddr,
        pa: PhysAddr,
        attr: MemAttr,
    ) -> Result<(), MapError> {
        let l2_index = Self::l2_index(ipa);

        if self.l2.entry(l2_index)?.is_valid() {
            return Err(MapError::AlreadyMapped);
        }

        *self.l2.entry_mut(l2_index)? = Descriptor::block(pa.as_u64(), attr);

        Ok(())
    }

    fn map_pages_inner(
        &mut self,
        ipa: IpaAddr,
        pa: PhysAddr,
        size: usize,
        attr: MemAttr,
    ) -> Result<(), MapError> {
        Self::validate_page_mapping(ipa, pa, size)?;

        let l1 = Self::l1_index(ipa);
        let l2 = Self::l2_index(ipa);

        self.ensure_l2_table(l1)?;
        self.ensure_l3_table(l2)?;

        for offset in (0..size as u64).step_by(Self::PAGE_SIZE as usize) {
            let cur_ipa = ipa.offset(offset);
            let cur_pa = pa.offset(offset);
            self.map_page(cur_ipa, cur_pa, attr)?;
        }

        Ok(())
    }

    fn validate_page_mapping(
        ipa: IpaAddr,
        pa: PhysAddr,
        size: usize,
    ) -> Result<(), MapError> {
        if !ipa.as_u64().is_multiple_of(Self::PAGE_SIZE)
            || !pa.as_u64().is_multiple_of(Self::PAGE_SIZE)
        {
            return Err(MapError::UnalignedAddress);
        }

        if !(size as u64).is_multiple_of(Self::PAGE_SIZE) {
            return Err(MapError::UnalignedSize);
        }

        let end_ipa = ipa.offset(size as u64 - 1);

        let start_l1 = Self::l1_index(ipa);
        let end_l1 = Self::l1_index(end_ipa);

        if start_l1 != end_l1 {
            return Err(MapError::CrossesL1Boundary);
        }

        let start_l2 = Self::l2_index(ipa);
        let end_l2 = Self::l2_index(end_ipa);

        if start_l2 != end_l2 {
            return Err(MapError::CrossesL1Boundary);
        }

        Ok(())
    }

    fn ensure_l3_table(&mut self, l2_index: usize) -> Result<(), MapError> {
        if self.l2.entry(l2_index)?.is_valid() {
            return Ok(());
        }

        let l3_pa = self.l3.paddr().as_u64();
        *self.l2.entry_mut(l2_index)? = Descriptor::table(l3_pa);

        Ok(())
    }

    fn map_page(
        &mut self,
        ipa: IpaAddr,
        pa: PhysAddr,
        attr: MemAttr,
    ) -> Result<(), MapError> {
        let l3_index = Self::l3_index(ipa);

        if self.l3.entry(l3_index)?.is_valid() {
            return Err(MapError::AlreadyMapped);
        }

        *self.l3.entry_mut(l3_index)? = Descriptor::page(pa.as_u64(), attr);

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
