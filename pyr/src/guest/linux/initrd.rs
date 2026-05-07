use crate::guest::{
    memory::{GuestMemory, GuestMemoryError},
    region::GuestRegion,
};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum InitrdLoadError {
    TooLarge,
    OutOfGuestRam,
}

pub struct LoadedInitrd {
    region: GuestRegion,
}

impl LoadedInitrd {
    pub const fn region(&self) -> GuestRegion {
        self.region
    }

    pub const fn start_ipa(&self) -> u64 {
        self.region.ipa().as_u64()
    }

    pub const fn end_ipa(&self) -> u64 {
        self.region.ipa().as_u64() + self.region.size() as u64
    }
}

pub fn load_initrd_blob(
    initrd: &[u8],
) -> Result<LoadedInitrd, InitrdLoadError> {
    let region = GuestMemory::load_initrd(initrd).map_err(|err| match err {
        GuestMemoryError::InitrdTooLarge => InitrdLoadError::TooLarge,

        GuestMemoryError::RegionOutOfGuestRam
        | GuestMemoryError::ImageTooLarge
        | GuestMemoryError::DtbTooLarge => InitrdLoadError::OutOfGuestRam,
    })?;

    Ok(LoadedInitrd { region })
}
