#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
#[repr(transparent)]
pub struct VtcrEl2(u64);

impl VtcrEl2 {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn mrs() -> Self {
        let value: u64;

        // SAFETY: Caller must execute at EL2 or higher.
        unsafe {
            core::arch::asm!(
                "mrs {out}, vtcr_el2",
                out = out(reg) value,
                options(nomem, nostack)
            );
        }

        Self(value)
    }

    pub fn msr(self) {
        // SAFETY: Caller must execute at EL2 or higher and provide a valid VTCR_EL2 value.
        unsafe {
            core::arch::asm!(
                "msr vtcr_el2, {value}",
                value = in(reg) self.0,
                options(nomem, nostack)
            );
        }
    }

    pub const fn raw(self) -> u64 {
        self.0
    }

    /// T0SZ: size offset. IPA size = 2^(64 - T0SZ).
    pub const fn with_t0sz(self, t0sz: u64) -> Self {
        Self((self.0 & !0x3f) | (t0sz & 0x3f))
    }

    /// SL0 bits. For 4 KiB granule, this selects starting level.
    pub const fn with_sl0_level1(self) -> Self {
        Self(self.0 | (1 << 6))
    }

    /// TG0 = 00, 4 KiB granule.
    pub const fn with_tg0_4k(self) -> Self {
        Self(self.0 & !(0b11 << 14))
    }

    /// SH0 = 0b11, inner shareable.
    pub const fn with_sh0_inner(self) -> Self {
        Self((self.0 & !(0b11 << 12)) | (0b11 << 12))
    }

    /// ORGN0 = 0b01, write-back read/write allocate cacheable.
    pub const fn with_orgn0_write_back(self) -> Self {
        Self((self.0 & !(0b11 << 10)) | (0b01 << 10))
    }

    /// IRGN0 = 0b01, write-back read/write allocate cacheable.
    pub const fn with_irgn0_write_back(self) -> Self {
        Self((self.0 & !(0b11 << 8)) | (0b01 << 8))
    }
}
