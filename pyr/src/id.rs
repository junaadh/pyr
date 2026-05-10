use core::fmt;

const fn stable_mix64(mut x: u64) -> u64 {
    x ^= x >> 30;
    x = x.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VmId(pub u64);

impl VmId {
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn as_u64(self) -> u64 {
        self.0
    }

    pub fn from_parts(stage2_root_pa: u64, guest_entry: u64) -> Self {
        Self(stable_mix64(stage2_root_pa ^ guest_entry))
    }
}

impl fmt::Debug for VmId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let short = (self.0 >> 32) as u32;

        write!(f, "vm:{short:08x}")
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VcpuId {
    vm: VmId,
    index: u16,
}

impl VcpuId {
    pub const fn from_parts(vm: VmId, index: u16) -> Self {
        Self { vm, index }
    }

    pub const fn vm(self) -> VmId {
        self.vm
    }

    pub const fn index(self) -> u16 {
        self.index
    }
}

impl fmt::Debug for VcpuId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let short = (self.vm.as_u64() >> 32) as u32;

        write!(f, "vcpu:{short:08x}:{:04x}", self.index)
    }
}
