use pyr_alloc::context::PyrContext;

use crate::stage2::vmid::{Vmid, VmidAllocator};

pub struct HypervisorContext<'a, T> {
    pub mem: PyrContext<'a, T>,
    vmids: VmidAllocator,
}

impl<'a, A> HypervisorContext<'a, A> {
    pub const fn new(mem: PyrContext<'a, A>) -> Self {
        Self {
            mem,
            vmids: VmidAllocator::new(),
        }
    }
}

impl<A> HypervisorContext<'_, A> {
    pub fn alloc_vmid(&mut self) -> Vmid {
        self.vmids.alloc()
    }
}
