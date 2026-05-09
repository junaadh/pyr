use pyr_arch::addr::IpaAddr;

use crate::stage2::vmid::Vmid;

pub struct Stage2Invalidation;

impl Stage2Invalidation {
    pub fn flush_all() {
        // SAFETY:
        //
        // This executes at EL2 after stage-2 translation has been configured
        // `vmalls12elis` invalidates all stage-2 translations for the current VMID regime
        unsafe {
            core::arch::asm!(
                "dsb ishst",
                "tlbi vmalls12e1is",
                "dsb ish",
                "isb",
                options(nostack, preserves_flags),
            );
        }
    }

    pub fn flush_ipa(_ipa: IpaAddr) {
        // FIXME: For now, using global stage-2 flush until
        // Pyr has VMID-aware targeted invalidation
        Self::flush_all();
    }

    pub fn flush_vmid(_vmid: Vmid) {
        // FIXME: For now, using global stage-2 flush until
        // Pyr has TLBI encoding is added
        Self::flush_all();
    }
}
