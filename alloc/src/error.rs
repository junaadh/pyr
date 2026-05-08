#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocError {
    OutOfMemory,
    BadAlignment,
    BadSize,
    BadRange,
    NotInitialized,

    #[cfg(debug_assertions)]
    DoubleFree,
}
