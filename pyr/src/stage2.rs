use pyr_arch::addr::{IpaAddr, PhysAddr};
use pyr_arch::barrier::{dsb_ish, isb};
use pyr_arch::page_table::{Built, MemAttr, PageTable, Stage2Tables};
use pyr_arch::sysregs::el2::{HcrEl2, VtcrEl2, VttbrEl2};

#[repr(align(4096))]
struct Tables {
    root: PageTable,
    l2: PageTable,
}

#[repr(C, align(4096))]
pub struct BootScratch {
    tables: Tables,
    guard: [u8; 4096],
    pub guest_stack: [u8; 16 * 1024],
}

#[unsafe(link_section = "__DATA,__stage2")]
pub static mut SCRATCH: BootScratch = BootScratch {
    tables: Tables {
        root: PageTable::zeroed(),
        l2: PageTable::zeroed(),
    },
    guard: [0; 4096],
    guest_stack: [0; 16 * 1024],
};

pub fn build_identity_map() -> Stage2Tables<Built> {
    // SAFETY: single-core early boot only. Stage-2 tables are built once.
    let scratch = unsafe { &mut *core::ptr::addr_of_mut!(SCRATCH) };
    let mut stage2 =
        Stage2Tables::new(&mut scratch.tables.root, &mut scratch.tables.l2);

    // QEMU virt RAM starts at 0x4000_0000.
    // Map first 1 GiB identity for now.
    stage2
        .map_range(
            IpaAddr::new(0x4000_0000),
            PhysAddr::new(0x4000_0000),
            1024 * 1024 * 1024,
            MemAttr::Normal,
        )
        .unwrap_or_else(|_| panic_stage2_map_failed());

    stage2.build()
}

fn panic_stage2_map_failed() -> ! {
    crate::log!("stage2: map_range failed");
    loop {
        core::hint::spin_loop();
    }
}

pub fn enable_stage2(root_pa: u64) {
    VtcrEl2::new()
        .with_t0sz(25)
        .with_sl0_level1()
        .with_tg0_4k()
        .with_sh0_inner()
        .with_orgn0_write_back()
        .with_irgn0_write_back()
        .msr();

    VttbrEl2::from_baddr(PhysAddr::new(root_pa)).msr();

    dsb_ish();
    isb();

    HcrEl2::mrs()
        .without_tge()
        .without_e2h()
        .with_rw()
        .with_amo()
        .with_imo()
        .with_fmo()
        .with_vm()
        .msr();

    isb();

    crate::log!("VTCR_EL2 after = {:#018x}", VtcrEl2::mrs().raw());
    crate::log!("VTTBR_EL2 after = {:#018x}", VttbrEl2::mrs().raw());
    crate::log!("HCR_EL2 stage2 = {:#018x}", HcrEl2::mrs().raw());
}
