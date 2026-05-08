use crate::{
    guest::region::GuestRegion,
    stage2::{Stage2Vm, scratch},
};
use pyr_alloc::{context::PyrContext, traits::PageAllocator};
use pyr_arch::{
    addr::{IpaAddr, PhysAddr},
    page_table::MapError,
};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum GuestMemoryError {
    ImageTooLarge,
    DtbTooLarge,
    RegionOutOfGuestRam,
    InitrdTooLarge,
}

pub struct GuestMemory;

impl GuestMemory {
    pub const GUEST_RAM_IPA: IpaAddr = IpaAddr::new(0x4000_0000);
    pub const GUEST_RAM_SIZE: usize = 1024 * 1024 * 1024;

    pub const KERNEL_LOAD_IPA: IpaAddr = IpaAddr::new(0x4000_0000);
    pub const DTB_IPA: IpaAddr = IpaAddr::new(0x7f00_0000);
    pub const STACK_TOP_IPA: IpaAddr = IpaAddr::new(0x7ff0_0000);

    pub const INITRD_IPA: IpaAddr = IpaAddr::new(0x7800_0000);
    pub const INITRD_MAX_SIZE: usize = 16 * 1024 * 1024;

    pub fn ram_window() -> GuestRegion {
        GuestRegion::ram(
            Self::GUEST_RAM_IPA,
            PhysAddr::new(scratch::guest_ram_base()),
            Self::GUEST_RAM_SIZE,
        )
    }

    pub fn load_kernel(image: &[u8]) -> Result<GuestRegion, GuestMemoryError> {
        Self::copy_into_guest_ram(Self::KERNEL_LOAD_IPA, image)?;

        Ok(GuestRegion::ram(
            Self::KERNEL_LOAD_IPA,
            Self::host_pa_for_ipa(Self::KERNEL_LOAD_IPA)?,
            image.len(),
        ))
    }

    pub fn load_image(image: &[u8]) -> Result<GuestRegion, GuestMemoryError> {
        Self::load_kernel(image)
    }

    pub fn load_dtb(dtb: &[u8]) -> Result<GuestRegion, GuestMemoryError> {
        if dtb.len() > 64 * 1024 {
            return Err(GuestMemoryError::DtbTooLarge);
        }

        Self::copy_into_guest_ram(Self::DTB_IPA, dtb)?;

        Ok(GuestRegion::ram(
            Self::DTB_IPA,
            Self::host_pa_for_ipa(Self::DTB_IPA)?,
            dtb.len(),
        ))
    }

    pub fn load_initrd(initrd: &[u8]) -> Result<GuestRegion, GuestMemoryError> {
        if initrd.len() > Self::INITRD_MAX_SIZE {
            return Err(GuestMemoryError::InitrdTooLarge);
        }

        Self::copy_into_guest_ram(Self::INITRD_IPA, initrd)?;

        Ok(GuestRegion::ram(
            Self::INITRD_IPA,
            Self::host_pa_for_ipa(Self::INITRD_IPA)?,
            initrd.len(),
        ))
    }

    pub fn stack_top_ipa() -> u64 {
        Self::STACK_TOP_IPA.as_u64()
    }

    pub fn map_region<A, S>(
        cx: &mut PyrContext<A>,
        stage2: &mut Stage2Vm<S>,
        region: GuestRegion,
    ) -> Result<(), MapError>
    where
        A: PageAllocator,
        Stage2Vm<S>: MapGuestRegion<A>,
    {
        stage2.map_guest_region(cx, region)
    }

    fn copy_into_guest_ram(
        dst_ipa: IpaAddr,
        src: &[u8],
    ) -> Result<(), GuestMemoryError> {
        let offset = Self::guest_ram_offset(dst_ipa, src.len())?;
        let scratch = scratch::get_mut();

        if scratch.guest_ram.len() < Self::GUEST_RAM_SIZE {
            return Err(GuestMemoryError::RegionOutOfGuestRam);
        }

        if let Some(slice) =
            scratch.guest_ram.get_mut(offset..offset + src.len())
        {
            slice.copy_from_slice(src);
        }

        Ok(())
    }

    fn guest_ram_offset(
        dst_ipa: IpaAddr,
        len: usize,
    ) -> Result<usize, GuestMemoryError> {
        let base = Self::GUEST_RAM_IPA.as_u64();
        let dst = dst_ipa.as_u64();

        if dst < base {
            return Err(GuestMemoryError::RegionOutOfGuestRam);
        }

        let offset = dst - base;
        let end = offset
            .checked_add(len as u64)
            .ok_or(GuestMemoryError::RegionOutOfGuestRam)?;

        if end > Self::GUEST_RAM_SIZE as u64 {
            return Err(GuestMemoryError::RegionOutOfGuestRam);
        }

        Ok(offset as usize)
    }

    fn host_pa_for_ipa(ipa: IpaAddr) -> Result<PhysAddr, GuestMemoryError> {
        let offset = Self::guest_ram_offset(ipa, 1)?;
        Ok(PhysAddr::new(scratch::guest_ram_base() + offset as u64))
    }
}

pub trait MapGuestRegion<A: PageAllocator> {
    fn map_guest_region(
        &mut self,
        cx: &mut PyrContext<A>,
        region: GuestRegion,
    ) -> Result<(), MapError>;
}
