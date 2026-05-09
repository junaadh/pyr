mod linux;
mod tiny;

use crate::{ActivePlatform, fatal, log, mem};
use pyr_arch::{
    barrier::isb,
    boot::{abi::RawBootInfo, info::BootInfo},
    exception::install_el2_vectors,
    platform::Platform,
    sysregs::el2::HcrEl2,
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

    let heap = boot_info
        .hypervisor_heap()
        .unwrap_or_else(|| fatal!("Boot Info missing HypervisorHeap"));

    // SAFETY:
    //
    // BootInfo validation succeeded, and the boot handoff says this region is
    // the HypervisorHeap. During this phase we are still single-core, before
    // allocator-backed dynamic boot work. The region must be writable,
    // non-overlapping, and owned by Pyr per the RawBootInfo ABI contract.
    unsafe {
        crate::mem::HEAP.init(heap.start, heap.len);
    }

    log!(
        "mem: heap={:#x}..{:#x}",
        heap.start.as_u64(),
        heap.end.as_u64()
    );

    // SAFETY:
    //
    // This is early single-core boot. The FramePool region was supplied by the
    // validated BootInfo memory map and is expected to be writable,
    // non-overlapping memory owned by Pyr.
    let mut cx = unsafe { mem::init_allocator(&boot_info) };

    log!(
        "mem: frame free={} total={}",
        cx.free_frames(),
        cx.total_frames(),
    );

    #[cfg(feature = "boot-linux")]
    {
        use crate::boot::el2::linux::boot_linux;
        use crate::guest::linux::boot_config::LinuxBootConfig;

        let config = LinuxBootConfig::from_dev_boot_resources(&boot_info)
            .unwrap_or_else(|err| {
                fatal!("could not build dev LinuxBootConfig: {err:?}")
            });

        boot_linux(&mut cx, config)
    }

    #[cfg(feature = "boot-tiny")]
    {
        tiny::boot_tiny(&mut cx)
    }
}

fn init_el2(boot_info: &BootInfo<'_>) {
    <ActivePlatform as Platform>::early_init();
    log!("booting Pyr Hypervisor...");
    log!(
        "boot: source={:?} machine={:?} entry_el={:?}",
        boot_info.boot_source(),
        boot_info.machine(),
        boot_info.entry_el()
    );

    install_el2_vectors();

    HcrEl2::mrs()
        .without_tge()
        .without_e2h()
        .with_rw()
        .with_amo()
        .with_imo()
        .with_fmo()
        .msr();
    isb();
}
