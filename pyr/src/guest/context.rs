use crate::{fatal, guest::config::GuestConfig};
use pyr_arch::{exception::TrapFrame, platform::GuestReg, reg::Gpr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuestContextError {
    InvalidRegister,
}

#[derive(Debug)]
pub struct GuestContext {
    x: [u64; 31],
    pc: u64,
    pstate: u64,
}

impl GuestContext {
    const fn new(pc: u64, pstate: u64) -> Self {
        Self {
            x: [0; 31],
            pc,
            pstate,
        }
    }

    pub fn from_config(config: GuestConfig) -> Self {
        let mut raw = Self::new(config.entry, 0);

        raw.write_x(Gpr::X0, config.x0);
        raw.write_x(Gpr::X1, config.x1);
        raw.write_x(Gpr::X2, config.x2);
        raw.write_x(Gpr::X3, config.x3);

        raw
    }

    pub fn x(&self, gpr: Gpr) -> u64 {
        *self.x.get(gpr.index()).unwrap_or_else(|| {
            fatal!("panic: this is undeterministic behaviour")
        })
    }

    pub fn write_x(&mut self, gpr: Gpr, value: u64) {
        if let Some(x) = self.x.get_mut(gpr.index()) {
            *x = value;
        }
    }

    pub const fn pc(&self) -> u64 {
        self.pc
    }

    pub const fn pstate(&self) -> u64 {
        self.pstate
    }

    pub fn read_reg(&self, reg: GuestReg) -> Result<u64, GuestContextError> {
        match reg {
            GuestReg::Zero => Ok(0),
            GuestReg::Gpr(idx) => Ok(self.x(idx)),
        }
    }

    pub fn write_reg(
        &mut self,
        reg: GuestReg,
        value: u64,
    ) -> Result<(), GuestContextError> {
        match reg {
            GuestReg::Zero => {}
            GuestReg::Gpr(idx) => self.write_x(idx, value),
        }

        Ok(())
    }

    pub fn advance_pc(&mut self) {
        self.pc = self.pc.wrapping_add(4);
    }

    pub fn sync_from_trap_frame(&mut self, frame: &TrapFrame) {
        self.x = frame.x;
        self.pc = frame.elr_el2;
        self.pstate = frame.spsr_el2;
    }

    pub fn sync_to_trap_frame(&self, frame: &mut TrapFrame) {
        frame.x = self.x;
        frame.elr_el2 = self.pc;
        frame.spsr_el2 = self.pstate;
    }
}
