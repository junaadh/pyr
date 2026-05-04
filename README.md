# pyr

`pyr` is an experimental type-1 hypervisor written in Rust for the ARMv8-A architecture.

The current focus is early boot on QEMU's `virt` machine through UEFI, with groundwork for custom and bare-metal boot paths. The long-term goal is to run Linux as a guest.

## Status

This project is early-stage. It currently contains:

- `no_std` Rust crates for early boot code
- ARMv8-A system register helpers
- A QEMU `virt` platform implementation
- Early UART console output
- A UEFI entry point for QEMU booting

## Requirements

- Rust nightly, managed by `rust-toolchain.toml`
- `qemu-system-aarch64`
- AArch64 EDK2 firmware

The QEMU run script currently expects firmware at:

```sh
/opt/homebrew/share/qemu/edk2-aarch64-code.fd
```

If your firmware is somewhere else, update `FIRMWARE` in `scripts/run-qemu-uefi`.

## Run

Boot the UEFI image in QEMU:

```sh
./scripts/run-qemu-uefi
```

The script builds `ember` for `aarch64-unknown-uefi`, copies it to `esp/EFI/BOOT/BOOTAA64.EFI`, and starts QEMU with the FAT ESP directory attached.

## Build Checks

Check the UEFI boot path:

```sh
cargo +nightly check -p ember --target aarch64-unknown-uefi --features platform-qemu-virt
```

Check the bare-metal kernel crate:

```sh
cargo +nightly check -p pyr --target aarch64-unknown-none
```

## Project Layout

- `arch/` - ARMv8-A architecture support and system register wrappers
- `platform/qemu/` - QEMU `virt` platform support
- `pyr/` - core hypervisor entry and early runtime code
- `ember/` - UEFI boot entry that transfers control to `pyr`
- `scripts/` - development and QEMU helper scripts

## License

Licensed under either MIT or Apache-2.0.
