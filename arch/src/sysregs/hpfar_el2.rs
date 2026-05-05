use crate::addr::IpaAddr;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(transparent)]
pub struct HpfarEl2(u64);

impl HpfarEl2 {
    pub fn mrs() -> Self {
        let value: u64;

        // SAFETY: Caller must execute at EL2 or higher. Reading HPFAR_EL2 is valid
        // while handling a stage-2 translation fault and does not access memory.
        unsafe {
            core::arch::asm!(
                "mrs {out}, hpfar_el2",
                out = out(reg) value,
                options(nomem, nostack)
            );
        }
        Self(value)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }

    pub const fn ipa_base(self) -> IpaAddr {
        // HPFAR_EL2.FIPA is bits [39:4], representing IPA[47:12].
        IpaAddr::new((self.0 & 0x0000_ffff_ffff_fff0) << 8)
    }
}
