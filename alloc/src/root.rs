use crate::{
    bitmap::BitmapFrameAllocator, error::AllocError, frame::PhysFrame,
    traits::PageAllocator,
};
use pyr_arch::addr::PhysAddr;

pub struct PyrAllocator {
    frames: BitmapFrameAllocator,
}

impl PyrAllocator {
    pub const fn uninit() -> Self {
        Self {
            frames: BitmapFrameAllocator::uninit(),
        }
    }

    /// Initialize the root allocator.
    ///
    /// # Safety
    ///
    /// Same contract as `BitmapFrameAllocator::init`.
    pub unsafe fn init_frame_pool(
        &mut self,
        start: PhysAddr,
        len: u64,
    ) -> Result<(), AllocError> {
        // SAFETY: `start + len` must be valid owned memory
        unsafe { self.frames.init(start, len) }
    }

    pub const fn is_initialized(&self) -> bool {
        self.frames.is_initialized()
    }
}

impl PageAllocator for PyrAllocator {
    #[inline]
    fn alloc_frame(&mut self) -> Result<PhysFrame, AllocError> {
        self.frames.alloc_frame()
    }

    #[inline]
    fn alloc_zeroed_frame(&mut self) -> Result<PhysFrame, AllocError> {
        self.frames.alloc_zeroed_frame()
    }

    #[inline]
    fn free_frame(&mut self, frame: PhysFrame) -> Result<(), AllocError> {
        self.frames.free_frame(frame)
    }

    #[inline]
    fn free_frames(&self) -> usize {
        self.frames.free_frames()
    }

    #[inline]
    fn total_frames(&self) -> usize {
        self.frames.total_frames()
    }
}
