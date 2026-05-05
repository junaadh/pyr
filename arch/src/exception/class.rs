#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExceptionClass {
    Hvc64 { imm16: u16 },
    DataAbortLower { iss: u32 },
    InstructionAbortLower { iss: u32 },
    SysregTrap { iss: u32 },
    WfiWfe,
    Unknown { ec: u8, iss: u32 },
}
