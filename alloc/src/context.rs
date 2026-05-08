use crate::{error::AllocError, frame::PhysFrame, traits::PageAllocator};

pub struct PyrContext<'a, A> {
    alloc: &'a mut A,
}

impl<'a, A> PyrContext<'a, A> {
    pub const fn new(alloc: &'a mut A) -> Self {
        Self { alloc }
    }

    pub fn alloc_mut(&mut self) -> &mut A {
        self.alloc
    }
}

impl<A> PyrContext<'_, A>
where
    A: PageAllocator,
{
    #[inline]
    pub fn alloc_frame(&mut self) -> Result<PhysFrame, AllocError> {
        self.alloc.alloc_frame()
    }

    #[inline]
    pub fn alloc_zeroed_frame(&mut self) -> Result<PhysFrame, AllocError> {
        self.alloc.alloc_zeroed_frame()
    }

    #[inline]
    pub fn free_frame(&mut self, frame: PhysFrame) -> Result<(), AllocError> {
        self.alloc.free_frame(frame)
    }

    #[inline]
    pub fn free_frames(&self) -> usize {
        self.alloc.free_frames()
    }

    #[inline]
    pub fn total_frames(&self) -> usize {
        self.alloc.total_frames()
    }

    #[inline]
    pub fn used_frames(&self) -> usize {
        self.alloc.used_frames()
    }
}
