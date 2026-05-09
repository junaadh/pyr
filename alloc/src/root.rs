use crate::{
    bitmap::BitmapFrameAllocator,
    error::AllocError,
    frame::PhysFrame,
    guest_ram::{GuestRam, GuestRamArena},
    traits::{GuestRamAllocator, PageAllocator},
};
use pyr_arch::addr::PhysAddr;

pub struct PyrAllocator {
    frames: BitmapFrameAllocator,
    guest_ram: GuestRamArena,
}

impl PyrAllocator {
    pub const fn uninit() -> Self {
        Self {
            frames: BitmapFrameAllocator::uninit(),
            guest_ram: GuestRamArena::uninit(),
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

    pub fn init_guest_ram_arena(
        &mut self,
        base: PhysAddr,
        size: u64,
    ) -> Result<(), AllocError> {
        self.guest_ram.init(base, size)
    }

    pub const fn is_initialized(&self) -> bool {
        self.frames.is_initialized()
    }

    pub fn guest_ram_free_bytes(&self) -> Result<u64, AllocError> {
        self.guest_ram.free_bytes()
    }

    pub const fn guest_ram_total_bytes(&self) -> u64 {
        self.guest_ram.total_bytes()
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

impl GuestRamAllocator for PyrAllocator {
    fn alloc_guest_ram(
        &mut self,
        size: u64,
        align: u64,
    ) -> Result<GuestRam, AllocError> {
        self.guest_ram.alloc_guest_ram(size, align)
    }

    fn free_guest_ram(&mut self, ram: GuestRam) -> Result<(), AllocError> {
        self.guest_ram.free_guest_ram(ram)
    }
}
