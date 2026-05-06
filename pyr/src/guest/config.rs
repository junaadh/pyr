#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct GuestConfig {
    pub entry: u64,
    pub stack_top: u64,
    pub x0: u64,
    pub x1: u64,
    pub x2: u64,
    pub x3: u64,
}

impl GuestConfig {
    pub const fn new(entry: u64, stack_top: u64) -> Self {
        Self {
            entry,
            stack_top,
            x0: 0,
            x1: 0,
            x2: 0,
            x3: 0,
        }
    }

    pub const fn with_linux_dtb(mut self, dtb: u64) -> Self {
        self.x0 = dtb;
        self.x1 = 0;
        self.x2 = 0;
        self.x3 = 0;
        self
    }
}
