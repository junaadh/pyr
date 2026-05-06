use crate::addr::PhysAddr;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(transparent)]
pub struct VbarEl1(u64);

impl VbarEl1 {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn from_phys(addr: PhysAddr) -> Self {
        Self(addr.as_u64())
    }

    pub fn msr(self) {
        // SAFETY: Caller must execute at EL2 or higher. Address must be valid and aligned if EL1 exceptions are enabled.
        unsafe {
            core::arch::asm!(
                "msr vbar_el1, {value}",
                value = in(reg) self.0,
                options(nomem, nostack)
            );
        }
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}
