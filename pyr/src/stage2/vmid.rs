#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Vmid(u16);

impl Vmid {
    pub const BOOT: Self = Self(1);
    pub const MIN: Self = Self(1);
    pub const MAX: Self = Self(0xffff);

    pub const fn is_reserved(self) -> bool {
        self.0 == 0
    }

    pub const fn new(raw: u16) -> Self {
        Self(raw)
    }

    pub const fn as_u16(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VmidGeneration(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllocatedVmid {
    pub vmid: Vmid,
    pub generation: VmidGeneration,
    pub wrapped: bool,
}

pub struct VmidAllocator {
    next: u16,
    generation: u64,
}

impl VmidAllocator {
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self {
            next: Vmid::MIN.as_u16(),
            generation: 0,
        }
    }

    pub fn alloc(&mut self) -> AllocatedVmid {
        let vmid = Vmid::new(self.next);
        let generation = VmidGeneration(self.generation);
        let wrapped = self.next == Vmid::MAX.as_u16();

        self.next = if self.next == Vmid::MAX.as_u16() {
            self.generation = self.generation.wrapping_add(1);
            Vmid::MIN.as_u16()
        } else {
            self.next + 1
        };

        AllocatedVmid {
            vmid,
            generation,
            wrapped,
        }
    }
}
