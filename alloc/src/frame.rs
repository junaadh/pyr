use core::{mem, ptr::NonNull};

use pyr_arch::addr::PhysAddr;

pub const FRAME_SIZE: usize = 4096;
pub const FRAME_SHIFT: usize = 12;
pub const FRAME_ALIGN: usize = FRAME_SIZE;
pub const FRAME_MASK: usize = !(FRAME_SIZE - 1);

#[inline]
pub const fn is_frame_aligned(addr: u64) -> bool {
    addr & (FRAME_SIZE as u64 - 1) == 0
}

#[inline]
pub const fn align_down(addr: u64) -> u64 {
    addr & FRAME_MASK as u64
}

#[inline]
pub fn align_up(addr: u64) -> Option<u64> {
    let mask = FRAME_SIZE as u64 - 1;
    addr.checked_add(mask).map(|v| v & !mask)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameRange {
    start: PhysAddr,
    count: usize,
}

impl FrameRange {
    pub fn new(start: PhysAddr, len: u64) -> Option<Self> {
        if len == 0 || !is_frame_aligned(start.as_u64()) {
            return None;
        }

        let len = align_down(len);
        let count = usize::try_from(len / FRAME_SIZE as u64).ok()?;

        if count == 0 {
            return None;
        }

        Some(Self { start, count })
    }

    pub const fn empty() -> Self {
        Self {
            start: PhysAddr::new(0),
            count: 0,
        }
    }

    pub const fn start(&self) -> PhysAddr {
        self.start
    }

    pub const fn end(&self) -> PhysAddr {
        PhysAddr::new(self.start.as_u64() + self.len())
    }

    pub const fn len(&self) -> u64 {
        self.count as u64 * FRAME_SIZE as u64
    }

    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub const fn count(&self) -> usize {
        self.count
    }

    pub fn contains(&self, frame: &PhysFrame) -> bool {
        let addr = frame.addr().as_u64();
        addr >= self.start.as_u64() && addr < self.end().as_u64()
    }

    pub const fn frame_addr(&self, index: usize) -> Option<PhysAddr> {
        if index >= self.count {
            return None;
        }

        Some(PhysAddr::new(
            self.start.as_u64() + index as u64 * FRAME_SIZE as u64,
        ))
    }
}

#[repr(transparent)]
pub struct PhysFrame {
    ptr: NonNull<[u8; FRAME_SIZE]>,
}

// SAFETY: PhysFrame has unique ownership of the frame. Sending it transfers
// ownership to another execution context and does not create aliasing.
unsafe impl Send for PhysFrame {}

impl PhysFrame {
    /// # Safety
    ///
    /// `ptr` must be 4 KiB aligned, valid for 4096 writable bytes, uniquely
    /// owned, and derived from the allocator pool.
    pub unsafe fn from_ptr(ptr: NonNull<[u8; FRAME_SIZE]>) -> Self {
        debug_assert_eq!(
            (ptr.as_ptr() as *mut u8 as usize) & (FRAME_SIZE - 1),
            0
        );

        Self { ptr }
    }

    pub fn addr(&self) -> PhysAddr {
        PhysAddr::new(self.ptr.as_ptr() as *mut u8 as u64)
    }

    pub fn as_ptr(&self) -> NonNull<[u8; FRAME_SIZE]> {
        self.ptr
    }

    pub fn zero(&mut self) {
        // SAFETY: PhysFrame has ownership over the ptr with len of a `FRAME_SIZE`
        unsafe {
            core::ptr::write_bytes(self.ptr.as_ptr() as *mut u8, 0, FRAME_SIZE);
        }
    }

    /// # Safety
    ///
    /// `T` must fit inside one frame, its alignment must be satisfied by 4 KiB
    /// alignment, and no other reference to this frame may exist while the
    /// returned reference is live.
    pub unsafe fn as_mut<T>(&mut self) -> &mut T {
        debug_assert!(mem::size_of::<T>() <= FRAME_SIZE);

        debug_assert!(mem::align_of::<T>() <= FRAME_ALIGN);

        // SAFETY: PhysFrame has ownership over the ptr with len of a `FRAME_SIZE`
        unsafe { &mut *(self.ptr.as_ptr() as *mut T) }
    }

    /// # Safety
    ///
    /// Caller becomes responsible for not using this pointer after the frame is
    /// freed or reissued.
    pub unsafe fn into_raw(self) -> NonNull<[u8; FRAME_SIZE]> {
        let ptr = self.ptr;

        // FIXME: This is allowed for this early boot dev
        #[allow(clippy::forget_non_drop)]
        mem::forget(self);

        ptr
    }
}

impl core::fmt::Debug for PhysFrame {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PhysFrame")
            .field("addr", &self.addr().as_u64())
            .finish()
    }
}
