use crate::{error::AllocError, frame::PhysFrame};

pub trait PageAllocator {
    fn alloc_frame(&mut self) -> Result<PhysFrame, AllocError>;

    fn alloc_zeroed_frame(&mut self) -> Result<PhysFrame, AllocError> {
        let mut frame = self.alloc_frame()?;
        frame.zero();
        Ok(frame)
    }

    fn free_frame(&mut self, frame: PhysFrame) -> Result<(), AllocError>;

    fn free_frames(&self) -> usize;
    fn total_frames(&self) -> usize;

    fn used_frames(&self) -> usize {
        self.total_frames() - self.free_frames()
    }

    fn is_empty(&self) -> bool {
        self.free_frames() == 0
    }
}
