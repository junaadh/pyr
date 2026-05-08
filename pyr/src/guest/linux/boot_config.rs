use pyr_arch::boot::info::{BootInfo, BootResource};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinuxBootConfigError {
    MissingKernel,
    MissingDtb,
}

pub struct LinuxBootConfig<'a> {
    pub kernel: BootResource<'a>,
    pub dtb: BootResource<'a>,
    pub initrd: Option<BootResource<'a>>,
}

impl<'a> LinuxBootConfig<'a> {
    pub fn from_dev_boot_resources(
        boot_info: &'a BootInfo<'a>,
    ) -> Result<Self, LinuxBootConfigError> {
        let resources = boot_info.resources();

        let kernel = resources
            .named("dev-linux-kernel")
            .ok_or(LinuxBootConfigError::MissingKernel)?;

        let dtb = resources
            .named("dev-linux-dtb")
            .ok_or(LinuxBootConfigError::MissingDtb)?;

        let initrd = resources.named("dev-linux-initrd");

        Ok(Self {
            kernel,
            dtb,
            initrd,
        })
    }
}
