#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(transparent)]
pub struct CurrentEl(u64);

impl CurrentEl {
    #[inline(always)]
    pub fn mrs() -> Self {
        let value: u64;

        // SAFETY: Reading CurrentEL is valid at every AArch64 exception level.
        unsafe {
            core::arch::asm!(
                "mrs {out}, CurrentEL",
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
    pub const fn exception_level(self) -> u8 {
        ((self.0 >> 2) & 0b11) as u8
    }

    #[inline(always)]
    pub const fn is_el2(self) -> bool {
        self.exception_level() == 2
    }
}
