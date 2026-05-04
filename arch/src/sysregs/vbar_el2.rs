use crate::addr::PhysAddr;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(transparent)]
pub struct VbarEl2(u64);

impl VbarEl2 {
    #[inline(always)]
    pub fn mrs() -> Self {
        let value: u64;

        // SAFETY: Caller must execute at EL2 or higher.
        unsafe {
            core::arch::asm!(
                "mrs {out}, vbar_el2",
                out = out(reg) value,
                options(nomem, nostack)
            );
        }

        Self(value)
    }

    #[inline(always)]
    pub fn msr(self) {
        // SAFETY: Caller must execute at EL2 or higher. Value must be correctly aligned.
        unsafe {
            core::arch::asm!(
                "msr vbar_el2, {value}",
                value = in(reg) self.0,
                options(nomem, nostack)
            );
        }
    }

    #[inline(always)]
    pub const fn from_phys(addr: PhysAddr) -> Self {
        Self(addr.as_u64())
    }

    #[inline(always)]
    pub const fn raw(self) -> u64 {
        self.0
    }
}
