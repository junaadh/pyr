mod data_abort;
mod dispatch;
mod hvc;
mod psci;
mod resume;
mod runner;

use pyr_arch::exception::TrapFrame;
pub use resume::*;
pub use runner::*;

#[unsafe(no_mangle)]
pub extern "C" fn pyr_sync_lower_el64(frame: &mut TrapFrame) {
    TrapRunner::run(frame)
}
