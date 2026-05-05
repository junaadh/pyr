#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
#[repr(transparent)]
pub struct IpaAddr(u64);

impl IpaAddr {
    pub const fn new(addr: u64) -> Self {
        Self(addr)
    }

    pub const fn as_u64(self) -> u64 {
        self.0
    }

    pub const fn offset(self, n: u64) -> Self {
        Self(self.0 + n)
    }

    pub const fn align_down(self, align: u64) -> Self {
        Self(self.0 & !(align - 1))
    }

    pub const fn align_up(self, align: u64) -> Self {
        Self((self.0 + align - 1) & !(align - 1))
    }
}
