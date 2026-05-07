use crate::{
    guest::{
        linux::{
            boot_config::LinuxBootConfig,
            dtb::{DtbLoadError, LoadedDtb, load_dtb_blob},
            loader::{LinuxLoadError, LoadedLinux, load_linux_image},
        },
        memory::{GuestMemory, MapGuestRegion},
    },
    stage2::Stage2Vm,
};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum LinuxBootLoadError {
    Image(LinuxLoadError),
    Dtb(DtbLoadError),
}

pub struct LoadedLinuxBoot {
    linux: LoadedLinux,
    dtb: LoadedDtb,
}

impl LoadedLinuxBoot {
    pub const fn linux(&self) -> &LoadedLinux {
        &self.linux
    }

    pub const fn dtb(&self) -> &LoadedDtb {
        &self.dtb
    }

    pub const fn boot_config(&self) -> LinuxBootConfig {
        self.linux.boot_config()
    }

    pub fn map_into<S>(&self, stage2: &mut Stage2Vm<S>)
    where
        Stage2Vm<S>: MapGuestRegion,
    {
        GuestMemory::map_region(stage2, GuestMemory::ram_window());
    }
}

pub fn load_linux_boot(
    image: &[u8],
    dtb: &[u8],
) -> Result<LoadedLinuxBoot, LinuxBootLoadError> {
    let linux = load_linux_image(image).map_err(LinuxBootLoadError::Image)?;
    let dtb = load_dtb_blob(dtb).map_err(LinuxBootLoadError::Dtb)?;

    Ok(LoadedLinuxBoot { linux, dtb })
}
