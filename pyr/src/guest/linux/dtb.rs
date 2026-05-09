use pyr_alloc::guest_ram::GuestRam;

use crate::{
    guest::{
        memory::{GuestMemory, GuestMemoryError},
        region::GuestRegion,
    },
    log,
};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum DtbLoadError {
    TooLarge,
}

pub struct LoadedDtb {
    region: GuestRegion,
}

impl LoadedDtb {
    pub const fn region(&self) -> GuestRegion {
        self.region
    }
}

pub fn load_dtb_blob(
    dtb: &[u8],
    ram: &GuestRam,
) -> Result<LoadedDtb, DtbLoadError> {
    let region = GuestMemory::load_dtb(ram, dtb).map_err(|err| {
        log!("Failed to load dtb: {err:?}");
        match err {
            GuestMemoryError::ImageTooLarge
            | GuestMemoryError::DtbTooLarge
            | GuestMemoryError::RegionOutOfGuestRam
            | GuestMemoryError::OutOfBounds
            | GuestMemoryError::InitrdTooLarge => DtbLoadError::TooLarge,
        }
    })?;

    Ok(LoadedDtb { region })
}
