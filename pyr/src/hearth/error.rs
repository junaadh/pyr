#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum HearthError {
    UnknownExtension(u64),
    UnknownFunction(u64),
    PermissionDenied,
}

impl HearthError {
    pub const fn code(self) -> u64 {
        match self {
            Self::UnknownExtension(_) => 1,
            Self::UnknownFunction(_) => 2,
            Self::PermissionDenied => 3,
        }
    }
}
