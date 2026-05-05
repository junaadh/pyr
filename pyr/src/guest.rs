use pyr_arch::{
    barrier::isb,
    exception::eret,
    sysregs::{ElrEl2, SpsrEl2, sp_el1::SpEl1},
};

#[repr(align(16))]
struct GuestStack([u8; 4096]);

static mut GUEST_STACK: GuestStack = GuestStack([0; 4096]);

#[unsafe(no_mangle)]
pub extern "C" fn tiny_guest_entry() -> ! {
    debug_print_guest('A');
    debug_print_guest('B');

    loop {
        core::hint::spin_loop();
    }
}

fn debug_print_guest(ch: char) {
    let arg = ch as u64;

    // SAFETY: This intentionally performs a Pyr debug-console hypercall from EL1.
    unsafe {
        core::arch::asm!(
            "hvc #0",
            in("x0") 0x7079u64,
            in("x1") 0x1u64,
            in("x2") arg,
            lateout("x0") _,
            options(nostack),
        );
    }
}

pub fn enter_tiny_guest() -> ! {
    let entry = tiny_guest_entry as *const () as u64;

    // SAFETY: We intentionally define a 4096 array in memory and point the top to the base + len
    let stack_top = unsafe {
        let base = core::ptr::addr_of!(GUEST_STACK.0) as u64;
        base + 4096
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
