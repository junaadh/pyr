use crate::{guest::region::GuestRegion, stage2::Stage2Vm};
use pyr_alloc::{
    context::PyrContext, guest_ram::GuestRam, traits::PageAllocator,
};
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
    OutOfBounds,
}

pub struct GuestMemory;

impl GuestMemory {
    pub const GUEST_RAM_IPA: IpaAddr = IpaAddr::new(0x4000_0000);
    pub const GUEST_RAM_SIZE: usize = 128 * 1024 * 1024;

    pub const KERNEL_LOAD_IPA: IpaAddr = IpaAddr::new(0x4000_0000);

    pub const DTB_IPA: IpaAddr = IpaAddr::new(0x47c0_0000);
    pub const DTB_MAX_SIZE: usize = 4 * 1024 * 1024;

    pub const STACK_TOP_IPA: IpaAddr = IpaAddr::new(0x47b0_0000);

    pub const INITRD_IPA: IpaAddr = IpaAddr::new(0x4600_0000);
    pub const INITRD_MAX_SIZE: usize = 16 * 1024 * 1024;

    pub fn ram_window(ram: &GuestRam) -> GuestRegion {
        GuestRegion::ram(Self::GUEST_RAM_IPA, ram.base(), ram.size() as usize)
    }

    pub fn load_kernel(
        ram: &GuestRam,
        image: &[u8],
    ) -> Result<GuestRegion, GuestMemoryError> {
        if image.len() > ram.size() as usize {
            return Err(GuestMemoryError::ImageTooLarge);
        }

        Self::copy_into_guest_ram(ram, Self::KERNEL_LOAD_IPA, image)?;

        Ok(GuestRegion::ram(
            Self::KERNEL_LOAD_IPA,
            Self::host_pa_for_ipa(ram, Self::KERNEL_LOAD_IPA)?,
            image.len(),
        ))
    }

    pub fn load_image(
        ram: &GuestRam,
        image: &[u8],
    ) -> Result<GuestRegion, GuestMemoryError> {
        Self::load_kernel(ram, image)
    }

    pub fn load_dtb(
        ram: &GuestRam,
        dtb: &[u8],
    ) -> Result<GuestRegion, GuestMemoryError> {
        if dtb.len() > Self::DTB_MAX_SIZE {
            return Err(GuestMemoryError::DtbTooLarge);
        }

        Self::copy_into_guest_ram(ram, Self::DTB_IPA, dtb)?;

        Ok(GuestRegion::ram(
            Self::DTB_IPA,
            Self::host_pa_for_ipa(ram, Self::DTB_IPA)?,
            dtb.len(),
        ))
    }

    pub fn load_initrd(
        ram: &GuestRam,
        initrd: &[u8],
    ) -> Result<GuestRegion, GuestMemoryError> {
        if initrd.len() > Self::INITRD_MAX_SIZE {
            return Err(GuestMemoryError::InitrdTooLarge);
        }

        Self::copy_into_guest_ram(ram, Self::INITRD_IPA, initrd)?;

        Ok(GuestRegion::ram(
            Self::INITRD_IPA,
            Self::host_pa_for_ipa(ram, Self::INITRD_IPA)?,
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
        ram: &GuestRam,
        dst_ipa: IpaAddr,
        src: &[u8],
    ) -> Result<(), GuestMemoryError> {
        let dst_pa = Self::host_pa_for_ipa_len(ram, dst_ipa, src.len())?;

        // SAFETY:
        //
        // - `src.as_ptr()` is valid for `src.len()` bytes.
        // - `dst_pa` points inside allocator-owned guest RAM.
        // - guest RAM is exclusively owned by this guest boot path.
        // - boot resource memory does not overlap guest RAM.
        // - current QEMU bare path gives EL2 identity access to this PA.
        unsafe {
            core::ptr::copy_nonoverlapping(
                src.as_ptr(),
                dst_pa.as_mut_ptr(),
                src.len(),
            );
        }

        Ok(())
    }

    fn guest_ram_offset(
        dst_ipa: IpaAddr,
        len: usize,
    ) -> Result<u64, GuestMemoryError> {
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

        Ok(offset)
    }

    fn host_pa_for_ipa(
        ram: &GuestRam,
        ipa: IpaAddr,
    ) -> Result<PhysAddr, GuestMemoryError> {
        Self::host_pa_for_ipa_len(ram, ipa, 1)
    }

    fn host_pa_for_ipa_len(
        ram: &GuestRam,
        ipa: IpaAddr,
        len: usize,
    ) -> Result<PhysAddr, GuestMemoryError> {
        let offset = Self::guest_ram_offset(ipa, len)?;

        if offset
            .checked_add(len as u64)
            .ok_or(GuestMemoryError::RegionOutOfGuestRam)?
            > ram.size()
        {
            return Err(GuestMemoryError::RegionOutOfGuestRam);
        }

        let pa = ram
            .base()
            .as_u64()
            .checked_add(offset)
            .ok_or(GuestMemoryError::RegionOutOfGuestRam)?;

        Ok(PhysAddr::new(pa))
    }
}

pub trait MapGuestRegion<A: PageAllocator> {
    fn map_guest_region(
        &mut self,
        cx: &mut PyrContext<A>,
        region: GuestRegion,
    ) -> Result<(), MapError>;
}
