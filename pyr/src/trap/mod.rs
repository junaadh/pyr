mod data_abort;
mod dispatch;
mod hvc;
mod interrupt;
mod outcome;
mod psci;
mod runner;

pub use interrupt::*;
pub use outcome::*;
use pyr_arch::exception::TrapFrame;
pub use runner::*;

#[unsafe(no_mangle)]
pub extern "C" fn pyr_sync_lower_el64(frame: &mut TrapFrame) {
    TrapRunner::run_sync(frame)
}

#[unsafe(no_mangle)]
pub extern "C" fn pyr_irq_lower_el64(frame: &mut TrapFrame) {
    TrapRunner::run_interrupt(frame, InterruptKind::Irq)
}

#[unsafe(no_mangle)]
pub extern "C" fn pyr_fiq_lower_el64(frame: &mut TrapFrame) {
    TrapRunner::run_interrupt(frame, InterruptKind::Fiq)
}
