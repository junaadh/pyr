use pyr_arch::sysregs::common::CurrentEl;

pub mod el2;
pub mod el3;

pub fn enter() -> ! {
    match CurrentEl::mrs().exception_level() {
        3 => el3::transition_to_el2(el2::pyr_entry as *const () as u64),
        2 => el2::pyr_entry(),
        level => panic!("unsupported EL{level}"),
    }
}
