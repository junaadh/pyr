#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(transparent)]
pub struct CntvoffEl2(u64);

impl CntvoffEl2 {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }

    pub fn mrs() -> Self {
        let value: u64;

        // SAFETY: Reading CNTVOFF_EL2 is valid while executing at EL2.
        unsafe {
            core::arch::asm!("mrs {value}, cntvoff_el2", value = out(reg) value);
        }

        Self(value)
    }

    pub fn msr(self) {
        // SAFETY: Writing CNTVOFF_EL2 is valid while executing at EL2.
        unsafe {
            core::arch::asm!("msr cntvoff_el2, {value}", value = in(reg) self.0);
        }
    }
}
