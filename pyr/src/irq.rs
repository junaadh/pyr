#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IrqNumber(u32);

impl IrqNumber {
    pub const SPURIOUS: Self = Self(1023);

    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u32 {
        self.0
    }

    pub const fn is_spurious(self) -> bool {
        self.0 >= 1020
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InterruptEvent {
    Irq(IrqNumber),
    Fiq,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InterruptSource {
    PhysicalTimer,
    Unknown(IrqNumber),
}

impl InterruptSource {
    pub fn from_irq(_irq: IrqNumber) -> Self {
        Self::PhysicalTimer
    }

    pub const fn guest_irq(self) -> Option<u32> {
        match self {
            Self::PhysicalTimer => Some(11),
            Self::Unknown(_) => None,
        }
    }
}
