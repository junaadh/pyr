use crate::addr::PhysAddr;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(transparent)]
pub struct VttbrEl2(u64);

impl VttbrEl2 {
    pub const BADDR_MASK: u64 = 0x0000_ffff_ffff_f000;
    pub const VMID_SHIFT: u64 = 48;

    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn from_baddr(base: PhysAddr) -> Self {
        Self(base.as_u64() & Self::BADDR_MASK)
    }

    pub const fn from_vmid_baddr(vmid: u16, base: PhysAddr) -> Self {
        let vmid = (vmid as u64) << Self::VMID_SHIFT;
        let baddr = base.as_u64() & Self::BADDR_MASK;

        Self(vmid | baddr)
    }

    pub fn mrs() -> Self {
        let value: u64;

        // SAFETY: Caller must execute at EL2 or higher.
        unsafe {
            core::arch::asm!(
                "mrs {out}, vttbr_el2",
                out = out(reg) value,
                options(nomem, nostack)
            );
        }

        Self(value)
    }

    pub fn msr(self) {
        // SAFETY: Caller must execute at EL2 or higher and provide a valid VTTBR_EL2 value.
        unsafe {
            core::arch::asm!(
                "msr vttbr_el2, {value}",
                value = in(reg) self.0,
                options(nomem, nostack)
            );
        }
    }

    pub const fn raw(self) -> u64 {
        self.0
    }

    pub const fn vmid(self) -> u16 {
        (self.0 >> Self::VMID_SHIFT) as u16
    }

    pub const fn baddr(self) -> PhysAddr {
        PhysAddr::new(self.0 & Self::BADDR_MASK)
    }
}
