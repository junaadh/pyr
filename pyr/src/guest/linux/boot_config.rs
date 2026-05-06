use crate::guest::config::GuestConfig;
use pyr_arch::addr::IpaAddr;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct LinuxBootConfig {
    pub kernel_entry: IpaAddr,
    pub dtb: IpaAddr,
    pub stack_top: u64,
}

impl LinuxBootConfig {
    pub const fn new(
        kernel_entry: IpaAddr,
        dtb: IpaAddr,
        stack_top: u64,
    ) -> Self {
        Self {
            kernel_entry,
            dtb,
            stack_top,
        }
    }

    pub const fn guest_config(self) -> GuestConfig {
        GuestConfig::new(self.kernel_entry.as_u64(), self.stack_top)
            .with_x0(self.dtb.as_u64())
            .with_x1(0)
            .with_x2(0)
            .with_x3(0)
    }
}
