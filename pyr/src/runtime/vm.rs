use crate::{vcpu::Vcpu, vm::Vm};

pub struct VmRuntime {
    vm: Vm,
    boot_vcpu: Vcpu,
}

impl VmRuntime {
    pub const fn new(vm: Vm, boot_vcpu: Vcpu) -> Self {
        Self { vm, boot_vcpu }
    }

    pub fn split_mut(&mut self) -> (&mut Vm, &mut Vcpu) {
        (&mut self.vm, &mut self.boot_vcpu)
    }

    pub const fn vm(&self) -> &Vm {
        &self.vm
    }

    pub const fn vcpu(&self) -> &Vcpu {
        &self.boot_vcpu
    }

    pub const fn vm_mut(&mut self) -> &mut Vm {
        &mut self.vm
    }

    pub const fn vcpu_mut(&mut self) -> &mut Vcpu {
        &mut self.boot_vcpu
    }
}
