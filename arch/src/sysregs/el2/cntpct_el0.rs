pub struct CntPctEl0;

impl CntPctEl0 {
    pub fn read() -> u64 {
        let value: u64;

        // SAFETY: CNTPCT_EL0 is readable at EL2 when architectural timer is present.
        unsafe {
            core::arch::asm!(
                "mrs {out}, cntpct_el0",
                out = out(reg) value,
                options(nomem, nostack, preserves_flags),
            );
        }

        value
    }
}
