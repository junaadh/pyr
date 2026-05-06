use crate::addr::PhysAddr;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(transparent)]
pub struct Ttbr0El1(u64);

impl Ttbr0El1 {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn from_baddr(base: PhysAddr) -> Self {
        Self(base.as_u64())
    }

    pub fn msr(self) {
        // SAFETY: Caller must execute at EL2 or higher.
        unsafe {
            core::arch::asm!(
                "msr ttbr0_el1, {value}",
                value = in(reg) self.0,
                options(nomem, nostack)
            );
        }
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}
