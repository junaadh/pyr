use core::ptr::NonNull;
use pyr_arch::sysregs::el2::TpidrEl2;

use crate::{fatal, vcpu::Vcpu, vm::Vm};

pub struct El2Context {
    vm: Vm,
    boot_vcpu: Vcpu,
}

impl El2Context {
    pub const fn new(vm: Vm, boot_vcpu: Vcpu) -> Self {
        Self { vm, boot_vcpu }
    }

    pub fn install_current(&mut self) {
        let ptr = NonNull::from(self).as_ptr() as u64;

        // SAFETY:
        // Pyr calls this only after EL2 boot has completed and before entering EL1.
        // The pointer is derived from `&mut self`, and `run_vcpu` never returns,
        // so the context outlives all traps from the launched guest on this CPU.
        unsafe {
            TpidrEl2::write_raw(ptr);
        }
    }

    pub fn current() -> &'static mut Self {
        // SAFETY:
        // Pyr installs `TPIDR_EL2` through `install_current` before guest entry.
        // Trap handlers only call this after an EL1->EL2 exception.
        let raw = unsafe { TpidrEl2::read_raw() };

        let ptr = NonNull::new(raw as *mut Self).unwrap_or_else(|| {
            fatal!("TPIDR_EL2 does not contain an active EL2 context")
        });

        // SAFETY:
        // The pointer came from `install_current`.
        // On the current single-vCPU execution model, traps are synchronous and
        // re-enter the same CPU-local context.
        unsafe { ptr.as_ptr().as_mut().unwrap_unchecked() }
    }

    pub fn split_mut(&mut self) -> (&mut Vm, &mut Vcpu) {
        (&mut self.vm, &mut self.boot_vcpu)
    }

    pub const fn vm(&self) -> &Vm {
        &self.vm
    }

    pub const fn vpcu(&self) -> &Vcpu {
        &self.boot_vcpu
    }
}
