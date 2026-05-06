#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(transparent)]
pub struct SctlrEl2(u64);

impl SctlrEl2 {
    #[inline(always)]
    pub fn mrs() -> Self {
        let value: u64;

        // SAFETY: Caller must execute at EL2 or higher.
        unsafe {
            core::arch::asm!(
                "mrs {out}, sctlr_el2",
                out = out(reg) value,
                options(nomem, nostack)
            );
        }

        Self(value)
    }

    #[inline(always)]
    pub fn msr(self) {
        // SAFETY: Caller must execute at EL2 or higher and provide a valid SCTLR_EL2 value.
        unsafe {
            core::arch::asm!(
                "msr sctlr_el2, {value}",
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
    pub const fn mmu_enabled(self) -> bool {
        self.0 & 1 != 0
    }

    #[inline(always)]
    pub const fn caches_enabled(self) -> bool {
        self.0 & (1 << 2) != 0
    }
}
