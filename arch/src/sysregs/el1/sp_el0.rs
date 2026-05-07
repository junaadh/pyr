#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(transparent)]
pub struct SpEl0(u64);

impl SpEl0 {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }

    pub fn mrs() -> Self {
        let value: u64;

        // SAFETY: Reading SP_EL0 is valid from EL2.
        unsafe {
            core::arch::asm!("mrs {value}, sp_el0", value = out(reg) value);
        }

        Self(value)
    }

    pub fn msr(self) {
        // SAFETY: Writing SP_EL0 is valid from EL2 and prepares EL1t entry.
        unsafe {
            core::arch::asm!("msr sp_el0, {value}", value = in(reg) self.0);
        }
    }
}
