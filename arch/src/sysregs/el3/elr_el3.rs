#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(transparent)]
pub struct ElrEl3(u64);

impl ElrEl3 {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn msr(self) {
        // SAFETY: Caller must execute at EL3. ELR_EL3 receives the exception-return target address.
        unsafe {
            core::arch::asm!(
                "msr elr_el3, {value}",
                value = in(reg) self.0,
                options(nomem, nostack)
            );
        }
    }

    pub fn mrs() -> Self {
        let value: u64;

        // SAFETY: Caller must execute at EL3 or higher privilege context where ELR_EL3 is accessible.
        unsafe {
            core::arch::asm!(
                "mrs {out}, elr_el3",
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
