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
    VirtualTimer,
    Uart,
    Unknown(IrqNumber),
}

impl InterruptSource {
    pub const GUEST_VTIMER_IRQ: u32 = 11;
    pub const GUEST_PTIMER_IRQ: u32 = 13;

    pub fn from_physical_irq(irq: IrqNumber) -> Self {
        #[allow(clippy::match_single_binding)]
        match irq.raw() {
            _ => Self::VirtualTimer,
        }
    }

    pub const fn guest_irq(self) -> Option<u32> {
        match self {
            Self::PhysicalTimer => Some(Self::GUEST_PTIMER_IRQ),
            Self::VirtualTimer => Some(Self::GUEST_VTIMER_IRQ),
            Self::Uart => Some(33),
            Self::Unknown(_) => None,
        }
    }
}
