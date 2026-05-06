use pyr_arch::addr::{IpaAddr, PhysAddr};

#[derive(Debug)]
pub struct GuestMemory {
    pub host_pa: PhysAddr,
    pub guest_ipa: IpaAddr,
    pub size: usize,
}
