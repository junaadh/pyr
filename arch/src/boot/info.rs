use crate::{
    addr::PhysAddr,
    boot::abi::{
        PYR_BOOT_MAGIC, PYR_BOOT_VERSION, RawBootFlags, RawBootInfo,
        RawBootResource, RawBootResourceKind, RawBootResourceMedia,
        RawBootSource, RawExceptionLevel, RawMachineKind, RawMemoryKind,
        RawMemoryRegion,
    },
};
use core::{mem, slice};

#[derive(Debug)]
pub enum BootInfoError {
    Null,
    BadMagic,
    UnsupportedVersion,
    BadSize,
    BadSlice,
    BadUtf8,
    MissingMemoryMap,
    MissingRequiredResource,
    RegionOverflow,
    ResourceOverflow,
    UnalignedMemoryMap,
    UnalignedResourceMap,
}

pub struct BootInfo<'a> {
    #[allow(dead_code)]
    raw: &'a RawBootInfo,
    boot_source: BootSource,
    machine: MachineKind,
    entry_el: ExceptionLevel,
    memory: MemoryMap<'a>,
    resources: BootResources<'a>,
    cmdline: Option<&'a str>,
    platform_info: Option<&'a [u8]>,
    firmware_info: Option<&'a [u8]>,
}

impl<'a> BootInfo<'a> {
    /// # Safety
    ///
    /// `ptr` must point to a valid `RawBootInfo` whose referenced arrays/blobs
    /// remain alive for `'a`.
    pub unsafe fn from_raw_ptr(
        ptr: *const RawBootInfo,
    ) -> Result<Self, BootInfoError> {
        // SAFETY: `ptr` must point to a valid `RawBootInfo` for the duration of `a`
        unsafe {
            let raw = ptr.as_ref().ok_or(BootInfoError::Null)?;
            Self::from_raw(raw)
        }
    }

    /// # Safety
    ///
    /// All raw pointers inside `raw` must be valid for the lifetime of `raw`.
    pub unsafe fn from_raw(
        raw: &'a RawBootInfo,
    ) -> Result<Self, BootInfoError> {
        validate_header(raw)?;

        // SAFETY: All raw pointers inside `raw` must be valid for the lifetime of `raw`
        let memory = unsafe {
            raw_slice::<RawMemoryRegion>(
                raw.memory_map_ptr,
                raw.memory_map_len,
                BootInfoError::UnalignedMemoryMap,
            )?
        };

        if memory.is_empty() && raw.flags.contains(RawBootFlags::HAS_MEMORY_MAP)
        {
            return Err(BootInfoError::MissingMemoryMap);
        }

        // SAFETY: All raw pointers inside `raw` must be valid for the lifetime of `raw`
        let resources = unsafe {
            raw_slice::<RawBootResource>(
                raw.resources_ptr,
                raw.resources_len,
                BootInfoError::UnalignedResourceMap,
            )?
        };

        let cmdline = optional_str(raw.cmdline_ptr, raw.cmdline_len)?;
        let platform_info =
            optional_bytes(raw.platform_info_ptr, raw.platform_info_len)?;
        let firmware_info =
            optional_bytes(raw.firmware_info_ptr, raw.firmware_info_len)?;

        Ok(Self {
            raw,
            boot_source: BootSource::from(raw.boot_source),
            machine: MachineKind::from(raw.machine_kind),
            entry_el: ExceptionLevel::from(raw.entry_el),
            memory: MemoryMap { raw: memory },
            resources: BootResources { raw: resources },
            cmdline,
            platform_info,
            firmware_info,
        })
    }

    pub const fn boot_source(&self) -> BootSource {
        self.boot_source
    }

    pub const fn machine(&self) -> MachineKind {
        self.machine
    }

    pub const fn entry_el(&self) -> ExceptionLevel {
        self.entry_el
    }

    pub const fn memory(&self) -> &MemoryMap<'a> {
        &self.memory
    }

    pub const fn cmdline(&self) -> Option<&'a str> {
        self.cmdline
    }

    pub const fn platform_info(&self) -> Option<&'a [u8]> {
        self.platform_info
    }

    pub const fn firmware_info(&self) -> Option<&'a [u8]> {
        self.firmware_info
    }

    pub const fn resources(&self) -> &BootResources<'a> {
        &self.resources
    }

    pub fn resource_named(&self, name: &str) -> Option<BootResource<'a>> {
        self.resources.named(name)
    }

    pub fn platform_dtb(&self) -> Option<BootResource<'a>> {
        self.resources.platform_dtb()
    }

    pub fn hypervisor_heap(&self) -> Option<MemoryRegion> {
        self.memory.first_of(MemoryKind::HypervisorHeap)
    }

    pub fn frame_pool(&self) -> Option<MemoryRegion> {
        self.memory.first_of(MemoryKind::FramePool)
    }

    pub fn guest_ram_arena(&self) -> Option<MemoryRegion> {
        self.memory.first_of(MemoryKind::PyrReserved)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BootSource {
    Unknown,
    BareEntry,
    Uefi,
    TestHarness,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MachineKind {
    Unknown,
    QemuVirt,
    RaspberryPi4,
    RaspberryPi5,
    GenericArmVirt,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExceptionLevel {
    Unknown,
    El1,
    El2,
}

pub struct MemoryMap<'a> {
    raw: &'a [RawMemoryRegion],
}

impl<'a> MemoryMap<'a> {
    pub fn iter(&self) -> impl Iterator<Item = MemoryRegion> + '_ {
        self.raw
            .iter()
            .filter_map(|raw| MemoryRegion::try_from_raw(raw).ok())
    }

    pub fn first_of(&self, kind: MemoryKind) -> Option<MemoryRegion> {
        self.iter().find(|r| r.kind == kind)
    }

    pub fn regions_of(
        &self,
        kind: MemoryKind,
    ) -> impl Iterator<Item = MemoryRegion> + '_ {
        self.iter().filter(move |r| r.kind == kind)
    }

    pub fn contains_phys(&self, addr: PhysAddr) -> Option<MemoryRegion> {
        self.iter().find(|r| r.contains(addr))
    }

    pub fn assert_no_overlaps(&self) -> Result<(), BootInfoError> {
        for a in self.iter() {
            for b in self.iter() {
                if a.start.as_u64() == b.start.as_u64()
                    && a.end.as_u64() == b.end.as_u64()
                {
                    continue;
                }

                if a.overlaps(b) {
                    return Err(BootInfoError::BadSlice);
                }
            }
        }

        Ok(())
    }
}

pub struct BootResources<'a> {
    raw: &'a [RawBootResource],
}

impl<'a> BootResources<'a> {
    pub fn iter(&self) -> impl Iterator<Item = BootResource<'a>> + '_ {
        self.raw
            .iter()
            .filter_map(|raw| BootResource::try_from_raw(raw).ok())
    }

    pub fn first_of(&self, kind: BootResourceKind) -> Option<BootResource<'a>> {
        self.iter().find(|m| m.kind == kind)
    }

    pub fn required(&self) -> impl Iterator<Item = BootResource<'a>> + '_ {
        self.iter().filter(|m| m.flags.required())
    }

    pub fn named(&self, name: &str) -> Option<BootResource<'a>> {
        self.iter().find(|r| r.name() == Some(name))
    }

    pub fn pyr_config(&self) -> Option<BootResource<'a>> {
        self.first_of(BootResourceKind::PyrConfig)
    }

    pub fn platform_dtb(&self) -> Option<BootResource<'a>> {
        self.first_of(BootResourceKind::PlatformDtb)
    }

    pub fn boot_archive(&self) -> Option<BootResource<'a>> {
        self.first_of(BootResourceKind::BootArchive)
    }
}

pub struct BootResource<'a> {
    raw: &'a RawBootResource,
    start: PhysAddr,
    end: PhysAddr,
    data: &'a [u8],
    name: Option<&'a str>,
    kind: BootResourceKind,
    flags: ResourceFlags,
    media: BootResourceMedia,
    metadata: Option<&'a [u8]>,
}

impl<'a> BootResource<'a> {
    fn try_from_raw(raw: &'a RawBootResource) -> Result<Self, BootInfoError> {
        let end = raw
            .start
            .checked_add(raw.len)
            .ok_or(BootInfoError::ResourceOverflow)?;

        // SAFETY: `raw` needs to be valid for the lifetime `a`
        let data = unsafe { raw_bytes(raw.start, raw.len)? };
        let name = optional_str(raw.name_ptr, raw.name_len)?;
        let metadata = optional_bytes(raw.metadata_ptr, raw.metadata_len)?;

        Ok(Self {
            raw,
            start: PhysAddr::new(raw.start),
            end: PhysAddr::new(end),
            data,
            name,
            kind: BootResourceKind::from(raw.kind),
            flags: ResourceFlags(raw.flags.0),
            media: BootResourceMedia::from(raw.media),
            metadata,
        })
    }

    pub const fn start(&self) -> PhysAddr {
        self.start
    }

    pub const fn end(&self) -> PhysAddr {
        self.end
    }

    pub const fn len(&self) -> u64 {
        self.raw.len
    }

    pub const fn is_empty(&self) -> bool {
        self.raw.len == 0
    }

    pub const fn data(&self) -> &'a [u8] {
        self.data
    }

    pub const fn name(&self) -> Option<&'a str> {
        self.name
    }

    pub const fn kind(&self) -> BootResourceKind {
        self.kind
    }

    pub const fn flags(&self) -> ResourceFlags {
        self.flags
    }

    pub const fn media(&self) -> BootResourceMedia {
        self.media
    }

    pub const fn metadata(&self) -> Option<&'a [u8]> {
        self.metadata
    }

    pub const fn align(&self) -> Option<u64> {
        if self.raw.align == 0 {
            None
        } else {
            Some(self.raw.align)
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MemoryRegion {
    pub start: PhysAddr,
    pub end: PhysAddr,
    pub len: u64,
    pub kind: MemoryKind,
    pub flags: MemoryFlags,
}

impl MemoryRegion {
    fn try_from_raw(raw: &RawMemoryRegion) -> Result<Self, BootInfoError> {
        let end = raw
            .start
            .checked_add(raw.len)
            .ok_or(BootInfoError::RegionOverflow)?;

        Ok(Self {
            start: PhysAddr::new(raw.start),
            end: PhysAddr::new(end),
            len: raw.len,
            kind: MemoryKind::from(raw.kind),
            flags: MemoryFlags(raw.flags.0),
        })
    }

    pub fn contains(&self, addr: PhysAddr) -> bool {
        let addr = addr.as_u64();
        self.start.as_u64() <= addr && addr < self.end.as_u64()
    }

    pub fn overlaps(self, other: Self) -> bool {
        self.start.as_u64() < other.end.as_u64()
            && other.start.as_u64() < self.end.as_u64()
    }

    pub fn is_usable(&self) -> bool {
        self.kind == MemoryKind::Usable
    }

    pub fn is_heap(&self) -> bool {
        self.kind == MemoryKind::HypervisorHeap
    }

    pub fn is_frame_pool(&self) -> bool {
        self.kind == MemoryKind::FramePool
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryKind {
    Unknown,
    Usable,
    Reserved,
    LoaderImage,
    HypervisorImage,
    HypervisorStack,
    HypervisorHeap,
    FramePool,
    PyrReserved,
    BootResource,
    BootResourceReserved,
    Mmio,
    FirmwareRuntime,
    FirmwareReclaimable,
    Acpi,
    BadMemory,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemoryFlags(pub u64);

impl MemoryFlags {
    pub const fn readable(self) -> bool {
        self.0 & (1 << 0) != 0
    }

    pub const fn writable(self) -> bool {
        self.0 & (1 << 1) != 0
    }

    pub const fn executable(self) -> bool {
        self.0 & (1 << 2) != 0
    }

    pub const fn device(self) -> bool {
        self.0 & (1 << 3) != 0
    }

    pub const fn runtime(self) -> bool {
        self.0 & (1 << 4) != 0
    }

    pub const fn reclaimable(self) -> bool {
        self.0 & (1 << 5) != 0
    }

    pub const fn zeroed(self) -> bool {
        self.0 & (1 << 6) != 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BootResourceKind {
    Unknown,
    /// Loader-provided config for Pyr itself.
    PyrConfig,
    /// Platform DTB describing the machine Pyr is running on.
    PlatformDtb,
    /// ACPI/RSDP/etc if booted from firmware world.
    FirmwareTable,
    /// Generic initial archive handed to Pyr.
    BootArchive,
    /// Blob available at boot but not semantically interpreted by ABI.
    Blob,
    /// Debug/symbol info for Pyr.
    SymbolTable,
    /// Crash log / previous boot state / diagnostics.
    Diagnostics,
    /// Temporary dev-only payload. Pyr may choose to interpret it.
    DevPayload,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResourceFlags(pub u64);

impl ResourceFlags {
    pub const fn required(self) -> bool {
        self.0 & (1 << 0) != 0
    }

    pub const fn reclaimable(self) -> bool {
        self.0 & (1 << 1) != 0
    }

    pub const fn compressed(self) -> bool {
        self.0 & (1 << 2) != 0
    }

    pub const fn executable(self) -> bool {
        self.0 & (1 << 3) != 0
    }

    pub const fn trusted(self) -> bool {
        self.0 & (1 << 4) != 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BootResourceMedia {
    Unknown,
    Memory,
    FirmwareFile,
    Embedded,
    Disk,
    Network,
}

fn validate_header(raw: &RawBootInfo) -> Result<(), BootInfoError> {
    if raw.magic != PYR_BOOT_MAGIC {
        return Err(BootInfoError::BadMagic);
    }

    if raw.version != PYR_BOOT_VERSION {
        return Err(BootInfoError::UnsupportedVersion);
    }

    if raw.size < mem::size_of::<RawBootInfo>() as u32 {
        return Err(BootInfoError::BadSize);
    }

    Ok(())
}

unsafe fn raw_slice<'a, T>(
    ptr: u64,
    len: u64,
    unaligned_error: BootInfoError,
) -> Result<&'a [T], BootInfoError> {
    if ptr == 0 || len == 0 {
        return Ok(&[]);
    }

    if !(ptr as usize).is_multiple_of(mem::align_of::<T>()) {
        return Err(unaligned_error);
    }

    let len = usize::try_from(len).map_err(|_| BootInfoError::BadSlice)?;
    let ptr = ptr as *const T;

    // SAFETY: Data soruce ptr and the len should remain within the valid memory
    Ok(unsafe { slice::from_raw_parts(ptr, len) })
}

unsafe fn raw_bytes<'a>(ptr: u64, len: u64) -> Result<&'a [u8], BootInfoError> {
    if ptr == 0 || len == 0 {
        return Ok(&[]);
    }

    let len = usize::try_from(len).map_err(|_| BootInfoError::BadSlice)?;
    // SAFETY: Data soruce ptr and the len should remain within the valid memory
    Ok(unsafe { slice::from_raw_parts(ptr as *const u8, len) })
}

fn optional_bytes<'a>(
    ptr: u64,
    len: u64,
) -> Result<Option<&'a [u8]>, BootInfoError> {
    // SAFETY: Data soruce ptr and the len should remain within the valid memory
    let bytes = unsafe { raw_bytes(ptr, len)? };
    Ok((!bytes.is_empty()).then_some(bytes))
}

fn optional_str<'a>(
    ptr: u64,
    len: u64,
) -> Result<Option<&'a str>, BootInfoError> {
    let Some(bytes) = optional_bytes(ptr, len)? else {
        return Ok(None);
    };

    let s = str::from_utf8(bytes).map_err(|_| BootInfoError::BadUtf8)?;
    Ok(Some(s))
}

impl From<RawBootSource> for BootSource {
    fn from(value: RawBootSource) -> Self {
        match value {
            RawBootSource::BareEntry => Self::BareEntry,
            RawBootSource::Uefi => Self::Uefi,
            RawBootSource::TestHarness => Self::TestHarness,
            RawBootSource::Unknown => Self::Unknown,
        }
    }
}

impl From<RawMachineKind> for MachineKind {
    fn from(value: RawMachineKind) -> Self {
        match value {
            RawMachineKind::QemuVirt => Self::QemuVirt,
            RawMachineKind::RaspberryPi4 => Self::RaspberryPi4,
            RawMachineKind::RaspberryPi5 => Self::RaspberryPi5,
            RawMachineKind::GenericArmVirt => Self::GenericArmVirt,
            RawMachineKind::Unknown => Self::Unknown,
        }
    }
}

impl From<RawExceptionLevel> for ExceptionLevel {
    fn from(value: RawExceptionLevel) -> Self {
        match value {
            RawExceptionLevel::El1 => Self::El1,
            RawExceptionLevel::El2 => Self::El2,
            RawExceptionLevel::Unknown => Self::Unknown,
        }
    }
}

impl From<RawMemoryKind> for MemoryKind {
    fn from(value: RawMemoryKind) -> Self {
        match value {
            RawMemoryKind::Usable => Self::Usable,
            RawMemoryKind::Reserved => Self::Reserved,
            RawMemoryKind::LoaderImage => Self::LoaderImage,
            RawMemoryKind::HypervisorImage => Self::HypervisorImage,
            RawMemoryKind::HypervisorStack => Self::HypervisorStack,
            RawMemoryKind::HypervisorHeap => Self::HypervisorHeap,
            RawMemoryKind::FramePool => Self::FramePool,
            RawMemoryKind::PyrReserved => Self::PyrReserved,
            RawMemoryKind::BootResource => Self::BootResource,
            RawMemoryKind::BootResourceReserved => Self::BootResourceReserved,
            RawMemoryKind::Mmio => Self::Mmio,
            RawMemoryKind::FirmwareRuntime => Self::FirmwareRuntime,
            RawMemoryKind::FirmwareReclaimable => Self::FirmwareReclaimable,
            RawMemoryKind::Acpi => Self::Acpi,
            RawMemoryKind::BadMemory => Self::BadMemory,
            RawMemoryKind::Unknown => Self::Unknown,
        }
    }
}

impl From<RawBootResourceKind> for BootResourceKind {
    fn from(value: RawBootResourceKind) -> Self {
        match value {
            RawBootResourceKind::Unknown => Self::Unknown,
            RawBootResourceKind::PyrConfig => Self::PyrConfig,
            RawBootResourceKind::PlatformDtb => Self::PlatformDtb,
            RawBootResourceKind::FirmwareTable => Self::FirmwareTable,
            RawBootResourceKind::BootArchive => BootResourceKind::BootArchive,
            RawBootResourceKind::Blob => Self::Blob,
            RawBootResourceKind::SymbolTable => Self::SymbolTable,
            RawBootResourceKind::Diagnostics => Self::Diagnostics,
            RawBootResourceKind::DevPayload => Self::DevPayload,
        }
    }
}

impl From<RawBootResourceMedia> for BootResourceMedia {
    fn from(value: RawBootResourceMedia) -> Self {
        match value {
            RawBootResourceMedia::Memory => Self::Memory,
            RawBootResourceMedia::FirmwareFile => Self::FirmwareFile,
            RawBootResourceMedia::Embedded => Self::Embedded,
            RawBootResourceMedia::Disk => Self::Disk,
            RawBootResourceMedia::Network => Self::Network,
            RawBootResourceMedia::Unknown => Self::Unknown,
        }
    }
}
