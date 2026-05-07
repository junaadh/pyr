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
        value |= attr.stage2_bits() << 2;

        Self(value)
    }

    pub const fn page(addr: u64, attr: MemAttr) -> Self {
        let mut value = addr & 0x0000_ffff_ffff_f000;

        // valid page descriptor at level 3
        value |= 0b11;

        // stage-2 access permissions: full access
        value |= 0b11 << 6;

        // shareability: inner shareable
        value |= 0b11 << 8;

        // access flag
        value |= 1 << 10;

        // attr index
        value |= attr.stage2_bits() << 2;

        Self(value)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum MemAttr {
    Device,
    Normal,
}

impl MemAttr {
    pub const fn stage2_bits(self) -> u64 {
        match self {
            // Device-nGnRE-ish
            Self::Device => 0b0001,

            // Normal memory, inner/outer write-back cacheable
            Self::Normal => 0b1111,
        }
    }
}
