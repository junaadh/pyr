#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Vmid(u16);

impl Vmid {
    pub const BOOT: Self = Self(1);

    pub const fn new(raw: u16) -> Self {
        Self(raw)
    }

    pub const fn as_u16(self) -> u16 {
        self.0
    }
}
