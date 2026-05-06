#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
#[repr(transparent)]
pub struct ScrEl3(u64);

impl ScrEl3 {
    pub const fn new() -> Self {
        Self(0)
    }

    pub const fn from_raw(value: u64) -> Self {
        Self(value)
    }

    pub fn mrs() -> Self {
        let value: u64;

        // SAFETY: Caller must execute at EL3. SCR_EL3 is only accessible from EL3.
        unsafe {
            core::arch::asm!(
                "mrs {out}, scr_el3",
                out = out(reg) value,
                options(nomem, nostack)
            );
        }

        Self(value)
    }

    pub fn msr(self) {
        // SAFETY: Caller must execute at EL3. Incorrect SCR_EL3 state can make lower-EL entry invalid.
        unsafe {
            core::arch::asm!(
                "msr scr_el3, {value}",
                value = in(reg) self.0,
                options(nomem, nostack)
            );
        }
    }

    pub const fn raw(self) -> u64 {
        self.0
    }

    /// NS: next lower EL runs in Non-secure state.
    pub const fn with_ns(self) -> Self {
        Self(self.0 | (1 << 0))
    }

    /// IRQ routing bit. 0 means IRQs are not routed to EL3.
    pub const fn without_irq_to_el3(self) -> Self {
        Self(self.0 & !(1 << 1))
    }

    /// FIQ routing bit. 0 means FIQs are not routed to EL3.
    pub const fn without_fiq_to_el3(self) -> Self {
        Self(self.0 & !(1 << 2))
    }

    /// EA routing bit. 0 means external aborts are not routed to EL3.
    pub const fn without_ea_to_el3(self) -> Self {
        Self(self.0 & !(1 << 3))
    }

    /// HCE: enable HVC instruction at lower ELs.
    pub const fn with_hce(self) -> Self {
        Self(self.0 | (1 << 8))
    }

    /// SMD: disable SMC instruction at lower ELs.
    pub const fn with_smd(self) -> Self {
        Self(self.0 | (1 << 7))
    }

    /// RW: next lower EL is AArch64.
    pub const fn with_rw(self) -> Self {
        Self(self.0 | (1 << 10))
    }

    /// ST: do not trap secure EL1 physical timer access to EL3.
    pub const fn with_st(self) -> Self {
        Self(self.0 | (1 << 11))
    }
}
