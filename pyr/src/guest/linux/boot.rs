use pyr_alloc::{context::PyrContext, traits::PageAllocator};
use pyr_arch::boot::info::BootResource;

use crate::{
    guest::{
        config::GuestConfig,
        linux::{
            boot_config::LinuxBootConfig,
            dtb::{DtbLoadError, load_dtb_blob},
            initrd::{InitrdLoadError, load_initrd_blob},
            loader::{LinuxLoadError, load_linux_image},
        },
        memory::{GuestMemory, MapGuestRegion},
        region::GuestRegion,
    },
    stage2::Stage2Vm,
};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum LinuxBootLoadError {
    Image(LinuxLoadError),
    Dtb(DtbLoadError),
    Initrd(InitrdLoadError),
}

pub struct LoadedLinuxBoot<'a> {
    pub linux: GuestRegion,
    pub dtb: GuestRegion,
    pub initrd: Option<GuestRegion>,
    pub boot: LinuxBootConfig<'a>,
    pub guest: GuestConfig,
}

impl<'a> LoadedLinuxBoot<'a> {
    pub const fn boot_config(&self) -> &LinuxBootConfig<'a> {
        &self.boot
    }

    pub const fn guest_config(&self) -> GuestConfig {
        self.guest
    }

    pub fn map_into<A, S>(
        &self,
        cx: &mut PyrContext<A>,
        stage2: &mut Stage2Vm<S>,
    ) -> Result<(), pyr_arch::page_table::MapError>
    where
        A: PageAllocator,
        Stage2Vm<S>: MapGuestRegion<A>,
    {
        GuestMemory::map_region(cx, stage2, GuestMemory::ram_window())?;

        Ok(())
    }
}

pub fn load_linux_boot<'a>(
    image: BootResource<'a>,
    dtb: BootResource<'a>,
    initrd: Option<BootResource<'a>>,
) -> Result<LoadedLinuxBoot<'a>, LinuxBootLoadError> {
    let linux =
        load_linux_image(image.data()).map_err(LinuxBootLoadError::Image)?;
    let loaded_dtb =
        load_dtb_blob(dtb.data()).map_err(LinuxBootLoadError::Dtb)?;
    let loaded_initrd = match &initrd {
        Some(slice) => Some(
            load_initrd_blob(slice.data())
                .map_err(LinuxBootLoadError::Initrd)?
                .region(),
        ),
        None => None,
    };

    let boot = LinuxBootConfig {
        kernel: image,
        dtb,
        initrd,
    };

    let guest = GuestConfig::new(
        linux.image.ipa().as_u64(),
        GuestMemory::stack_top_ipa(),
    )
    .with_x0(loaded_dtb.region().ipa().as_u64());

    Ok(LoadedLinuxBoot {
        linux: linux.image,
        dtb: loaded_dtb.region(),
        initrd: loaded_initrd,
        boot,
        guest,
    })
}
