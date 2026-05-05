#[derive(Debug)]
#[repr(C)]
pub struct TrapFrame {
    /// x0..x7 for early HVC abi
    pub x: [u64; 8],
    pub elr_el2: u64,
    pub spsr_el2: u64,
}
