use crate::{ActivePlatform, trap::Resume};
use pyr_arch::{
    addr::IpaAddr,
    exception::{DataAbortIss, TrapFrame},
    platform::Platform,
    sysregs::el2::{FarEl2, HpfarEl2},
};

pub fn handle_data_abort(frame: &mut TrapFrame, iss: DataAbortIss) -> Resume {
    let far = FarEl2::mrs();
    let hpfar = HpfarEl2::mrs();

    let ipa = hpfar.ipa_base().as_u64() | (far.raw() & 0xfff);

    // crate::log!("trap = DataAbortLower");
    // crate::log!("fault IPA = {ipa:#018x}");

    match ActivePlatform::mmio_emulate(IpaAddr::new(ipa), frame, iss) {
        Ok(()) => Resume::AdvancePcAndReturn,
        Err(err) => {
            crate::log!("mmio.error: {err:?} @ {ipa:#018x}");

            Resume::Halt
        }
    }
}
