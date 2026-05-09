use pyr_alloc::{
    guest_ram::{GUEST_RAM_MIN_ALIGN, GuestRam},
    traits::{GuestRamAllocator, PageAllocator},
};
use pyr_arch::boot::info::BootResource;

use crate::{
    context::HypervisorContext,
    guest::{
        config::GuestConfig,
        linux::{
            boot_config::LinuxBootConfig,
            dtb::{DtbLoadError, load_dtb_blob},
            initrd::{InitrdLoadError, load_initrd_blob},
            loader::{LinuxLoadError, load_linux_image},
        },
        memory::{GuestMemory, GuestMemoryError, MapGuestRegion},
        region::GuestRegion,
    },
    log,
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
    pub ram: GuestRam,
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
        cx: &mut HypervisorContext<A>,
        stage2: &mut Stage2Vm<S>,
    ) -> Result<(), pyr_arch::page_table::MapError>
    where
        A: PageAllocator,
        Stage2Vm<S>: MapGuestRegion<A>,
    {
        GuestMemory::map_region(
            cx,
            stage2,
            GuestMemory::ram_window(&self.ram),
        )?;

        Ok(())
    }
}

pub fn load_linux_boot<'a, A>(
    cx: &mut HypervisorContext<A>,
    image: BootResource<'a>,
    dtb: BootResource<'a>,
    initrd: Option<BootResource<'a>>,
) -> Result<LoadedLinuxBoot<'a>, LinuxBootLoadError>
where
    A: PageAllocator + GuestRamAllocator,
{
    let ram = cx
        .mem
        .alloc_guest_ram(
            GuestMemory::GUEST_RAM_SIZE as u64,
            GUEST_RAM_MIN_ALIGN,
        )
        .map_err(|err| {
            crate::log!("Failed to allocate RAM for linux: {err:?}");
            LinuxBootLoadError::Image(LinuxLoadError::Memory(
                GuestMemoryError::OutOfBounds,
            ))
        })?;

    log!(
        "mem: guest_ram base={:#x} size={}",
        ram.base().as_u64(),
        ram.size(),
    );

    let linux = load_linux_image(image.data(), &ram)
        .map_err(LinuxBootLoadError::Image)?;

    log!(
        "linux: image ipa={:#x} pa={:#x} size={}",
        linux.image.ipa().as_u64(),
        linux.image.pa().as_u64(),
        linux.image.size()
    );

    let loaded_dtb =
        load_dtb_blob(dtb.data(), &ram).map_err(LinuxBootLoadError::Dtb)?;

    log!(
        "linux: dtb ipa={:#x} pa={:#x} size={}",
        loaded_dtb.region().ipa().as_u64(),
        loaded_dtb.region().pa().as_u64(),
        loaded_dtb.region().size()
    );

    let loaded_initrd = match &initrd {
        Some(slice) => {
            let tmp = Some(
                load_initrd_blob(slice.data(), &ram)
                    .map_err(LinuxBootLoadError::Initrd)?
                    .region(),
            );
            if let Some(initrd_gr) = tmp {
                log!(
                    "linux: initrd ipa={:#x} pa={:#x} size={}",
                    initrd_gr.ipa().as_u64(),
                    initrd_gr.pa().as_u64(),
                    initrd_gr.size()
                );
            }
            tmp
        }
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
        ram,
        dtb: loaded_dtb.region(),
        initrd: loaded_initrd,
        boot,
        guest,
    })
}
