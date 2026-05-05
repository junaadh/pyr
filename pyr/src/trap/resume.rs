#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resume {
    ReturnToGuest,
    AdvancePcAndReturn,
    Halt,
}
