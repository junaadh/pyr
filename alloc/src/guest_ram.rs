use crate::error::AllocError;
use core::fmt;
use pyr_arch::addr::PhysAddr;

pub const GUEST_RAM_MIN_ALIGN: u64 = 2 * 1024 * 1024;
const MAX_FREE_BLOCKS: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GuestRam {
    base: PhysAddr,
    size: u64,
}

impl GuestRam {
    pub const fn new(base: PhysAddr, size: u64) -> Self {
        Self { base, size }
    }

    pub const fn base(&self) -> PhysAddr {
        self.base
    }

    pub const fn size(&self) -> u64 {
        self.size
    }

    pub fn end(&self) -> Result<PhysAddr, AllocError> {
        let end = self
            .base
            .as_u64()
            .checked_add(self.size)
            .ok_or(AllocError::BadRange)?;

        Ok(PhysAddr::new(end))
    }

    pub fn contains(&self, pa: PhysAddr) -> bool {
        let start = self.base.as_u64();
        let Ok(end) = self.end() else {
            return false;
        };

        start <= pa.as_u64() && pa.as_u64() < end.as_u64()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FreeRegion {
    base: PhysAddr,
    size: u64,
}

impl FreeRegion {
    fn end(&self) -> Result<u64, AllocError> {
        self.base
            .as_u64()
            .checked_add(self.size)
            .ok_or(AllocError::BadRange)
    }

    fn overlaps(&self, other: &Self) -> Result<bool, AllocError> {
        Ok(self.base.as_u64() < other.end()?
            && other.base.as_u64() < self.end()?)
    }
}

pub struct GuestRamArena {
    region_base: PhysAddr,
    region_size: u64,

    free_list: [Option<FreeRegion>; MAX_FREE_BLOCKS],
    block_count: usize,

    initialized: bool,
}

impl GuestRamArena {
    pub const fn uninit() -> Self {
        Self {
            region_base: PhysAddr::new(0),
            region_size: 0,
            free_list: [None; MAX_FREE_BLOCKS],
            block_count: 0,
            initialized: false,
        }
    }

    pub fn init(
        &mut self,
        base: PhysAddr,
        size: u64,
    ) -> Result<(), AllocError> {
        if self.initialized {
            return Err(AllocError::BadRange);
        }

        if !base.as_u64().is_multiple_of(GUEST_RAM_MIN_ALIGN) {
            return Err(AllocError::BadAlignment);
        }

        if size == 0 || !size.is_multiple_of(GUEST_RAM_MIN_ALIGN) {
            return Err(AllocError::BadSize);
        }

        base.as_u64()
            .checked_add(size)
            .ok_or(AllocError::BadRange)?;

        self.region_base = base;
        self.region_size = size;
        self.free_list = [None; MAX_FREE_BLOCKS];
        self.free_list[0] = Some(FreeRegion { base, size });
        self.block_count = 1;
        self.initialized = true;

        Ok(())
    }

    pub fn alloc_guest_ram(
        &mut self,
        size: u64,
        align: u64,
    ) -> Result<GuestRam, AllocError> {
        self.ensure_init()?;

        if size == 0 || !size.is_multiple_of(GUEST_RAM_MIN_ALIGN) {
            return Err(AllocError::BadSize);
        }

        if !align.is_power_of_two() || align < GUEST_RAM_MIN_ALIGN {
            return Err(AllocError::BadAlignment);
        }

        for index in 0..self.block_count {
            let Some(block) = self.slot(index)? else {
                continue;
            };

            let aligned = align_up_checked(block.base.as_u64(), align)?;
            let padding = aligned
                .checked_sub(block.base.as_u64())
                .ok_or(AllocError::BadRange)?;

            let needed =
                padding.checked_add(size).ok_or(AllocError::BadRange)?;

            if block.size < needed {
                continue;
            }

            let block_end = block.end()?;
            let tail_base =
                aligned.checked_add(size).ok_or(AllocError::BadRange)?;
            let tail_size = block_end
                .checked_sub(tail_base)
                .ok_or(AllocError::BadRange)?;

            if padding > 0 {
                *self.slot_mut(index)? = Some(FreeRegion {
                    base: block.base,
                    size: padding,
                });

                if tail_size > 0 {
                    self.insert_free(FreeRegion {
                        base: PhysAddr::new(tail_base),
                        size: tail_size,
                    })?;
                }
            } else if tail_size > 0 {
                *self.slot_mut(index)? = Some(FreeRegion {
                    base: PhysAddr::new(tail_base),
                    size: tail_size,
                });
            } else {
                self.remove_slot(index)?
            }

            return Ok(GuestRam::new(PhysAddr::new(aligned), size));
        }

        Err(AllocError::OutOfMemory)
    }

    pub fn free_guest_ram(&mut self, ram: GuestRam) -> Result<(), AllocError> {
        self.ensure_init()?;

        self.validate_returned_region(ram)?;

        let returned = FreeRegion {
            base: ram.base(),
            size: ram.size(),
        };

        for block in self.live_blocks()?.iter().flatten() {
            if block.overlaps(&returned)? {
                return Err(AllocError::BadRange);
            }
        }

        self.insert_free(returned)
    }

    pub const fn total_bytes(&self) -> u64 {
        self.region_size
    }

    pub fn free_bytes(&self) -> Result<u64, AllocError> {
        let mut total = 0u64;

        for block in self.live_blocks()?.iter().flatten() {
            total =
                total.checked_add(block.size).ok_or(AllocError::BadRange)?;
        }

        Ok(total)
    }

    pub const fn free_block_count(&self) -> usize {
        self.block_count
    }

    fn ensure_init(&self) -> Result<(), AllocError> {
        self.initialized
            .then_some(())
            .ok_or(AllocError::NotInitialized)
    }

    fn validate_returned_region(
        &self,
        ram: GuestRam,
    ) -> Result<(), AllocError> {
        if ram.size() == 0 || !ram.size().is_multiple_of(GUEST_RAM_MIN_ALIGN) {
            return Err(AllocError::BadSize);
        }

        if !ram.base().as_u64().is_multiple_of(GUEST_RAM_MIN_ALIGN) {
            return Err(AllocError::BadAlignment);
        }

        let arena_start = self.region_base.as_u64();
        let arena_end = arena_start
            .checked_add(self.region_size)
            .ok_or(AllocError::BadRange)?;

        let ram_start = ram.base().as_u64();
        let ram_end = ram.end()?.as_u64();

        if ram_start < arena_start || ram_end > arena_end {
            return Err(AllocError::BadRange);
        }

        Ok(())
    }

    fn insert_free(&mut self, region: FreeRegion) -> Result<(), AllocError> {
        if region.size == 0 {
            return Ok(());
        }

        let insert_at = self
            .live_blocks()?
            .iter()
            .flatten()
            .position(|block| block.base.as_u64() > region.base.as_u64())
            .unwrap_or(self.block_count);

        let merge_left = insert_at > 0
            && self
                .slot(insert_at - 1)?
                .map(|left| left.end() == Ok(region.base.as_u64()))
                .unwrap_or(false);

        let merge_right = insert_at < self.block_count
            && self
                .slot(insert_at)?
                .map(|right| region.end() == Ok(right.base.as_u64()))
                .unwrap_or(false);

        match (merge_left, merge_right) {
            (true, true) => {
                let right =
                    self.slot(insert_at)?.ok_or(AllocError::BadRange)?;
                let left = self
                    .slot_mut(insert_at - 1)?
                    .as_mut()
                    .ok_or(AllocError::BadRange)?;

                left.size = left
                    .size
                    .checked_add(region.size)
                    .and_then(|v| v.checked_add(right.size))
                    .ok_or(AllocError::BadRange)?;

                self.remove_slot(insert_at)?;
            }
            (true, false) => {
                let left = self
                    .slot_mut(insert_at - 1)?
                    .as_mut()
                    .ok_or(AllocError::BadRange)?;

                left.size = left
                    .size
                    .checked_add(region.size)
                    .ok_or(AllocError::BadRange)?;
            }
            (false, true) => {
                let right = self
                    .slot_mut(insert_at)?
                    .as_mut()
                    .ok_or(AllocError::BadRange)?;

                right.base = region.base;
                right.size = right
                    .size
                    .checked_add(region.size)
                    .ok_or(AllocError::BadRange)?;
            }
            (false, false) => {
                if self.block_count >= MAX_FREE_BLOCKS {
                    return Err(AllocError::OutOfMemory);
                }

                self.shift_right(insert_at)?;
                *self.slot_mut(insert_at)? = Some(region);
                self.block_count += 1;
            }
        }

        Ok(())
    }

    fn remove_slot(&mut self, index: usize) -> Result<(), AllocError> {
        if index >= self.block_count || self.block_count == 0 {
            return Err(AllocError::BadRange);
        }

        for i in index..self.block_count - 1 {
            let next = self.slot(i + 1)?;
            *self.slot_mut(i)? = next;
        }

        *self.slot_mut(self.block_count - 1)? = None;
        self.block_count -= 1;

        Ok(())
    }

    fn shift_right(&mut self, from: usize) -> Result<(), AllocError> {
        if self.block_count >= MAX_FREE_BLOCKS || from > self.block_count {
            return Err(AllocError::BadRange);
        }

        let mut i = self.block_count;

        while i > from {
            let prev = self.slot(i - 1)?;
            *self.slot_mut(i)? = prev;
            i -= 1;
        }

        Ok(())
    }
    fn live_blocks(&self) -> Result<&[Option<FreeRegion>], AllocError> {
        self.free_list
            .get(..self.block_count)
            .ok_or(AllocError::BadRange)
    }

    fn slot(&self, index: usize) -> Result<Option<FreeRegion>, AllocError> {
        self.free_list
            .get(index)
            .copied()
            .ok_or(AllocError::BadRange)
    }

    fn slot_mut(
        &mut self,
        index: usize,
    ) -> Result<&mut Option<FreeRegion>, AllocError> {
        self.free_list.get_mut(index).ok_or(AllocError::BadRange)
    }
}

impl fmt::Debug for GuestRamArena {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let free_bytes = self.free_bytes().unwrap_or(0);

        f.debug_struct("GuestRamArena")
            .field("base", &self.region_base.as_u64())
            .field("size", &self.region_size)
            .field("free_bytes", &free_bytes)
            .field("free_blocks", &self.block_count)
            .finish()
    }
}

fn align_up_checked(addr: u64, align: u64) -> Result<u64, AllocError> {
    let mask = align.checked_sub(1).ok_or(AllocError::BadAlignment)?;

    addr.checked_add(mask)
        .map(|v| v & !mask)
        .ok_or(AllocError::BadRange)
}
