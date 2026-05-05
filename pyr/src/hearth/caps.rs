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
