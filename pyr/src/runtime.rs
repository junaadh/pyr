use crate::{
    fatal,
    id::VcpuId,
    runtime::{
        scheduler::{Scheduler, SchedulerDecision},
        vm::VmRuntime,
    },
    vcpu::Vcpu,
    vm::Vm,
};
use core::ptr::NonNull;
use pyr_arch::sysregs::el2::TpidrEl2;

pub mod scheduler;
pub mod vm;

pub struct El2Context {
    runtime: VmRuntime,
    scheduler: Scheduler,
}

impl El2Context {
    pub const fn from_vm(vm: Vm, boot_vcpu: Vcpu) -> Self {
        Self {
            runtime: VmRuntime::new(vm, boot_vcpu),
            scheduler: Scheduler::new(),
        }
    }

    pub const fn new(runtime: VmRuntime) -> Self {
        Self {
            runtime,
            scheduler: Scheduler::new(),
        }
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

    pub const fn runtime(&self) -> &VmRuntime {
        &self.runtime
    }

    pub const fn runtime_mut(&mut self) -> &mut VmRuntime {
        &mut self.runtime
    }

    pub const fn scheduler_mut(&mut self) -> &mut Scheduler {
        &mut self.scheduler
    }

    pub fn on_vcpu_blocked(&mut self, _vcpu_id: VcpuId) -> SchedulerDecision {
        // FIXME: later wen multiple vcpu
        self.scheduler.on_blocked(self.runtime.vcpu())
    }

    pub fn on_vcpu_exited(&mut self, _vcpu_id: VcpuId) -> SchedulerDecision {
        // FIXME: later wen multiple vcpu
        self.scheduler.on_exited(self.runtime.vcpu())
    }
}
