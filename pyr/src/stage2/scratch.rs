use pyr_arch::page_table::{PageTable, PageTablePool};

use crate::guest::memory::GuestMemory;

#[repr(align(4096))]
pub struct Tables {
    pub root: PageTable,
    pub l2: PageTable,
    pub l3: PageTablePool,
}

#[repr(C, align(4096))]
pub struct BootScratch {
    pub tables: Tables,
    pub _guard: [u8; 4096],
    pub guest_stack: [u8; 16 * 1024],
    pub guest_ram: [u8; GuestMemory::GUEST_RAM_SIZE],
    pub dtb: [u8; 64 * 1024],
}

impl BootScratch {
    pub const fn zeroed() -> Self {
        Self {
            tables: Tables {
                root: PageTable::zeroed(),
                l2: PageTable::zeroed(),
                l3: PageTablePool::new(),
            },
            _guard: [0; 4096],
            guest_stack: [0; 16 * 1024],
            guest_ram: [0; GuestMemory::GUEST_RAM_SIZE],
            dtb: [0; 64 * 1024],
        }
    }
}

#[unsafe(link_section = "__DATA,__stage2")]
pub static mut SCRATCH: BootScratch = BootScratch::zeroed();

pub fn get_mut() -> &'static mut BootScratch {
    // SAFETY: Pyr is currently single-core during early boot. SCRATCH is initialized once
    // before guest execution and not aliased through another mutable reference.
    unsafe { &mut *core::ptr::addr_of_mut!(SCRATCH) }
}

pub fn guest_stack_base() -> u64 {
    // SAFETY: Taking a raw address of static storage; no reference to mutable static is created.
    unsafe {
        let scratch = &raw const SCRATCH;
        core::ptr::addr_of!((*scratch).guest_stack) as u64
    }
}

pub fn guest_stack_top() -> u64 {
    // SAFETY: Taking a raw address of static storage; no reference to mutable static is created.
    unsafe {
        let scratch = &raw const SCRATCH;
        let base = core::ptr::addr_of!((*scratch).guest_stack) as u64;
        base + 16 * 1024
    }
}

pub fn guest_ram_base() -> u64 {
    // SAFETY: Taking a raw address of static storage; no reference to mutable static is created.
    unsafe {
        let scratch = &raw const SCRATCH;
        core::ptr::addr_of!((*scratch).guest_ram) as u64
    }
}

pub fn dtb_base() -> u64 {
    // SAFETY: Taking a raw address of static storage; no reference to mutable static is created.
    unsafe {
        let scratch = &raw const SCRATCH;
        core::ptr::addr_of!((*scratch).dtb) as u64
    }
}
