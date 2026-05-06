#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(transparent)]
pub struct SpsrEl3(u64);

impl SpsrEl3 {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Return to EL2h, AArch64, all DAIF masked.
    ///
    /// M[3:0] = 0b1001 means EL2h.
    /// DAIF bits [9:6] mask debug, SError, IRQ, FIQ.
    pub const fn el2h_masked() -> Self {
        let mode_el2h = 0b1001;
        let daif_masked = 0b1111 << 6;

        Self(mode_el2h | daif_masked)
    }

    pub fn msr(self) {
        // SAFETY: Caller must execute at EL3. SPSR_EL3 controls the state restored by ERET.
        unsafe {
            core::arch::asm!(
                "msr spsr_el3, {value}",
                value = in(reg) self.0,
                options(nomem, nostack)
            );
        }
    }

    pub fn mrs() -> Self {
        let value: u64;

        // SAFETY: Caller must execute at EL3 or higher privilege context where SPSR_EL3 is accessible.
        unsafe {
            core::arch::asm!(
                "mrs {out}, spsr_el3",
                out = out(reg) value,
                options(nomem, nostack)
            );
        }

        Self(value)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}
