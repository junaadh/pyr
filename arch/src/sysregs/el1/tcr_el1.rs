#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(transparent)]
pub struct TcrEl1(u64);

impl TcrEl1 {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn disabled_mmu_minimal() -> Self {
        // MMU disabled for now, but leave TCR deterministic.
        Self(0)
    }

    pub fn msr(self) {
        // SAFETY: Caller must execute at EL2 or higher.
        unsafe {
            core::arch::asm!(
                "msr tcr_el1, {value}",
                value = in(reg) self.0,
                options(nomem, nostack)
            );
        }
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}
