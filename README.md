# pyr

`pyr` is an experimental, early-stage type-1 hypervisor for ARMv8-A written in `no_std` Rust.

The current codebase focuses on booting at EL2 on QEMU's AArch64 `virt` machine, installing an EL2 exception vector table, enabling a minimal stage-2 translation setup, and entering a tiny in-tree EL1 guest. The long-term direction is to grow this into a hypervisor capable of running Linux guests, but the current implementation is still a small bring-up environment rather than a production VMM.

This README is based on the repository's tracked and non-gitignored files only.

## Current Status

The project currently provides:

- A Rust workspace with four `no_std` crates: `pyr-arch`, `pyr-platform-qemu`, `pyr`, and `ember`.
- QEMU `virt` platform support with early PL011 UART output at physical address `0x0900_0000`.
- A UEFI boot path through `ember`, producing an `aarch64-unknown-uefi` EFI binary.
- A bare-metal `pyr` entry point for `aarch64-unknown-none`.
- ARMv8-A address wrappers, synchronization barriers, EL2 system register wrappers, and exception helpers.
- An EL2 vector table implemented in assembly and installed through `VBAR_EL2`.
- Minimal stage-2 page table construction using 4 KiB tables and 2 MiB block descriptors.
- A 1 GiB identity map for QEMU RAM starting at `0x4000_0000`.
- A tiny EL1 guest assembled into the hypervisor image.
- Basic trap handling for HVC calls and a narrow PL011 MMIO write emulation path.
- A small internal HVC ABI named `hearth` for debug-console output.

Notable current limitations:

- Only QEMU `virt` is implemented as a concrete platform.
- The guest is a tiny in-tree assembly routine, not Linux.
- Stage-2 mapping is static, identity-mapped, and limited to the current early boot use case.
- Most exception vector entries halt in a `wfe` loop; only lower EL AArch64 synchronous exceptions are dispatched into Rust.
- There is no allocator, scheduler, device model, vCPU abstraction, guest loader, or persistent configuration layer yet.
- There are no test files in the current non-gitignored source tree.

## Repository Layout

```text
.
|-- Cargo.toml                 # Workspace manifest and shared lint/profile settings
|-- Cargo.lock                 # Lockfile for the local workspace crates
|-- rust-toolchain.toml        # Nightly toolchain, components, and AArch64 targets
|-- .cargo/config.toml         # Target rustflags and linker script for bare-metal builds
|-- arch/                      # `pyr-arch`: ARMv8-A architecture support
|-- ember/                     # `ember`: UEFI entry crate that jumps into `pyr`
|-- platform/qemu/             # `pyr-platform-qemu`: QEMU virt platform support
|-- pyr/                       # `pyr`: hypervisor runtime, traps, guest, stage-2 setup
`-- scripts/run-qemu-uefi      # Builds the UEFI image and boots it in QEMU
```

Ignored/generated paths include `target/`, `esp/`, `generated/`, `.venv/`, `docs/DDI0487F_a_armv8_arm.pdf`, `armv8-book/`, and `**/*.py` according to `.gitignore`. The QEMU run script creates `esp/EFI/BOOT/BOOTAA64.EFI` as a generated boot artifact.

## Workspace

The root `Cargo.toml` defines a Rust 2024 workspace:

| Crate | Package name | Purpose |
| --- | --- | --- |
| `arch/` | `pyr-arch` | Architecture-level ARMv8-A primitives shared by the hypervisor and platform crates. |
| `platform/qemu/` | `pyr-platform-qemu` | QEMU `virt` platform implementation, currently early UART output. |
| `pyr/` | `pyr` | Core hypervisor runtime and bare-metal entry point. |
| `ember/` | `ember` | UEFI entry point that calls into `pyr`. |

The workspace uses only local crates. `Cargo.lock` contains no third-party dependencies.

Shared package metadata:

- Version: `0.1.0`
- Edition: `2024`
- License: `MIT OR Apache-2.0`
- Repository: `https://github.com/junaadh/pyr.git`

Shared lint policy in `Cargo.toml`:

- `unsafe_op_in_unsafe_fn = "deny"`
- `clippy::undocumented_unsafe_blocks = "deny"`
- `clippy::unwrap_used = "deny"`
- `clippy::expect_used = "deny"`
- `clippy::indexing_slicing = "deny"`

Both dev and release profiles set `panic = "abort"`. The release profile also enables LTO, uses one codegen unit, optimizes for size, and strips symbols.

## Toolchain

`rust-toolchain.toml` pins the project to nightly Rust and installs:

- Components: `rust-src`, `llvm-tools-preview`, `clippy`, `rustfmt`, `rust-analyzer`
- Targets: `aarch64-unknown-none`, `aarch64-unknown-uefi`

The project is intended to be built with `cargo +nightly`, although `rust-toolchain.toml` should select nightly automatically when commands are run from the repository root.

## Requirements

To run the current QEMU UEFI path, install:

- Rust nightly with the targets from `rust-toolchain.toml`
- `qemu-system-aarch64`
- AArch64 EDK2 firmware

The run script currently expects firmware at:

```sh
/opt/homebrew/share/qemu/edk2-aarch64-code.fd
```

If your firmware is located elsewhere, update `FIRMWARE` in `scripts/run-qemu-uefi`.

## Quick Start

Boot the UEFI image in QEMU:

```sh
./scripts/run-qemu-uefi
```

The script performs these steps:

1. Builds `ember` for `aarch64-unknown-uefi` in release mode with `platform-qemu-virt` enabled.
2. Creates the generated EFI system partition directory at `esp/EFI/BOOT`.
3. Copies `target/aarch64-unknown-uefi/release/ember.efi` to `esp/EFI/BOOT/BOOTAA64.EFI`.
4. Starts `qemu-system-aarch64` with `-machine virt,virtualization=on`, `-cpu cortex-a72`, `-m 1024M`, `-nographic`, EDK2 firmware, and the FAT ESP directory attached as a virtio block device.

## Build And Check Commands

Check the UEFI boot crate:

```sh
cargo +nightly check -p ember --target aarch64-unknown-uefi --features platform-qemu-virt
```

Check the bare-metal hypervisor crate:

```sh
cargo +nightly check -p pyr --target aarch64-unknown-none
```

Check the architecture crate:

```sh
cargo +nightly check -p pyr-arch --target aarch64-unknown-none
```

Check the QEMU platform crate:

```sh
cargo +nightly check -p pyr-platform-qemu --target aarch64-unknown-none
```

Format the workspace:

```sh
cargo +nightly fmt
```

Run clippy for the bare-metal hypervisor path:

```sh
cargo +nightly clippy -p pyr --target aarch64-unknown-none
```

## Boot Flow

The current QEMU UEFI flow is:

```text
scripts/run-qemu-uefi
  -> cargo builds `ember.efi`
  -> QEMU starts EDK2 firmware
  -> EDK2 launches `EFI/BOOT/BOOTAA64.EFI`
  -> ember::efi_main()
  -> pyr::pyr_entry()
  -> pyr::<QemuVirt>()
  -> QemuVirt early console init
  -> install EL2 vectors in VBAR_EL2
  -> inspect CurrentEL, HCR_EL2, SCTLR_EL2, VTCR_EL2, VTTBR_EL2
  -> configure HCR_EL2 for an AArch64 EL1 guest
  -> build a static stage-2 identity map
  -> enable stage-2 translation through VTCR_EL2, VTTBR_EL2, HCR_EL2.VM
  -> enter the tiny EL1 guest with ERET
```

The UEFI crate `ember` defines `efi_main` with the `efiapi` ABI and immediately calls `pyr_entry()`. The bare-metal `pyr` binary defines `_start()` and also calls `pyr_entry()`.

## Runtime Flow

The main hypervisor runtime is `pyr::pyr<P>()`, where `P` implements `pyr_arch::platform::Platform`.

The default feature is `platform-qemu-virt`, which selects `pyr_platform_qemu::QemuVirt` as the active platform.

At startup, `pyr`:

1. Calls `P::early_init()`.
2. Initializes the early console with `P::early_putc`.
3. Installs EL2 exception vectors.
4. Logs key EL2 registers.
5. Updates `HCR_EL2` to clear `TGE` and `E2H`, set `RW`, and route physical IRQ, FIQ, and SError to EL2.
6. Builds the stage-2 page tables.
7. Enables stage-2 translation.
8. Enters the tiny guest at EL1h with DAIF masked.

## Crate Details

### `pyr-arch`

`pyr-arch` is a `no_std` architecture support crate.

Main modules:

| Module | Contents |
| --- | --- |
| `addr` | Transparent wrappers for physical addresses, virtual addresses, and intermediate physical addresses. |
| `barrier` | `isb`, `dsb ish`, and `dmb ish` wrappers using inline assembly. |
| `exception` | Exception classes, trap frame layout, `eret`, and EL2 vector installation. |
| `page` | Page constants for 4 KiB pages and 512-entry translation tables. |
| `page_table` | Stage-2 descriptor and page table construction helpers. |
| `platform` | The `Platform` trait used by `pyr` for early platform operations. |
| `sysregs` | Typed wrappers for ARM EL2 system registers used by the hypervisor. |

Address wrappers:

- `PhysAddr`: physical address wrapper with `new`, `as_u64`, `offset`, `align_down`, and `align_up`.
- `IpaAddr`: intermediate physical address wrapper with `new`, `as_u64`, `offset`, `align_down`, and `align_up`.
- `VirtAddr`: virtual address wrapper with `new` and `as_u64`.

System register wrappers currently cover:

- `CurrentEl`
- `ElrEl2`
- `EsrEl2`
- `FarEl2`
- `HcrEl2`
- `HpfarEl2`
- `SctlrEl2`
- `SpEl1`
- `SpsrEl2`
- `VbarEl2`
- `VtcrEl2`
- `VttbrEl2`

### `pyr-platform-qemu`

`pyr-platform-qemu` defines `QemuVirt`, the only concrete platform currently implemented.

`QemuVirt` implements `pyr_arch::platform::Platform`:

- `early_init()` is currently empty.
- `early_putc(byte)` writes one byte to the QEMU `virt` PL011 UART MMIO base at physical address `0x0900_0000` with `write_volatile`.

### `pyr`

`pyr` contains the core hypervisor runtime.

Main modules:

| Module | Purpose |
| --- | --- |
| `console` | Global early console callback and `core::fmt::Write` adapter. |
| `guest` | Tiny in-tree EL1 guest and EL1 entry setup. |
| `hearth` | Internal HVC ABI dispatch and debug console capability checks. |
| `stage2` | Static stage-2 table storage, identity map construction, and stage-2 enablement. |
| `trap` | Lower EL AArch64 synchronous trap handler. |

The crate exports logging macros:

- `print!`
- `println!`
- `debug!`
- `log!`

`pyr/src/main.rs` provides a `no_std`, `no_main` bare-metal `_start()` that calls `pyr::pyr_entry()`. Its panic handler prints `[pyr] panic` and spins.

### `ember`

`ember` is a `no_std`, `no_main` UEFI entry crate.

It defines:

```rust
pub extern "efiapi" fn efi_main(_image: Handle, _st: SytemTable) -> usize
```

`efi_main` calls `pyr_entry()` and never returns in normal operation. The panic handler spins.

## Stage-2 Translation

Stage-2 page table support lives in `arch/src/page_table` and `pyr/src/stage2.rs`.

Current model:

- Tables are 4 KiB aligned and contain 512 descriptors.
- `Descriptor::table(addr)` creates valid table descriptors.
- `Descriptor::block(addr, attr)` creates stage-2 block descriptors.
- Block descriptors currently set full access, inner shareable, access flag, and an attribute index.
- `MemAttr::Normal` uses attribute index `0`.
- `MemAttr::Device` uses attribute index `1`.
- `Stage2Tables<Building>` enforces a build state before producing `Stage2Tables<Built>`.
- `map_range` maps 2 MiB blocks and rejects unaligned addresses, unaligned sizes, out-of-range L1 spans, and duplicate mappings.

The current hypervisor creates a statically allocated `BootScratch` area with:

- One root page table.
- One L2 page table.
- One 4 KiB guard area.
- One 16 KiB guest stack.

`build_identity_map()` maps the first 1 GiB of QEMU RAM:

| IPA | PA | Size | Attribute |
| --- | --- | --- | --- |
| `0x4000_0000` | `0x4000_0000` | `1024 * 1024 * 1024` | `Normal` |

`enable_stage2(root_pa)` configures:

- `VTCR_EL2.T0SZ = 25`
- `VTCR_EL2.SL0` for level 1 start with a 4 KiB granule
- `VTCR_EL2.TG0 = 4 KiB`
- `VTCR_EL2.SH0 = inner shareable`
- `VTCR_EL2.ORGN0 = write-back read/write allocate`
- `VTCR_EL2.IRGN0 = write-back read/write allocate`
- `VTTBR_EL2` with the root table base address
- `HCR_EL2.VM = 1`

## Tiny Guest

The tiny guest is assembled in `pyr/src/guest.rs` as `__tiny_guest_entry`.

Guest behavior:

1. Calls `hvc #0` with extension `0x7079`, function `1`, and byte `'A'`.
2. Calls `hvc #0` with extension `0x7079`, function `1`, and byte `'B'`.
3. Writes byte `'X'` directly to IPA `0x0900_0000`, the QEMU PL011 UART address.
4. Calls `hvc #0` with extension `0x7079`, function `1`, and byte `'Z'`.
5. Enters a `wfe` loop.

`enter_tiny_guest()` sets:

- `ELR_EL2` to the tiny guest entry symbol.
- `SP_EL1` to the top of the 16 KiB scratch guest stack.
- `SPSR_EL2` to EL1h with DAIF masked.

It then executes `eret` to enter the guest.

## Exceptions And Trap Handling

`arch/src/exception/vectors.S` defines the EL2 vector table.

Only the lower EL AArch64 synchronous vector currently saves state into a `TrapFrame` and calls Rust:

```text
pyr_sync_lower_el64(&mut TrapFrame)
```

The trap frame stores:

- `x0..x30`
- `ELR_EL2`
- `SPSR_EL2`

`pyr/src/trap/mod.rs` reads and decodes `ESR_EL2`.

Handled cases:

| Exception | Behavior |
| --- | --- |
| HVC64 | Dispatches to `hearth::handle_hvc` and returns to the guest on success. |
| DataAbortLower | If it is an 8-bit write to IPA `0x0900_0000`, prints the guest byte, advances `ELR_EL2` by 4, and returns. |

Unhandled traps log the decoded class and halt in a spin loop. Most vector table entries that are not wired to Rust halt in `wfe` loops.

## Hearth HVC ABI

`hearth` is the current internal hypercall dispatch layer.

The HVC call format is read from the saved guest registers:

| Register | Meaning |
| --- | --- |
| `x0` | Extension ID |
| `x1` | Function ID |
| `x2` | Argument 0 |
| `x3` | Argument 1 |
| `x4` | Argument 2 |

The only implemented extension is debug console:

| Name | Value |
| --- | --- |
| Extension ID | `0x7079` |
| Function ID `Putc` | `0x0001` |

`DebugConsole::Putc` requires `Scope::GuestConsoleWrite`. The current dispatcher grants `CapSet::debug_guest()`, which allows guest console writes.

Return and error behavior:

- On success, `x0` is set to `0` and the guest resumes.
- Unknown extension returns error code `1` and halts.
- Unknown function returns error code `2` and halts.
- Permission denied returns error code `3` and halts.

## Memory Map And Important Addresses

| Address | Use |
| --- | --- |
| `0x0900_0000` | QEMU `virt` PL011 UART MMIO base. |
| `0x4000_0000` | QEMU `virt` RAM base used by the stage-2 identity map. |
| `0x4008_0000` | Bare-metal `pyr` image link address from `platform/qemu/pyr.link.ld`. |

The linker script for `aarch64-unknown-none` is configured by `.cargo/config.toml` and points at `platform/qemu/pyr.link.ld`. It defines `_start` as the entry point, places `.text`, `.rodata`, `.data`, and `.bss` from `0x40080000`, and exposes `__bss_start`, `__bss_end`, and `__image_end` symbols.

## Features

The `pyr` and `ember` crates define the same platform feature:

| Feature | Default | Effect |
| --- | --- | --- |
| `platform-qemu-virt` | Yes | Enables the optional `pyr-platform-qemu` dependency and selects QEMU `virt` support. |

## Development Notes

- The codebase is intentionally `no_std`.
- The UEFI path is `no_main` through `ember::efi_main`.
- The bare-metal path is `no_main` through `pyr::_start`.
- Unsafe blocks are documented and `unsafe_op_in_unsafe_fn` is denied.
- Early console state is a single global callback and is documented as single-core early boot only.
- The platform abstraction is intentionally small: `early_init`, `early_putc`, and default `early_print`.
- Generated QEMU boot files are written under ignored `esp/`.

## License

Licensed under either MIT or Apache-2.0.
