use crate::guest::{
    linux::header::{LinuxImageError, LinuxImageHeader},
    memory::{GuestMemory, GuestMemoryError},
    region::GuestRegion,
};
use pyr_alloc::guest_ram::GuestRam;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinuxLoadError {
    Image(LinuxImageError),
    Memory(GuestMemoryError),
    GuestMemory(GuestMemoryError),
}

pub struct LoadedLinux {
    pub image: GuestRegion,
    pub header: LinuxImageHeader,
}

pub fn load_linux_image(
    image: &[u8],
    ram: &GuestRam,
) -> Result<LoadedLinux, LinuxLoadError> {
    let header =
        LinuxImageHeader::parse(image).map_err(LinuxLoadError::Image)?;

    let image_region =
        GuestMemory::load_kernel(ram, image).map_err(LinuxLoadError::Memory)?;

    Ok(LoadedLinux {
        image: image_region,
        header,
    })
}
