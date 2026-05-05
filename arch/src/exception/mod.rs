mod class;
pub mod trap_frame;
pub mod vectors;

pub use class::*;
pub use trap_frame::*;
pub use vectors::*;

/// # SAFETY
///
/// Caller must ensure that `ELR_EL2` and `SPSR_EL2` are configured before calling `eret`
#[inline(always)]
pub unsafe fn eret() -> ! {
    // SAFETY: Caller must have configured ELR_EL2 and SPSR_EL2 correctly
    unsafe { core::arch::asm!("eret", options(noreturn)) }
}
