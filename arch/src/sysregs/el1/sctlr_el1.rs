#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(transparent)]
pub struct SctlrEl1(u64);

impl SctlrEl1 {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn mmu_disabled() -> Self {
        // Keep RES1-ish common bit 11 set. M/C/I off.
        Self(1 << 11)
    }

    pub fn msr(self) {
        // SAFETY: Caller must execute at EL2 or higher.
        unsafe {
            core::arch::asm!(
                "msr sctlr_el1, {value}",
                value = in(reg) self.0,
                options(nomem, nostack)
            );
        }
    }

    pub fn mrs() -> Self {
        let value: u64;
        // SAFETY: Caller must execute at EL2 or higher.
        unsafe {
            core::arch::asm!("mrs {out}, sctlr_el1", out = out(reg) value, options(nomem, nostack));
        }
        Self(value)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}
