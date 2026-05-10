use crate::{
    addr::{IpaAddr, PhysAddr},
    exception::DataAbortIss,
    reg::Gpr,
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
    fn emulate(
        &mut self,
        access: MmioAccess,
    ) -> Result<MmioResult, MmioDeviceError>;
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
    Gpr(Gpr),
    Zero,
}

impl GuestReg {
    pub fn from_srt(srt: u8) -> Result<Self, MmioError> {
        if let Some(reg) = Gpr::from_u8(srt) {
            Ok(Self::Gpr(reg))
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

impl MmioAccess {}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct MmioAccessRequest {
    pub ipa: u64,
    pub offset: u64,
    pub width: MmioWidth,
    pub kind: MmioAccessRequestKind,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum MmioAccessRequestKind {
    Read { target: GuestReg },
    Write { source: GuestReg },
}

impl MmioAccessRequest {
    pub fn decode_abort(
        ipa: IpaAddr,
        base: u64,
        iss: DataAbortIss,
    ) -> Result<MmioAccessRequest, MmioError> {
        if !iss.isv {
            return Err(MmioError::InvalidSyndrome);
        }

        let ipa = ipa.as_u64();
        let width = MmioWidth::from_sas(iss.sas)?;
        let reg = GuestReg::from_srt(iss.srt)?;

        let kind = if iss.wnr {
            MmioAccessRequestKind::Write { source: reg }
        } else {
            MmioAccessRequestKind::Read { target: reg }
        };

        Ok(Self {
            ipa,
            offset: ipa - base,
            width,
            kind,
        })
    }
}

pub trait PhysicalInterruptController {
    type Irq: Copy;

    fn acknowledge() -> Self::Irq;
    fn complete(irq: Self::Irq);
    fn is_spurious(irq: Self::Irq) -> bool;
}
