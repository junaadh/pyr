use pyr_arch::exception::TrapFrame;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]

pub enum ExtensionId {
    DebugConsole,
    Unknown(u64),
}

impl ExtensionId {
    pub const DEBUG_CONSOLE_RAW: u64 = 0x7079;

    pub const fn from_raw(raw: u64) -> Self {
        match raw {
            Self::DEBUG_CONSOLE_RAW => Self::DebugConsole,
            other => Self::Unknown(other),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]

pub enum FunctionId {
    Putc,
    Unknown(u64),
}

impl FunctionId {
    pub const DEBUG_PUTC_RAW: u64 = 0x0001;

    pub const fn from_raw(raw: u64) -> Self {
        match raw {
            Self::DEBUG_PUTC_RAW => Self::Putc,
            other => Self::Unknown(other),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]

pub struct HvcCall {
    pub imm16: u16,
    pub extension: ExtensionId,
    pub function: FunctionId,
    pub arg0: u64,
    pub arg1: u64,
    pub arg2: u64,
}

impl HvcCall {
    pub const fn new(
        imm16: u16,

        extension: ExtensionId,

        function: FunctionId,

        arg0: u64,

        arg1: u64,

        arg2: u64,
    ) -> Self {
        Self {
            imm16,

            extension,

            function,

            arg0,

            arg1,

            arg2,
        }
    }

    pub const fn from_frame(frame: &TrapFrame, imm16: u16) -> Self {
        Self::new(
            imm16,
            ExtensionId::from_raw(frame.x[0]),
            FunctionId::from_raw(frame.x[1]),
            frame.x[2],
            frame.x[3],
            frame.x[4],
        )
    }
}
