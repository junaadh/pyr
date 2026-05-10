#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Gpr {
    X0 = 0,
    X1 = 1,
    X2 = 2,
    X3 = 3,
    X4 = 4,
    X5 = 5,
    X6 = 6,
    X7 = 7,
    X8 = 8,
    X9 = 9,
    X10 = 10,
    X11 = 11,
    X12 = 12,
    X13 = 13,
    X14 = 14,
    X15 = 15,
    X16 = 16,
    X17 = 17,
    X18 = 18,
    X19 = 19,
    X20 = 20,
    X21 = 21,
    X22 = 22,
    X23 = 23,
    X24 = 24,
    X25 = 25,
    X26 = 26,
    X27 = 27,
    X28 = 28,
    X29 = 29,
    X30 = 30,
}

impl Gpr {
    pub const fn index(self) -> usize {
        self as usize
    }

    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::X0),
            1 => Some(Self::X1),
            2 => Some(Self::X2),
            3 => Some(Self::X3),
            4 => Some(Self::X4),
            5 => Some(Self::X5),
            6 => Some(Self::X6),
            7 => Some(Self::X7),
            8 => Some(Self::X8),
            9 => Some(Self::X9),
            10 => Some(Self::X10),
            11 => Some(Self::X11),
            12 => Some(Self::X12),
            13 => Some(Self::X13),
            14 => Some(Self::X14),
            15 => Some(Self::X15),
            16 => Some(Self::X16),
            17 => Some(Self::X17),
            18 => Some(Self::X18),
            19 => Some(Self::X19),
            20 => Some(Self::X20),
            21 => Some(Self::X21),
            22 => Some(Self::X22),
            23 => Some(Self::X23),
            24 => Some(Self::X24),
            25 => Some(Self::X25),
            26 => Some(Self::X26),
            27 => Some(Self::X27),
            28 => Some(Self::X28),
            29 => Some(Self::X29),
            30 => Some(Self::X30),
            _ => None,
        }
    }
}
