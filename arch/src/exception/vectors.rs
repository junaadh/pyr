use crate::{addr::PhysAddr, barrier::isb, sysregs::VbarEl2};

core::arch::global_asm!(include_str!("vectors.S"));

unsafe extern "C" {
    pub static __el2_vector_table: u8;
}

pub fn install_el2_vectors() {
    let addr = {
        let ptr = &raw const __el2_vector_table;
        PhysAddr::new(ptr as u64)
    };

    VbarEl2::from_phys(addr).msr();
    isb();
}
