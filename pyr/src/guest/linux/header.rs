use pyr_arch::addr::IpaAddr;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum LinuxImageError {
    TooSmall,
    BadMagic,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct LinuxImageHeader {
    code0: u32,
    code1: u32,
    text_offset: u64,
    image_size: u64,
    flags: u64,
    res2: u64,
    res3: u64,
    res4: u64,
    magic: u32,
    res5: u32,
}

impl LinuxImageHeader {
    pub const MAGIC: u32 = 0x644d_5241; // "ARM\x64" little endian

    pub fn parse(image: &[u8]) -> Result<Self, LinuxImageError> {
        if image.len() < core::mem::size_of::<Self>() {
            return Err(LinuxImageError::TooSmall);
        }

        // SAFETY: We checked the slice is large enough. Header is copied by value,
        // so no unaligned reference is created.
        let header =
            unsafe { core::ptr::read_unaligned(image.as_ptr().cast::<Self>()) };

        if header.magic != Self::MAGIC {
            return Err(LinuxImageError::BadMagic);
        }

        Ok(header)
    }

    pub const fn text_offset(self) -> u64 {
        self.text_offset
    }

    pub const fn image_size(self) -> u64 {
        self.image_size
    }

    pub const fn entry_ipa(self, load_base: IpaAddr) -> IpaAddr {
        load_base.offset(self.text_offset)
    }
}
