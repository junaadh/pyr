#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(transparent)]
pub struct ElrEl2(u64);

impl ElrEl2 {
    #[inline(always)]
    pub const fn new(addr: u64) -> Self {
        Self(addr)
    }

    #[inline(always)]
    pub fn mrs() -> Self {
        let value: u64;

        // SAFETY: Caller must execute at EL2 or higher.
        unsafe {
            core::arch::asm!(
                "mrs {out}, elr_el2",
                out = out(reg) value,
                options(nomem, nostack)
            );
        }

        Self(value)
    }

    #[inline(always)]
    pub fn msr(self) {
        // SAFETY: Caller must execute at EL2 or higher.
        unsafe {
            core::arch::asm!(
                "msr elr_el2, {value}",
                value = in(reg) self.0,
                options(nomem, nostack)
            );
        }
    }

    #[inline(always)]
    pub const fn raw(self) -> u64 {
        self.0
    }
}
