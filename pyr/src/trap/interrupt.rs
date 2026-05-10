#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InterruptKind {
    Irq,
    Fiq,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrapEntryKind {
    Sync,
    Interrupt(InterruptKind),
}
