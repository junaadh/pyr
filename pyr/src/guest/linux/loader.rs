use pyr_arch::addr::PhysAddr;

use crate::{
    guest::{
        linux::{
            boot_config::LinuxBootConfig,
            header::{LinuxImageError, LinuxImageHeader},
        },
        memory::{GuestMemory, GuestMemoryError},
        region::GuestRegion,
    },
    log,
    stage2::scratch,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinuxLoadError {
    Image(LinuxImageError),
    Memory(GuestMemoryError),
}

pub struct LoadedLinux {
    pub image: GuestRegion,
    pub boot: LinuxBootConfig,
}

impl LoadedLinux {
    pub const fn image_region(&self) -> GuestRegion {
        self.image
    }

    pub const fn boot_config(&self) -> LinuxBootConfig {
        self.boot
    }
}

pub fn load_linux_image(image: &[u8]) -> Result<LoadedLinux, LinuxLoadError> {
    let header =
        LinuxImageHeader::parse(image).map_err(LinuxLoadError::Image)?;

    let scratch = scratch::get_mut();

    if image.len() > scratch.guest_ram.len() {
        return Err(LinuxLoadError::Memory(GuestMemoryError::ImageTooLarge));
    }

    // SAFETY:
    // - `image.as_ptr()` is valid for `image.len()` bytes.
    // - destination is Pyr-owned guest RAM scratch storage.
    // - source slice and guest RAM scratch do not overlap.
    unsafe {
        core::ptr::copy_nonoverlapping(
            image.as_ptr(),
            scratch.guest_ram.as_mut_ptr(),
            image.len(),
        );
    }

    let image_region = GuestRegion::ram(
        GuestMemory::KERNEL_LOAD_IPA,
        PhysAddr::new(scratch::guest_ram_base()),
        image.len(),
    );

    let boot = LinuxBootConfig::new(
        header.entry_ipa(GuestMemory::KERNEL_LOAD_IPA),
        GuestMemory::DTB_IPA,
        GuestMemory::stack_top_ipa(),
    );

    log!("linux text_offset = {:#x}", header.text_offset());
    log!("linux image_size  = {:#x}", header.image_size());
    log!(
        "linux entry IPA   = {:#018x}",
        header.entry_ipa(GuestMemory::KERNEL_LOAD_IPA).as_u64()
    );

    Ok(LoadedLinux {
        image: image_region,

        boot,
    })
}
