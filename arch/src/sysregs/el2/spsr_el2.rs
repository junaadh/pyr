#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(transparent)]
pub struct SpsrEl2(u64);

impl SpsrEl2 {
    #[inline(always)]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    #[inline(always)]
    pub fn mrs() -> Self {
        let value: u64;

        // SAFETY: Caller must execute at EL2 or higher.
        unsafe {
            core::arch::asm!(
                "mrs {out}, spsr_el2",
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
                "msr spsr_el2, {value}",
                value = in(reg) self.0,
                options(nomem, nostack)
            );
        }
    }

    #[inline(always)]
    pub const fn raw(self) -> u64 {
        self.0
    }

    /// Return to EL1h with DAIF masked.
    ///
    /// - M[3:0] = 0b0101 = EL1h.
    /// - D/A/I/F bits masked = bits 9, 8, 7, 6.
    /// - bit 4 is nRW. 0 = AArch64, 1 = AArch32.
    #[inline(always)]
    pub const fn el1h_masked() -> Self {
        let m = 0b0101; // EL1h
        let daif = 0b1111 << 6; // mask D A I F

        Self(m | daif)
    }
}
