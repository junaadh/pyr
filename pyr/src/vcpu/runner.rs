use crate::{guest::launch::enter_el1_guest, runtime::El2Context};

pub struct VcpuRunner;

impl VcpuRunner {
    pub fn run(cx: &mut El2Context) -> ! {
        let config = cx.runtime().vcpu().config();

        cx.install_current();

        let (_, vcpu) = cx.runtime_mut().split_mut();
        vcpu.mark_running();

        crate::log!("el1: entering guest {:?}", vcpu.id());

        enter_el1_guest(config)
    }
}
