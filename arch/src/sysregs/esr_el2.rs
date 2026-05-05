#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(transparent)]
pub struct EsrEl2(u64);

impl EsrEl2 {
    #[inline(always)]
    pub fn mrs() -> Self {
        let value: u64;

        // SAFETY: Caller must execute at EL2 or higher.
        unsafe {
            core::arch::asm!(
                "mrs {out}, esr_el2",
                out = out(reg) value,
                options(nomem, nostack)
            );
        }

        Self(value)
    }

    #[inline(always)]
    pub const fn raw(self) -> u64 {
        self.0
    }

    #[inline(always)]
    pub const fn ec(self) -> u8 {
        ((self.0 >> 26) & 0b11_1111) as u8
    }

    #[inline(always)]
    pub const fn iss(self) -> u32 {
        (self.0 & 0x01ff_ffff) as u32
    }

    #[inline(always)]
    pub const fn is_hvc64(self) -> bool {
        self.ec() == 0x16
    }
}
