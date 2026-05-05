#[derive(Debug)]
#[repr(C)]
pub struct TrapFrame {
    /// x0..x30
    pub x: [u64; 31],
    #[doc(hidden)]
    _pad: u64,
    pub elr_el2: u64,
    pub spsr_el2: u64,
}
