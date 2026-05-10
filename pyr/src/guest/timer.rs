use crate::{irq::InterruptSource, vcpu::Vcpu, vm::Vm};
use pyr_arch::sysregs::el2::CntPctEl0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuestTimerKind {
    Virtual,
    Physical,
}

#[derive(Clone, Copy, Debug)]
pub struct GuestTimer {
    ctl: u32,
    cval: u64,
}

impl GuestTimer {
    pub const ENABLE: u32 = 1 << 0;
    pub const IMASK: u32 = 1 << 1;
    pub const ISTATUS: u32 = 1 << 2;

    pub const fn new() -> Self {
        Self {
            ctl: Self::IMASK,
            cval: 0,
        }
    }

    pub const fn ctl(&self) -> u32 {
        self.ctl
    }

    pub fn set_ctl(&mut self, value: u32) {
        self.ctl = value & 0b11;
    }

    pub const fn cval(&self) -> u64 {
        self.cval
    }

    pub fn set_cval(&mut self, value: u64) {
        self.cval = value;
    }

    pub const fn enabled(&self) -> bool {
        self.ctl & Self::ENABLE != 0
    }

    pub const fn masked(&self) -> bool {
        self.ctl & Self::IMASK != 0
    }

    pub fn expired(&self, counter: u64) -> bool {
        self.enabled() && counter >= self.cval
    }

    pub fn readable_ctl(&self, counter: u64) -> u32 {
        let mut ctl = self.ctl & 0b11;

        if self.expired(counter) {
            ctl |= Self::ISTATUS;
        }

        ctl
    }

    pub fn should_inject(&self, counter: u64) -> bool {
        self.expired(counter) && !self.masked()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct GuestTimers {
    virtual_timer: GuestTimer,
    physical_timer: GuestTimer,
}

impl GuestTimers {
    pub const fn new() -> Self {
        Self {
            virtual_timer: GuestTimer::new(),
            physical_timer: GuestTimer::new(),
        }
    }

    pub const fn get(&self, kind: GuestTimerKind) -> &GuestTimer {
        match kind {
            GuestTimerKind::Virtual => &self.virtual_timer,
            GuestTimerKind::Physical => &self.physical_timer,
        }
    }

    pub fn get_mut(&mut self, kind: GuestTimerKind) -> &mut GuestTimer {
        match kind {
            GuestTimerKind::Virtual => &mut self.virtual_timer,
            GuestTimerKind::Physical => &mut self.physical_timer,
        }
    }
}

pub fn evaluate_guest_timers(vm: &mut Vm, vcpu: &mut Vcpu) {
    let now = CntPctEl0::read();

    if vcpu
        .timers()
        .get(GuestTimerKind::Virtual)
        .should_inject(now)
    {
        vm.inject_irq(InterruptSource::GUEST_VTIMER_IRQ);
    }

    if vcpu
        .timers()
        .get(GuestTimerKind::Physical)
        .should_inject(now)
    {
        vm.inject_irq(InterruptSource::GUEST_PTIMER_IRQ);
    }
}
