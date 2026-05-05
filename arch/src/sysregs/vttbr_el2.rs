use crate::addr::PhysAddr;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(transparent)]
pub struct VttbrEl2(u64);

impl VttbrEl2 {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn from_baddr(base: PhysAddr) -> Self {
        Self(base.as_u64())
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
}
