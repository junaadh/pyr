use crate::{
    guest::region::GuestRegion,
    stage2::{Stage2Vm, scratch},
};
use core::ptr;
use pyr_arch::addr::{IpaAddr, PhysAddr};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum GuestMemoryError {
    ImageTooLarge,
}

/// Owns the guest memory layout decisions for the current VM.
///
/// This does not allocate memory yet. Backing storage currently comes from
/// `BootScratch`. Later this can become a real physical page allocator.
///
/// Invariant not caught by normal CI:
/// - every address handed to EL1 must be an IPA, never a host PA.
/// - every byte executed by EL1 must be backed by an explicit stage-2 mapping.
/// - MMIO interception depends on *not* mapping device IPAs like PL011.
pub struct GuestMemory;

impl GuestMemory {
    pub const ENTRY_IPA: IpaAddr = IpaAddr::new(0x4000_0000);
    pub const STACK_IPA: IpaAddr = IpaAddr::new(0x4002_0000);
    pub const STACK_SIZE: usize = 16 * 1024;

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn load_image(
        src: *const u8,
        len: usize,
    ) -> Result<GuestRegion, GuestMemoryError> {
        let scratch = scratch::get_mut();

        if len > scratch.guest_ram.len() {
            return Err(GuestMemoryError::ImageTooLarge);
        }

        // SAFETY:
        // - `src` points to an embedded guest payload range.
        // - destination is dedicated scratch guest RAM.
        // - caller provides `len` derived from start/end linker symbols.
        // - source and destination are distinct regions.
        unsafe {
            ptr::copy_nonoverlapping(src, scratch.guest_ram.as_mut_ptr(), len);
        }

        Ok(GuestRegion::ram(
            Self::ENTRY_IPA,
            PhysAddr::new(scratch::guest_ram_base()),
            len,
        ))
    }

    pub fn stack_region() -> GuestRegion {
        GuestRegion::ram(
            Self::STACK_IPA,
            PhysAddr::new(scratch::guest_stack_base()),
            Self::STACK_SIZE,
        )
    }

    pub fn stack_top_ipa() -> u64 {
        Self::STACK_IPA.as_u64() + Self::STACK_SIZE as u64
    }

    pub fn map_region<S>(stage2: &mut Stage2Vm<S>, region: GuestRegion)
    where
        Stage2Vm<S>: MapGuestRegion,
    {
        stage2.map_guest_region(region);
    }
}

pub trait MapGuestRegion {
    fn map_guest_region(&mut self, region: GuestRegion);
}
