use pyr_arch::{
    barrier::isb,
    exception::eret,
    sysregs::{ElrEl2, SpsrEl2, sp_el1::SpEl1},
};

use crate::stage2::SCRATCH;

core::arch::global_asm!(
    r#"
    .section .text.guest, "ax"
    .align 4
    .global __tiny_guest_entry

__tiny_guest_entry:
    mov x0, #0x7079
    mov x1, #1
    mov x2, #'A'
    hvc #0x0

    mov x0, #0x7079
    mov x1, #1
    mov x2, #'B'
    hvc #0x0
    
    mov x3, #0x09000000
    mov w4, #'X'
    strb w4, [x3]

    mov x0, #0x7079
    mov x1, #1
    mov x2, #'Z'
    hvc #0x0

1:
    wfe
    b 1b
    "#
);

unsafe extern "C" {
    fn __tiny_guest_entry() -> !;
}

pub fn enter_tiny_guest() -> ! {
    let entry = __tiny_guest_entry as *const () as u64;

    // SAFETY: We intentionally define a 4096 array in memory and point the top to the base + len
    let stack_top = unsafe {
        let scratch = &raw const SCRATCH;
        let base = core::ptr::addr_of!((*scratch).guest_stack) as u64;
        base + 16 * 1024
    };

    crate::log!("entering tiney EL1 guest at {entry:#018x}");
    crate::log!("SP_EL1 = {stack_top:#018x}");

    ElrEl2::new(entry).msr();
    SpEl1::new(stack_top).msr();
    SpsrEl2::el1h_masked().msr();
    isb();

    crate::log!("ELR_EL2 = {:#018x}", ElrEl2::mrs().raw());
    crate::log!("SPSR_EL2 = {:#018x}", SpsrEl2::mrs().raw());

    // SAFETY: ELR_EL2 points to tiny_guest_entry and SPSR_EL2 is EL1h masked
    unsafe { eret() }
}
