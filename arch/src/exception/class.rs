#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExceptionClass {
    Hvc64 { imm16: u16 },
    DataAbortLower { iss: DataAbortIss },
    InstructionAbortLower { iss: u32 },
    SysregTrap { iss: SysRegIss },
    Unknown { ec: u8, iss: u32 },
    Smc64 { imm16: u16 },
    Wfx { kind: WfxKind },
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct DataAbortIss {
    pub raw: u32,
    pub dfsc: u8,
    pub wnr: bool,
    pub s1ptw: bool,
    pub isv: bool,
    pub sas: u8,
    pub srt: u8,
}

impl DataAbortIss {
    pub const fn decode(iss: u32) -> Self {
        Self {
            raw: iss,
            dfsc: (iss & 0x3f) as u8,
            wnr: ((iss >> 6) & 1) != 0,
            s1ptw: ((iss >> 7) & 1) != 0,
            isv: ((iss >> 24) & 1) != 0,
            sas: ((iss >> 22) & 0b11) as u8,
            srt: ((iss >> 16) & 0b1_1111) as u8,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WfxKind {
    Wfe,
    Wfi,
}

impl WfxKind {
    pub const fn from_iss(iss: u32) -> Self {
        if iss & 1 == 0 { Self::Wfi } else { Self::Wfe }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SysRegIss {
    pub raw: u32,
    pub op0: u8,
    pub op1: u8,
    pub crn: u8,
    pub crm: u8,
    pub op2: u8,
    pub rt: u8,
    pub is_write: bool,
}

impl SysRegIss {
    pub const fn decode(iss: u32) -> Self {
        Self {
            raw: iss,
            op0: ((iss >> 20) & 0b11) as u8,
            op1: ((iss >> 14) & 0b111) as u8,
            crn: ((iss >> 10) & 0b1111) as u8,
            crm: ((iss >> 1) & 0b1111) as u8,
            op2: ((iss >> 17) & 0b111) as u8,
            rt: ((iss >> 5) & 0b1_1111) as u8,
            is_write: (iss & 1) == 0,
        }
    }
}
