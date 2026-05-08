use crate::{
    error::AllocError,
    frame::{FRAME_SIZE, FrameRange, PhysFrame, is_frame_aligned},
    traits::PageAllocator,
};
use core::ptr::NonNull;
use pyr_arch::addr::PhysAddr;

pub struct BitmapFrameAllocator {
    range: FrameRange,

    bitmap: NonNull<u64>,
    word_count: usize,

    frame_base: NonNull<u8>,
    frame_count: usize,
    free_count: usize,

    initialized: bool,
}

impl BitmapFrameAllocator {
    pub const fn uninit() -> Self {
        Self {
            range: FrameRange::empty(),
            bitmap: NonNull::dangling(),
            word_count: 0,
            frame_base: NonNull::dangling(),
            frame_count: 0,
            free_count: 0,
            initialized: false,
        }
    }

    /// Initialize the allocator over `start..start + len`.
    ///
    /// The allocator stores its bitmap at the beginning of the supplied range.
    /// The remaining frame-aligned memory becomes allocatable frames.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that:
    ///
    /// - `start..start + len` is valid writable physical memory.
    /// - The whole range is exclusively owned by this allocator.
    /// - No live Rust references, `PhysFrame`s, page tables, stacks, kernel image
    ///   bytes, DTBs, initrds, MMIO regions, or firmware data overlap this range.
    /// - `start` is identity-accessible by EL2 at the time this function runs,
    ///   because this implementation writes to `start` as a raw pointer.
    /// - This function is called at most once per allocator instance.
    /// - No other CPU or interrupt handler can access this allocator during init.
    pub unsafe fn init(
        &mut self,
        start: PhysAddr,
        len: u64,
    ) -> Result<(), AllocError> {
        if self.initialized {
            return Err(AllocError::BadRange);
        }

        let range = FrameRange::new(start, len).ok_or(AllocError::BadRange)?;
        let pool_start = start.as_u64() as *mut u8;
        let pool_bytes = range.len() as usize;

        let naive_frames = pool_bytes / FRAME_SIZE;
        let words_needed = naive_frames.div_ceil(64);
        let bitmap_bytes = words_needed * size_of::<u64>();
        let bitmap_frames = bitmap_bytes.div_ceil(FRAME_SIZE);

        if bitmap_frames >= naive_frames {
            return Err(AllocError::BadSize);
        }

        let frame_count = naive_frames - bitmap_frames;
        let word_count = frame_count.div_ceil(64);

        // SAFETY:
        // `pool_start` points to the beginning of the caller-owned pool.
        // `bitmap_frames * FRAME_SIZE` is within the pool because:
        //
        // - `bitmap_frames < naive_frames` was checked above.
        // - `naive_frames == pool_bytes / FRAME_SIZE`.
        // - therefore the computed frame base is still inside `start..start + len`.
        //
        // The resulting pointer is not dereferenced here; it is stored as the base
        // for later frame pointer derivation.
        let frame_base = unsafe { pool_start.add(bitmap_frames * FRAME_SIZE) };

        // SAFETY:
        // The bitmap lives at the beginning of the caller-owned pool.
        // `word_count * size_of::<u64>()` bytes fit inside the reserved bitmap area,
        // because `word_count <= words_needed` and `bitmap_frames` was computed from
        // `words_needed * size_of::<u64>()`, rounded up to full frames.
        //
        // The caller's `init` safety contract guarantees this memory is writable and
        // exclusively owned.
        unsafe {
            core::ptr::write_bytes(
                pool_start,
                0,
                word_count * size_of::<u64>(),
            );
        }

        self.range = range;
        self.bitmap =
            NonNull::new(pool_start as *mut u64).ok_or(AllocError::BadRange)?;
        self.word_count = word_count;
        self.frame_base =
            NonNull::new(frame_base).ok_or(AllocError::BadRange)?;
        self.frame_count = frame_count;
        self.free_count = frame_count;
        self.initialized = true;

        Ok(())
    }

    pub const fn is_initialized(&self) -> bool {
        self.initialized
    }

    pub const fn range(&self) -> FrameRange {
        self.range
    }

    fn ensure_init(&self) -> Result<(), AllocError> {
        self.initialized
            .then_some(())
            .ok_or(AllocError::NotInitialized)
    }

    fn bitmap_slice(&mut self) -> Result<&mut [u64], AllocError> {
        self.ensure_init()?;

        // SAFETY:
        // After successful init, `self.bitmap` points to the bitmap stored at the
        // start of the allocator pool and is valid for `self.word_count` u64 words.
        // `bitmap_slice` requires `&mut self`, so no second mutable bitmap slice can
        // exist at the same time through this allocator.
        Ok(unsafe {
            core::slice::from_raw_parts_mut(
                self.bitmap.as_ptr(),
                self.word_count,
            )
        })
    }

    fn alloc_index(&mut self) -> Result<usize, AllocError> {
        self.ensure_init()?;

        let frame_count = self.frame_count;
        let words = self.bitmap_slice()?;

        for (wi, word) in words.iter_mut().enumerate() {
            if *word == u64::MAX {
                continue;
            }

            let bit = word.trailing_ones() as usize;
            let idx = wi * 64 + bit;

            if idx >= frame_count {
                break;
            }

            *word |= 1u64 << bit;
            self.free_count -= 1;
            return Ok(idx);
        }

        Err(AllocError::OutOfMemory)
    }

    fn free_index(&mut self, idx: usize) -> Result<(), AllocError> {
        self.ensure_init()?;

        if idx >= self.frame_count {
            return Err(AllocError::BadRange);
        }

        let wi = idx / 64;
        let bit = idx % 64;
        let words = self.bitmap_slice()?;

        let word = words.get_mut(wi).ok_or(AllocError::BadRange)?;

        #[cfg(debug_assertions)]
        if *word & (1u64 << bit) == 0 {
            return Err(AllocError::DoubleFree);
        }

        *word &= !(1u64 << bit);
        self.free_count += 1;

        Ok(())
    }

    /// Convert an allocated frame index into an owned `PhysFrame`.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that:
    ///
    /// - `idx < self.frame_count`.
    /// - The bitmap bit for `idx` is currently set.
    /// - No other live `PhysFrame` exists for the same index.
    /// - The allocator has been initialized.
    unsafe fn index_to_frame(&self, idx: usize) -> PhysFrame {
        // SAFETY:
        // `idx` is guaranteed valid by the caller. `frame_base` was derived from
        // the original allocator pool pointer during init. Adding
        // `idx * FRAME_SIZE` stays within the frame area and preserves provenance.
        let ptr = unsafe {
            self.frame_base.as_ptr().add(idx * FRAME_SIZE)
                as *mut [u8; FRAME_SIZE]
        };

        // SAFETY:
        // The computed pointer is frame-aligned, valid for exactly one frame, and
        // uniquely owned because the bitmap bit for `idx` is set and no other
        // live `PhysFrame` exists for this index.
        unsafe { PhysFrame::from_ptr(NonNull::new_unchecked(ptr)) }
    }

    fn frame_to_index(&self, frame: &PhysFrame) -> Result<usize, AllocError> {
        let base = self.frame_base.as_ptr() as usize;
        let addr = frame.addr().as_u64() as usize;

        if addr < base || !is_frame_aligned(addr as u64) {
            return Err(AllocError::BadRange);
        }

        let idx = (addr - base) / FRAME_SIZE;

        if idx >= self.frame_count {
            return Err(AllocError::BadRange);
        }

        Ok(idx)
    }
}

impl PageAllocator for BitmapFrameAllocator {
    fn alloc_frame(&mut self) -> Result<PhysFrame, AllocError> {
        let idx = self.alloc_index()?;

        // SAFETY: The caller needs to guarentee that no other bitmap bit exist for the idx
        unsafe { Ok(self.index_to_frame(idx)) }
    }

    fn free_frame(&mut self, frame: PhysFrame) -> Result<(), AllocError> {
        let idx = self.frame_to_index(&frame)?;
        self.free_index(idx)?;

        // Prevent `frame` from being used after its bitmap bit is cleared.
        // `PhysFrame` has no Drop logic; forgetting it only consumes the handle.
        #[allow(clippy::forget_non_drop)]
        core::mem::forget(frame);

        Ok(())
    }

    fn free_frames(&self) -> usize {
        self.free_count
    }

    fn total_frames(&self) -> usize {
        self.frame_count
    }
}
