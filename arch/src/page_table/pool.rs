use crate::{addr::PhysAddr, page_table::PageTable};

#[derive(Debug)]
pub enum PoolError {
    Exhausted,
    InvalidIndex,
}

pub struct PageTablePool {
    tables: [PageTable; Self::MAX_TABLES],
    used: usize,
}

impl PageTablePool {
    pub const MAX_TABLES: usize = 640;

    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self {
            tables: [const { PageTable::zeroed() }; Self::MAX_TABLES],
            used: 0,
        }
    }

    pub fn reset(&mut self) {
        self.used = 0;

        for table in self.tables.iter_mut() {
            table.clear();
        }
    }

    pub fn alloc_index(&mut self) -> Result<u16, PoolError> {
        let index = self.used;

        if index >= Self::MAX_TABLES {
            return Err(PoolError::Exhausted);
        }

        self.used += 1;

        if let Some(x) = self.tables.get_mut(index) {
            x.clear()
        }

        Ok(index as u16)
    }

    pub fn get_mut(&mut self, index: u16) -> Result<&mut PageTable, PoolError> {
        self.tables
            .get_mut(index as usize)
            .ok_or(PoolError::InvalidIndex)
    }

    pub fn get(&mut self, index: u16) -> Result<&PageTable, PoolError> {
        self.tables
            .get(index as usize)
            .ok_or(PoolError::InvalidIndex)
    }

    pub fn phys_addr_of(&self, index: u16) -> Result<PhysAddr, PoolError> {
        let Some(table) = self.tables.get(index as usize) else {
            return Err(PoolError::InvalidIndex);
        };

        Ok(PhysAddr::new(table as *const _ as u64))
    }
}
