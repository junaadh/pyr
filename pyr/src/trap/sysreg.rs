use pyr_arch::{exception::SysRegIss, reg::Gpr, sysregs::el2::CntPctEl0};

use crate::{
    guest::timer::GuestTimerKind,
    trap::TrapOutcome,
    vcpu::{Vcpu, VcpuExitReason},
    vm::Vm,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrappedSysReg {
    CntvCtlEl0,
    CntvCvalEl0,
    CntvTvalEl0,
    CntpTvalEl0,
    CntpCtlEl0,
    CntpCvalEl0,
    Unknown(SysRegIss),
}

impl TrappedSysReg {
    pub const fn from_iss(iss: SysRegIss) -> Self {
        match (iss.op0, iss.op1, iss.crn, iss.crm, iss.op2) {
            // AArch64 timer regs. If one mapping is wrong, log raw ISS and adjust.
            (3, 3, 14, 3, 1) => Self::CntvCtlEl0,
            (3, 3, 14, 3, 2) => Self::CntvCvalEl0,
            (3, 3, 14, 3, 0) => Self::CntvTvalEl0,
            (3, 3, 14, 2, 0) => Self::CntpTvalEl0,
            (3, 3, 14, 2, 1) => Self::CntpCtlEl0,
            (3, 3, 14, 2, 2) => Self::CntpCvalEl0,
            _ => Self::Unknown(iss),
        }
    }
}

pub fn handle(vm: &mut Vm, vcpu: &mut Vcpu, iss: SysRegIss) -> TrapOutcome {
    match TrappedSysReg::from_iss(iss) {
        TrappedSysReg::CntvCtlEl0 => {
            handle_timer_ctl(vcpu, GuestTimerKind::Virtual, iss)
        }
        TrappedSysReg::CntvCvalEl0 => {
            handle_timer_cval(vcpu, GuestTimerKind::Virtual, iss)
        }
        TrappedSysReg::CntvTvalEl0 => {
            handle_timer_tval(vcpu, GuestTimerKind::Virtual, iss)
        }

        TrappedSysReg::CntpCtlEl0 => {
            handle_timer_ctl(vcpu, GuestTimerKind::Physical, iss)
        }
        TrappedSysReg::CntpCvalEl0 => {
            handle_timer_cval(vcpu, GuestTimerKind::Physical, iss)
        }
        TrappedSysReg::CntpTvalEl0 => {
            handle_timer_tval(vcpu, GuestTimerKind::Physical, iss)
        }

        TrappedSysReg::Unknown(_) => {
            crate::log!(
                "trap: unknown sysreg {:?} {:?}: iss={:#x}",
                vm.id(),
                vcpu.id(),
                iss.raw,
            );

            TrapOutcome::Exit(VcpuExitReason::UnknownSysReg)
        }
    }
}

fn rt(iss: SysRegIss) -> Option<Gpr> {
    Gpr::from_u8(iss.rt)
}

fn handle_timer_ctl(
    vcpu: &mut Vcpu,
    kind: GuestTimerKind,
    iss: SysRegIss,
) -> TrapOutcome {
    let Some(rt) = rt(iss) else {
        return TrapOutcome::Exit(VcpuExitReason::UnknownSysReg);
    };

    let now = CntPctEl0::read();

    if iss.is_write {
        let value = vcpu.context().x(rt) as u32;
        vcpu.timers_mut().get_mut(kind).set_ctl(value);
    } else {
        let value = vcpu.timers().get(kind).readable_ctl(now) as u64;
        vcpu.context_mut().write_x(rt, value);
    }

    TrapOutcome::AdvancePc
}

fn handle_timer_cval(
    vcpu: &mut Vcpu,
    kind: GuestTimerKind,
    iss: SysRegIss,
) -> TrapOutcome {
    let Some(rt) = rt(iss) else {
        return TrapOutcome::Exit(VcpuExitReason::UnknownSysReg);
    };

    if iss.is_write {
        let value = vcpu.context().x(rt);
        vcpu.timers_mut().get_mut(kind).set_cval(value);
    } else {
        let value = vcpu.timers().get(kind).cval();
        vcpu.context_mut().write_x(rt, value);
    }

    TrapOutcome::AdvancePc
}

fn handle_timer_tval(
    vcpu: &mut Vcpu,
    kind: GuestTimerKind,
    iss: SysRegIss,
) -> TrapOutcome {
    let Some(rt) = rt(iss) else {
        return TrapOutcome::Exit(VcpuExitReason::UnknownSysReg);
    };

    let now = CntPctEl0::read();

    if iss.is_write {
        let delta = vcpu.context().x(rt);
        vcpu.timers_mut()
            .get_mut(kind)
            .set_cval(now.wrapping_add(delta));
    } else {
        let cval = vcpu.timers().get(kind).cval();
        vcpu.context_mut().write_x(rt, cval.wrapping_sub(now));
    }

    TrapOutcome::AdvancePc
}
