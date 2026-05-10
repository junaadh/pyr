#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(transparent)]
pub struct HcrEl2(u64);

impl HcrEl2 {
    #[inline(always)]
    pub fn mrs() -> Self {
        let value: u64;

        // SAFETY: Caller must execute at EL2 or higher.
        unsafe {
            core::arch::asm!(
                "mrs {out}, hcr_el2",
                out = out(reg) value,
                options(nomem, nostack)
            );
        }

        Self(value)
    }

    #[inline(always)]
    pub fn msr(self) {
        // SAFETY: Caller must execute at EL2 or higher and provide a valid HCR_EL2 value.
        unsafe {
            core::arch::asm!(
                "msr hcr_el2, {value}",
                value = in(reg) self.0,
                options(nomem, nostack)
            );
        }
    }

    #[inline(always)]
    pub const fn raw(self) -> u64 {
        self.0
    }

    #[inline(always)]
    pub const fn with_rw(self) -> Self {
        Self(self.0 | (1 << 31))
    }

    #[inline(always)]
    pub const fn with_vm(self) -> Self {
        Self(self.0 | (1 << 0))
    }

    #[inline(always)]
    pub const fn with_twi(self) -> Self {
        Self(self.0 | (1 << 13))
    }

    #[inline(always)]
    pub const fn with_twe(self) -> Self {
        Self(self.0 | (1 << 14))
    }

    #[inline(always)]
    pub const fn empty() -> Self {
        Self(0)
    }

    #[inline(always)]
    pub const fn without_tge(self) -> Self {
        Self(self.0 & !(1 << 27))
    }

    #[inline(always)]
    pub const fn without_e2h(self) -> Self {
        Self(self.0 & !(1 << 34))
    }

    #[inline(always)]
    pub const fn with_amo(self) -> Self {
        Self(self.0 | (1 << 5))
    }

    #[inline(always)]
    pub const fn with_imo(self) -> Self {
        Self(self.0 | (1 << 4))
    }

    #[inline(always)]
    pub const fn with_fmo(self) -> Self {
        Self(self.0 | (1 << 3))
    }

    pub const fn with_vi(self) -> Self {
        Self(self.0 | (1 << 7))
    }

    pub const fn without_vi(self) -> Self {
        Self(self.0 & !(1 << 7))
    }
}
