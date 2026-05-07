use crate::guest::{
    memory::{GuestMemory, GuestMemoryError},
    region::GuestRegion,
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

pub fn load_dtb_blob(dtb: &[u8]) -> Result<LoadedDtb, DtbLoadError> {
    let region = GuestMemory::load_dtb(dtb).map_err(|err| match err {
        GuestMemoryError::ImageTooLarge
        | GuestMemoryError::DtbTooLarge
        | GuestMemoryError::RegionOutOfGuestRam => DtbLoadError::TooLarge,
    })?;

    Ok(LoadedDtb { region })
}
