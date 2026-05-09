pub struct TpidrEl2;

impl TpidrEl2 {
    /// Reads `TPIDR_EL2`.
    ///
    /// # Safety
    ///
    /// Caller must ensure this executes at EL2.
    pub unsafe fn read_raw() -> u64 {
        let value: u64;

        // SAFETY: Caller must ensure this executes at EL2
        unsafe {
            core::arch::asm!(
                "mrs {value}, tpidr_el2",
                value = out(reg) value,
                options(nomem, nostack, preserves_flags),
            );
        }

        value
    }

    /// Writes `TPIDR_EL2`.
    ///
    /// # Safety
    ///
    /// Caller must ensure this executes at EL2 and `value` obeys the owner’s contract.
    pub unsafe fn write_raw(value: u64) {
        // SAFETY: Caller must ensure this executes at EL2
        unsafe {
            core::arch::asm!(
                "msr tpidr_el2, {value}",
                value = in(reg) value,
                options(nomem, nostack, preserves_flags),
            );
        }
    }
}
