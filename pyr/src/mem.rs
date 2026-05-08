use core::{
    alloc::{GlobalAlloc, Layout},
    cell::UnsafeCell,
    ptr,
};

use pyr_alloc::{
    bump::{BumpHeap, HeapStats},
    context::PyrContext,
    root::PyrAllocator,
    sync::SpinLock,
};
use pyr_arch::{addr::PhysAddr, boot::info::BootInfo};

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

struct GlobalAllocatorCell(UnsafeCell<PyrAllocator>);

// SAFETY:
// Access is manually restricted to single-core early boot.
// No concurrent access is allowed before the returned PyrContext owns
// the allocator borrow for the rest of boot.
unsafe impl Sync for GlobalAllocatorCell {}

static PYR_ALLOC: GlobalAllocatorCell =
    GlobalAllocatorCell(UnsafeCell::new(PyrAllocator::uninit()));

/// Initialize Pyr's physical frame allocator from BootInfo.
///
/// # Safety
///
/// Caller must guarantee:
///
/// - called exactly once
/// - called during single-core early boot
/// - interrupts cannot access the allocator
/// - FramePool is valid writable memory owned by Pyr
/// - FramePool does not overlap image, stack, heap, boot resources, MMIO, or firmware memory
pub unsafe fn init_frame_allocator<'a>(
    boot_info: &BootInfo<'a>,
) -> PyrContext<'static, PyrAllocator> {
    // SAFETY:
    // PYR_ALLOC is only accessed here during single-core boot. The caller
    // guarantees one-time initialization and exclusive ownership.
    unsafe {
        let pool = boot_info
            .frame_pool()
            .unwrap_or_else(|| fatal!("BootInfo missing FramePool region"));

        let alloc = &mut *PYR_ALLOC.0.get();

        alloc
            .init_frame_pool(pool.start, pool.len)
            .unwrap_or_else(|err| {
                fatal!("frame allocator init failed: {err:?}")
            });
        PyrContext::new(alloc)
    }
}
