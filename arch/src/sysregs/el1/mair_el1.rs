#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(transparent)]
pub struct MairEl1(u64);

impl MairEl1 {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn minimal() -> Self {
        // Attr0 = Normal WB/WA: 0xff
        // Attr1 = Device-nGnRE: 0x04
        Self(0xff | (0x04 << 8))
    }

    pub fn msr(self) {
        // SAFETY: Caller must execute at EL2 or higher.
        unsafe {
            core::arch::asm!(
                "msr mair_el1, {value}",
                value = in(reg) self.0,
                options(nomem, nostack)
            );
        }
    }

    pub fn mrs() -> Self {
        let value: u64;
        // SAFETY: Caller must execute at EL2 or higher.
        unsafe {
            core::arch::asm!("mrs {out}, mair_el1", out = out(reg) value, options(nomem, nostack));
        }
        Self(value)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}
