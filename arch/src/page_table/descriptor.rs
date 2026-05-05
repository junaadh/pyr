#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(transparent)]
pub struct Descriptor(u64);

impl Descriptor {
    pub const fn invalid() -> Self {
        Self(0)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }

    pub const fn is_valid(self) -> bool {
        self.0 & 1 != 0
    }

    pub const fn table(addr: u64) -> Self {
        Self((addr & 0x0000_ffff_ffff_f000) | 0b11)
    }

    pub const fn block(addr: u64, attr: MemAttr) -> Self {
        let mut value = addr & 0x0000_ffff_ffe0_0000;

        // valid block descriptor
        value |= 0b01;

        // stage-2 access permissions: full access
        value |= 0b11 << 6;

        // shareability: inner shareable
        value |= 0b11 << 8;

        // access flag
        value |= 1 << 10;

        // attr index
        value |= (attr.index() as u64) << 2;

        Self(value)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum MemAttr {
    Normal,
    Device,
}

impl MemAttr {
    pub const fn index(self) -> u8 {
        match self {
            Self::Normal => 0,
            Self::Device => 1,
        }
    }
}
