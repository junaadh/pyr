use core::{alloc::Layout, ptr::NonNull};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BumpError {
    AlreadyInitialized,
    InvalidRegion,
    OutOfMemory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeapStats {
    pub start: u64,
    pub end: u64,
    pub cursor: u64,
    pub used: u64,
    pub total: u64,
}

pub struct BumpHeap {
    start: u64,
    end: u64,
    cursor: u64,
    initialized: bool,
}

impl BumpHeap {
    pub const fn uninit() -> Self {
        Self {
            start: 0,
            end: 0,
            cursor: 0,
            initialized: false,
        }
    }

    /// # Safety
    ///
    /// `start..start + len` must be valid, writable, exclusively owned memory
    /// for the lifetime of this heap
    pub unsafe fn init(
        &mut self,
        start: u64,
        len: u64,
    ) -> Result<(), BumpError> {
        if self.initialized {
            return Err(BumpError::AlreadyInitialized);
        }

        let end = start.checked_add(len).ok_or(BumpError::InvalidRegion)?;

        if start == 0 || len == 0 || start >= end {
            return Err(BumpError::InvalidRegion);
        }

        self.start = align_up(start, 16).ok_or(BumpError::InvalidRegion)?;
        self.end = end;
        self.cursor = self.start;
        self.initialized = true;

        Ok(())
    }

    pub fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, BumpError> {
        if !self.initialized {
            return Err(BumpError::InvalidRegion);
        }

        let aligned = align_up(self.cursor, layout.align() as u64)
            .ok_or(BumpError::OutOfMemory)?;
        let next = aligned
            .checked_add(layout.size() as u64)
            .ok_or(BumpError::OutOfMemory)?;

        if next > self.end {
            return Err(BumpError::OutOfMemory);
        }

        self.cursor = next;

        let ptr = aligned as *mut u8;
        NonNull::new(ptr).ok_or(BumpError::InvalidRegion)
    }

    pub fn stats(&self) -> HeapStats {
        HeapStats {
            start: self.start,
            end: self.end,
            cursor: self.cursor,
            used: self.cursor.saturating_sub(self.start),
            total: self.end.saturating_sub(self.start),
        }
    }
}

const fn align_up(value: u64, align: u64) -> Option<u64> {
    if align == 0 || !align.is_power_of_two() {
        return None;
    }

    Some((value + align - 1) & !(align - 1))
}
