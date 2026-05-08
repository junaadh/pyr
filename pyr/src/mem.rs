use core::{
    alloc::{GlobalAlloc, Layout},
    ptr,
};

use pyr_alloc::{
    bump::{BumpHeap, HeapStats},
    sync::SpinLock,
};
use pyr_arch::addr::PhysAddr;

use crate::fatal;

pub struct LockedBumpHeap {
    inner: SpinLock<BumpHeap>,
}

impl LockedBumpHeap {
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self {
            inner: SpinLock::new(BumpHeap::uninit()),
        }
    }

    /// # Safety
    ///
    /// `start..start + len` must be valid writable memory owned by Pyr.
    pub unsafe fn init(&self, start: PhysAddr, len: u64) {
        // SAFETY: `start..start+len` must be valid writable memory owned by Pyr
        unsafe {
            self.inner
                .lock()
                .init(start.as_u64(), len)
                .unwrap_or_else(|err| fatal!("heap init failed: {err:?}"))
        }
    }

    pub fn stats(&self) -> HeapStats {
        self.inner.lock().stats()
    }
}

#[global_allocator]
pub static HEAP: LockedBumpHeap = LockedBumpHeap::new();

// SAFETY: Lock serializes allocator access. Returned pointers come from
// the initialized bump heap or null on failure
unsafe impl GlobalAlloc for LockedBumpHeap {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        self.inner
            .lock()
            .alloc(layout)
            .map_or(ptr::null_mut(), |ptr| ptr.as_ptr())
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {
        // bump allocator: no-op
    }
}

#[alloc_error_handler]
fn alloc_error(layout: Layout) -> ! {
    fatal!(
        "allocation failed: size={} align={}",
        layout.size(),
        layout.align()
    )
}
