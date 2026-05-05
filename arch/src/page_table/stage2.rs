use core::marker::PhantomData;

use crate::{
    addr::{IpaAddr, PhysAddr},
    page::ENTRIES_PER_TABLE,
};

use super::{Descriptor, MemAttr};

pub struct Building;
pub struct Built;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum MapError {
    UnalignedAddress,
    UnalignedSize,
    OutOfRange,
    AlreadyMapped,
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
        for entry in self.entries.iter_mut() {
            *entry = Descriptor::invalid();
        }
    }
}

pub struct Stage2Tables<S> {
    root: &'static mut PageTable,
    l2: &'static mut PageTable,
    _state: PhantomData<S>,
}

impl Stage2Tables<Building> {
    pub fn new(
        root: &'static mut PageTable,
        l2: &'static mut PageTable,
    ) -> Self {
        root.clear();
        l2.clear();

        Self {
            root,
            l2,
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
        const BLOCK_SIZE: u64 = 2 * 1024 * 1024;

        if !ipa.as_u64().is_multiple_of(BLOCK_SIZE)
            || !pa.as_u64().is_multiple_of(BLOCK_SIZE)
        {
            return Err(MapError::UnalignedAddress);
        }

        if !(size as u64).is_multiple_of(BLOCK_SIZE) {
            return Err(MapError::UnalignedSize);
        }

        let start_l1 = ((ipa.as_u64() >> 30) & 0x1ff) as usize;
        let end_l1 =
            (((ipa.as_u64() + size as u64 - 1) >> 30) & 0x1ff) as usize;

        if start_l1 != end_l1 {
            return Err(MapError::OutOfRange);
        }

        let l2_pa = self.l2.entries.as_ptr() as u64;

        if let Some(desc) = self.root.entries.get(start_l1)
            && !desc.is_valid()
            && let Some(desc) = self.root.entries.get_mut(start_l1)
        {
            *desc = Descriptor::table(l2_pa);
        }

        let mut offset = 0u64;

        while offset < size as u64 {
            let cur_ipa = ipa.as_u64() + offset;
            let cur_pa = pa.as_u64() + offset;

            let l2_index = ((cur_ipa >> 21) & 0x1ff) as usize;

            if let Some(desc) = self.l2.entries.get(l2_index)
                && desc.is_valid()
            {
                return Err(MapError::AlreadyMapped);
            }

            if let Some(desc) = self.l2.entries.get_mut(l2_index) {
                *desc = Descriptor::block(cur_pa, attr);
            }

            offset += BLOCK_SIZE;
        }

        Ok(())
    }

    pub fn build(self) -> Stage2Tables<Built> {
        Stage2Tables {
            root: self.root,
            l2: self.l2,
            _state: PhantomData,
        }
    }
}

impl Stage2Tables<Built> {
    pub fn root_base(&self) -> PhysAddr {
        PhysAddr::new(self.root.entries.as_ptr() as u64)
    }

    pub fn root_raw(&self) -> u64 {
        self.root_base().as_u64()
    }
}
