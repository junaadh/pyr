use crate::exception::{DataAbortIss, ExceptionClass, SysRegIss, WfxKind};

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

    pub const fn decode(self) -> ExceptionClass {
        let ec = self.ec();
        let iss = self.iss();

        match ec {
            0x01 => ExceptionClass::Wfx {
                kind: WfxKind::from_iss(iss),
            },
            0x16 => ExceptionClass::Hvc64 {
                imm16: (iss & 0xffff) as u16,
            },
            0x17 => ExceptionClass::Smc64 {
                imm16: (iss & 0xffff) as u16,
            },
            0x24 => ExceptionClass::DataAbortLower {
                iss: DataAbortIss::decode(iss),
            },
            0x20 => ExceptionClass::InstructionAbortLower { iss },
            0x18 => ExceptionClass::SysregTrap {
                iss: SysRegIss::decode(iss),
            },
            _ => ExceptionClass::Unknown { ec, iss },
        }
    }
}
