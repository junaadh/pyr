// pyr-arch/src/boot/abi.rs

#![allow(dead_code)]

pub const PYR_BOOT_MAGIC: u64 = u64::from_le_bytes(*b"FCKUEL1\0");
pub const PYR_BOOT_VERSION: u32 = 1;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RawBootInfo {
    pub magic: u64,
    pub version: u32,
    pub size: u32,

    pub boot_source: RawBootSource,
    pub machine_kind: RawMachineKind,
    pub entry_el: RawExceptionLevel,
    pub reserved0: u32,

    pub flags: RawBootFlags,

    pub memory_map_ptr: u64,
    pub memory_map_len: u64,

    pub resources_ptr: u64,
    pub resources_len: u64,

    pub cmdline_ptr: u64,
    pub cmdline_len: u64,

    pub platform_info_ptr: u64,
    pub platform_info_len: u64,

    pub firmware_info_ptr: u64,
    pub firmware_info_len: u64,

    pub reserved_ptr: u64,
    pub reserved_len: u64,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RawBootSource {
    Unknown = 0,
    BareEntry = 1,
    Uefi = 2,
    TestHarness = 3,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RawMachineKind {
    Unknown = 0,
    QemuVirt = 1,
    RaspberryPi4 = 2,
    RaspberryPi5 = 3,
    GenericArmVirt = 4,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RawExceptionLevel {
    Unknown = 0,
    El1 = 1,
    El2 = 2,
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RawBootFlags(pub u64);

impl RawBootFlags {
    pub const NONE: Self = Self(0);
    pub const HAS_MEMORY_MAP: Self = Self(1 << 0);
    pub const HAS_RESOURCES: Self = Self(1 << 1);
    pub const HAS_CMDLINE: Self = Self(1 << 2);
    pub const HAS_PLATFORM_INFO: Self = Self(1 << 3);
    pub const HAS_FIRMWARE_INFO: Self = Self(1 << 4);

    pub const fn contains(self, rhs: Self) -> bool {
        (self.0 & rhs.0) == rhs.0
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RawMemoryRegion {
    pub start: u64,
    pub len: u64,
    pub kind: RawMemoryKind,
    pub flags: RawMemoryFlags,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RawMemoryKind {
    Unknown = 0,
    Usable = 1,
    Reserved = 2,

    LoaderImage = 3,
    HypervisorImage = 4,
    HypervisorStack = 5,
    HypervisorHeap = 6,
    FramePool = 7,

    GuestRamArena = 8,
    BootResource = 9,
    BootResourceReserved = 10,

    Mmio = 11,
    FirmwareRuntime = 12,
    FirmwareReclaimable = 13,

    Acpi = 14,
    BadMemory = 15,
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RawMemoryFlags(pub u64);

impl RawMemoryFlags {
    pub const NONE: Self = Self(0);
    pub const READABLE: Self = Self(1 << 0);
    pub const WRITABLE: Self = Self(1 << 1);
    pub const EXECUTABLE: Self = Self(1 << 2);
    pub const DEVICE: Self = Self(1 << 3);
    pub const RUNTIME: Self = Self(1 << 4);
    pub const RECLAIMABLE: Self = Self(1 << 5);
    pub const ZEROED: Self = Self(1 << 6);

    pub const fn contains(self, rhs: Self) -> bool {
        (self.0 & rhs.0) == rhs.0
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RawBootResource {
    pub start: u64,
    pub len: u64,

    pub kind: RawBootResourceKind,
    pub flags: RawBootResourceFlags,

    pub name_ptr: u64,
    pub name_len: u64,

    pub media: RawBootResourceMedia,
    pub reserved0: u32,

    pub align: u64,
    pub metadata_ptr: u64,
    pub metadata_len: u64,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RawBootResourceKind {
    Unknown = 0,

    /// Loader-provided config for Pyr itself.
    PyrConfig = 1,

    /// Platform DTB describing the machine Pyr is running on.
    PlatformDtb = 2,

    /// ACPI/RSDP/etc if booted from firmware world.
    FirmwareTable = 3,

    /// Generic initial archive handed to Pyr.
    BootArchive = 4,

    /// Blob available at boot but not semantically interpreted by ABI.
    Blob = 5,

    /// Debug/symbol info for Pyr.
    SymbolTable = 6,

    /// Crash log / previous boot state / diagnostics.
    Diagnostics = 7,

    /// Temporary dev-only payload. Pyr may choose to interpret it.
    DevPayload = 8,
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RawBootResourceFlags(pub u64);

impl RawBootResourceFlags {
    pub const NONE: Self = Self(0);
    pub const REQUIRED: Self = Self(1 << 0);
    pub const RECLAIMABLE: Self = Self(1 << 1);
    pub const COMPRESSED: Self = Self(1 << 2);
    pub const EXECUTABLE: Self = Self(1 << 3);
    pub const TRUSTED: Self = Self(1 << 4);

    pub const fn contains(self, rhs: Self) -> bool {
        (self.0 & rhs.0) == rhs.0
    }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RawBootResourceMedia {
    Unknown = 0,
    Memory = 1,
    FirmwareFile = 2,
    Embedded = 3,
    Disk = 4,
    Network = 5,
}
