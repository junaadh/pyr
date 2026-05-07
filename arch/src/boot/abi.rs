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

    pub modules_ptr: u64,
    pub modules_len: u64,

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
    pub const HAS_MODULES: Self = Self(1 << 1);
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

    GuestRam = 8,
    GuestReserved = 9,

    Dtb = 10,
    Initrd = 11,

    Mmio = 12,
    FirmwareRuntime = 13,
    FirmwareReclaimable = 14,

    Acpi = 15,
    BadMemory = 16,
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
pub struct RawBootModule {
    pub start: u64,
    pub len: u64,

    pub kind: RawModuleKind,
    pub flags: RawModuleFlags,

    pub name_ptr: u64,
    pub name_len: u64,

    pub load_addr_hint: u64,
    pub align: u64,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RawModuleKind {
    Unknown = 0,

    LinuxKernel = 1,
    Dtb = 2,
    Initrd = 3,

    TinyPayload = 4,
    GuestPayload = 5,

    SymbolTable = 6,
    DeviceTreeOverlay = 7,
    ConfigBlob = 8,
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RawModuleFlags(pub u64);

impl RawModuleFlags {
    pub const NONE: Self = Self(0);
    pub const REQUIRED: Self = Self(1 << 0);
    pub const COMPRESSED: Self = Self(1 << 1);
    pub const RELOCATABLE: Self = Self(1 << 2);
    pub const EXECUTABLE: Self = Self(1 << 3);

    pub const fn contains(self, rhs: Self) -> bool {
        (self.0 & rhs.0) == rhs.0
    }
}
