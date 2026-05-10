#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(transparent)]
pub struct CnthctlEl2(u64);

const EL1PCTEN: u64 = 1 << 0;
const EL1PCEN: u64 = 1 << 1;
const EL1TVT: u64 = 1 << 13;
const EL1TVCT: u64 = 1 << 14;

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

    /// EL1 physical timer register access enable.
    pub const fn with_el1pcen(self) -> Self {
        Self(self.0 | EL1PCEN)
    }

    /// Trap EL1 physical timer register accesses to EL2.
    pub const fn without_el1pcen(self) -> Self {
        Self(self.0 & !EL1PCEN)
    }

    /// EL1 physical counter register access enable.
    pub const fn with_el1pcten(self) -> Self {
        Self(self.0 | EL1PCTEN)
    }

    /// Trap EL1 physical counter register accesses to EL2.
    pub const fn without_el1pcten(self) -> Self {
        Self(self.0 & !EL1PCTEN)
    }

    /// Trap EL1 virtual timer register accesses to EL2.
    pub const fn with_el1tvt(self) -> Self {
        Self(self.0 | EL1TVT)
    }

    /// Do not trap EL1 virtual timer register accesses to EL2.
    pub const fn without_el1tvt(self) -> Self {
        Self(self.0 & !EL1TVT)
    }

    /// Trap EL1 virtual counter register accesses to EL2.
    pub const fn with_el1tvct(self) -> Self {
        Self(self.0 | EL1TVCT)
    }

    /// Do not trap EL1 virtual counter register accesses to EL2.
    pub const fn without_el1tvct(self) -> Self {
        Self(self.0 & !EL1TVCT)
    }

    /// EL1 virtual timer register access enable.
    pub const fn with_el1vten(self) -> Self {
        self.without_el1tvt()
    }

    /// Trap EL1 virtual timer register accesses to EL2.
    pub const fn without_el1vten(self) -> Self {
        self.with_el1tvt()
    }

    /// EL1 virtual counter register access enable.
    pub const fn with_el1vcten(self) -> Self {
        self.without_el1tvct()
    }

    /// Trap EL1 virtual counter register accesses to EL2.
    pub const fn without_el1vcten(self) -> Self {
        self.with_el1tvct()
    }
}
