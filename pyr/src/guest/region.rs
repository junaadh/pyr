use pyr_arch::{
    addr::{IpaAddr, PhysAddr},
    page_table::MemAttr,
};

/// Guest-visible memory mapping.
///
/// A `GuestRegion` describes one stage-2 mapping:
///
/// ```text
/// guest IPA range  ->  host physical backing range
/// ```
///
/// Important invariant:
/// - `ipa` is what EL1 sees.
/// - `pa` is where Pyr stores the bytes.
/// - guest code must never receive `pa`.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct GuestRegion {
    ipa: IpaAddr,
    pa: PhysAddr,
    size: usize,
    attr: MemAttr,
}

impl GuestRegion {
    /// Create a normal-cacheable guest RAM region.
    pub const fn ram(ipa: IpaAddr, pa: PhysAddr, size: usize) -> Self {
        Self {
            ipa,
            pa,
            size,
            attr: MemAttr::Normal,
        }
    }

    /// Create a device-like guest region.
    ///
    /// Usually devices are intentionally left unmapped so they trap to EL2.
    /// Use this only when the device should be directly exposed.
    pub const fn device(ipa: IpaAddr, pa: PhysAddr, size: usize) -> Self {
        Self {
            ipa,
            pa,
            size,
            attr: MemAttr::Device,
        }
    }

    pub const fn ipa(self) -> IpaAddr {
        self.ipa
    }

    pub const fn pa(self) -> PhysAddr {
        self.pa
    }

    pub const fn size(self) -> usize {
        self.size
    }

    pub const fn attr(self) -> MemAttr {
        self.attr
    }

    pub const fn end_ipa(self) -> IpaAddr {
        self.ipa.offset(self.size as u64)
    }
}
