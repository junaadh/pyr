#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(transparent)]
pub struct CnthctlEl2(u64);

impl CnthctlEl2 {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }

    pub fn mrs() -> Self {
        let value: u64;

        // SAFETY: Reading CNTHCTL_EL2 is valid while executing at EL2.
        unsafe {
            core::arch::asm!("mrs {value}, cnthctl_el2", value = out(reg) value);
        }

        Self(value)
    }

    pub fn msr(self) {
        // SAFETY: Writing CNTHCTL_EL2 is valid while executing at EL2.
        unsafe {
            core::arch::asm!("msr cnthctl_el2, {value}", value = in(reg) self.0);
        }
    }

    /// EL1 physical counter/timer register access enable.
    pub const fn with_el1pcen(self) -> Self {
        Self(self.0 | (1 << 1))
    }

    /// EL1 physical counter register access enable.
    pub const fn with_el1pcten(self) -> Self {
        Self(self.0 | (1 << 0))
    }
}
