mod linux;
mod tiny;

use crate::{ActivePlatform, fatal, log};
use pyr_arch::{
    barrier::isb,
    boot::{abi::RawBootInfo, info::BootInfo},
    exception::install_el2_vectors,
    platform::Platform,
    sysregs::{
        common::CurrentEl,
        el2::{HcrEl2, SctlrEl2, VbarEl2, VtcrEl2, VttbrEl2},
    },
};

#[cfg(all(feature = "boot-tiny", feature = "boot-linux"))]
compile_error!("Only one feature can be active at the same time");

/// # Safety
///
/// `raw` must satisfy the RawBootInfo ABI contract:
///
/// - `raw` is non-null and properly aligned
/// - `raw` points to a valid `RawBootInfo` structure
/// - all embedded pointers (`memory_map_ptr`, `modules_ptr`, etc)
///   are valid for reads for the specified lengths
/// - all referenced memory remains alive for the lifetime of this call
/// - the bootloader / firmware constructed the structure according to
///   `PYR_BOOT_VERSION` semantics
///
/// This function is the single unsafe ABI boundary between external
/// boot environments (Ember, UEFI, QEMU trampoline, tests) and Pyr's
/// internal safe Rust boot model.
pub unsafe fn pyr_el2_entry_raw(raw: *const RawBootInfo) -> ! {
    // SAFETY:
    //
    // The caller guarantees that `raw` satisfies the RawBootInfo ABI
    // invariants documented above. `from_raw_ptr` validates:
    //
    // - magic
    // - version
    // - structure size
    // - slice bounds
    // - enum discriminants
    // - UTF-8 command lines / module names
    //
    // and converts the raw ABI representation into a validated safe
    // `BootInfo<'_>` view.
    let boot_info =
        unsafe { BootInfo::from_raw_ptr(raw) }.unwrap_or_else(|err| {
            fatal!("could not parse RawBootInfo into BootInfo: {err:?}")
        });

    pyr_el2_entry(boot_info)
}

fn pyr_el2_entry(boot_info: BootInfo<'_>) -> ! {
    init_el2(&boot_info);

    #[cfg(feature = "boot-linux")]
    {
        use crate::boot::el2::linux::boot_linux;

        let kernel = boot_info
            .kernel()
            .unwrap_or_else(|| fatal!("BootInfo missing Linux kernel module"));

        let dtb = boot_info
            .dtb()
            .unwrap_or_else(|| fatal!("BootInfo missing DTB module"));

        let initrd = boot_info.initrd().map(|m| m.data());

        // SAFETY:
        //
        // `boot_linux` assumes:
        //
        // - kernel image bytes are a valid AArch64 Linux Image
        // - DTB bytes are a valid flattened device tree blob
        // - initrd bytes (if present) remain alive during guest setup
        // - the stage-2 mapper will establish the required guest-visible
        //   mappings before entering EL1
        //
        // These invariants are upheld by the validated BootInfo module
        // model and the boot artifact construction performed by the
        // bootloader / bare-metal trampoline.
        boot_linux(kernel.data(), dtb.data(), initrd)
    }

    #[cfg(feature = "boot-tiny")]
    {
        tiny::boot_tiny()
    }
}

fn init_el2(boot_info: &BootInfo<'_>) {
    <ActivePlatform as Platform>::early_init();
    log!("booting");
    log!("boot source = {:?}", boot_info.boot_source());
    log!("machine     = {:?}", boot_info.machine());
    log!("entry EL    = {:?}", boot_info.entry_el());

    install_el2_vectors();

    log!("VBAR_EL2 = {:#018x}", VbarEl2::mrs().raw());

    let el = CurrentEl::mrs();
    log!("CurrentEL = {:#018x}", el.raw());
    log!("Exception level = EL{}", el.exception_level());

    let hcr = HcrEl2::mrs();
    let sctlr = SctlrEl2::mrs();

    log!("HCR_EL2 = {:#018x}", hcr.raw());
    log!("SCTLR_EL2 = {:#018x}", sctlr.raw());
    log!("SCTLR_EL2.M = {}", sctlr.mmu_enabled());

    HcrEl2::mrs()
        .without_tge()
        .without_e2h()
        .with_rw()
        .with_amo()
        .with_imo()
        .with_fmo()
        .msr();
    isb();

    log!("HCR_EL2 after RW = {:#018x}", HcrEl2::mrs().raw());

    log!("VTCR_EL2 = {:#018x}", VtcrEl2::mrs().raw());
    log!("VTTBR_EL2 = {:#018x}", VttbrEl2::mrs().raw());
}
