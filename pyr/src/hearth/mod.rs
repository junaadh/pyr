use crate::trap::Resume;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Scope {
    GuestConsoleWrite,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct CapSet {
    guest_console_write: bool,
}

impl CapSet {
    pub const fn debug_guest() -> Self {
        Self {
            guest_console_write: true,
        }
    }

    pub const fn allows(self, scope: Scope) -> bool {
        match scope {
            Scope::GuestConsoleWrite => self.guest_console_write,
        }
    }
}

pub fn handle_hvc(imm16: u16) -> Resume {
    let caps = CapSet::debug_guest();

    match imm16 {
        0 => {
            if caps.allows(Scope::GuestConsoleWrite) {
                crate::log!("hearth.debug_console: HVC accepted");

                Resume::Halt
            } else {
                crate::log!("hearth.debug_console: denied");

                Resume::Halt
            }
        }

        _ => {
            crate::log!("unknown HVC imm16 = {:#06x}", imm16);

            Resume::Halt
        }
    }
}
