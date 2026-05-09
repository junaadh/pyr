use crate::{
    addr::{IpaAddr, PhysAddr},
    exception::{DataAbortIss, TrapFrame},
};

pub trait Platform {
    const UART_BASE: PhysAddr;

    fn early_init();

    fn early_putc(byte: u8);

    fn early_print(s: &str) {
        s.bytes().for_each(Self::early_putc);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MmioError {
    UnknownDevice,
    InvalidSyndrome,
    DeviceError(MmioDeviceError),
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum MmioDeviceError {
    UnsupportedAccess,
    BadRegister,
    StubbedRegister { policy: MmioStub, offset: u64 },
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum MmioStub {
    ReadAsZero,
    IgnoreWrite,
}

impl MmioStub {
    pub const fn describe(self) -> &'static str {
        match self {
            Self::ReadAsZero => "read-as-zero",
            Self::IgnoreWrite => "ignore-write",
        }
    }
}

pub trait MmioDevice {
    fn contains(ipa: u64) -> bool;
    fn emulate(access: MmioAccess) -> Result<MmioResult, MmioDeviceError>;
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct MmioAccess {
    pub ipa: u64,
    pub offset: u64,
    pub width: MmioWidth,
    pub kind: MmioAccessKind,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum MmioAccessKind {
    Read { target: GuestReg },
    Write { source: GuestReg, value: u64 },
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum GuestReg {
    Gpr(u8),
    Zero,
}

impl GuestReg {
    pub const fn from_srt(srt: u8) -> Result<Self, MmioError> {
        if srt < 31 {
            Ok(Self::Gpr(srt))
        } else if srt == 31 {
            Ok(Self::Zero)
        } else {
            Err(MmioError::InvalidSyndrome)
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum MmioWidth {
    U8,
    U16,
    U32,
    U64,
}

impl MmioWidth {
    pub const fn from_sas(sas: u8) -> Result<Self, MmioError> {
        match sas {
            0 => Ok(Self::U8),
            1 => Ok(Self::U16),
            2 => Ok(Self::U32),
            3 => Ok(Self::U64),
            _ => Err(MmioError::InvalidSyndrome),
        }
    }

    pub const fn mask(self, value: u64) -> u64 {
        match self {
            Self::U8 => value & 0xff,
            Self::U16 => value & 0xffff,
            Self::U32 => value & 0xffff_ffff,
            Self::U64 => value,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum MmioResult {
    Done,
    Read(u64),
}

impl MmioAccess {
    pub fn from_abort(
        ipa: IpaAddr,
        base: u64,
        frame: &TrapFrame,
        iss: DataAbortIss,
    ) -> Result<Self, MmioError> {
        if !iss.isv {
            return Err(MmioError::InvalidSyndrome);
        }

        let ipa = ipa.as_u64();
        let width = MmioWidth::from_sas(iss.sas)?;
        let reg = GuestReg::from_srt(iss.srt)?;

        let kind = if iss.wnr {
            let value = match reg {
                GuestReg::Gpr(index) => frame
                    .x
                    .get(index as usize)
                    .copied()
                    .ok_or(MmioError::InvalidSyndrome)?,

                GuestReg::Zero => 0,
            };

            MmioAccessKind::Write {
                source: reg,
                value: width.mask(value),
            }
        } else {
            MmioAccessKind::Read { target: reg }
        };

        Ok(Self {
            ipa,
            offset: ipa - base,
            width,
            kind,
        })
    }
}
