#[inline(always)]
pub fn isb() {
    // SAFETY: ISB is a synchronization barrier with no memory safety preconditions.
    unsafe {
        core::arch::asm!("isb", options(nomem, nostack));
    }
}

#[inline(always)]
pub fn dsb_ish() {
    // SAFETY: DSB ISH is a synchronization barrier with no memory safety preconditions.
    unsafe {
        core::arch::asm!("dsb ish", options(nomem, nostack));
    }
}

#[inline(always)]
pub fn dmb_ish() {
    // SAFETY: DMB ISH is a memory ordering barrier with no memory safety preconditions.
    unsafe {
        core::arch::asm!("dmb ish", options(nomem, nostack));
    }
}
