#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExceptionClass {
    Hvc64 { imm16: u16 },
    DataAbortLower { iss: DataAbortIss },
    InstructionAbortLower { iss: u32 },
    SysregTrap { iss: u32 },
    WfiWfe,
    Unknown { ec: u8, iss: u32 },
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
