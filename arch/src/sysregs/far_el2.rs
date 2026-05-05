#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(transparent)]
pub struct FarEl2(u64);

impl FarEl2 {
    pub fn mrs() -> Self {
        let value: u64;

        // SAFETY: Caller must execute at EL2 or higher. Reading FAR_EL2 does not
        // access memory and is valid while handling an EL2 exception.
        unsafe {
            core::arch::asm!(
                "mrs {out}, far_el2",
                out = out(reg) value,
                options(nomem, nostack)
            );
        }
        Self(value)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}
