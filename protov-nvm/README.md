# protov-nvm

Non-volatile memory partition map and linker scripts for ProtoV.
- **2 MiB XIP flash**: W25Q16

Three hand-maintained copies of the same layout:

| File | Purpose |
|------|---------|
| `linker/memory-bootloader.x` | Full linker script for feature `bootloader` |
| `linker/memory-application.x` | Full linker script for feature `application` |
| `src/layout.rs` | Rust `const`s for compile-time and host tests |

`build.rs` copies the chosen `.x` file to `OUT_DIR/memory.x`.

## Flash layout

The application and bootloader layouts are based on Embassy's RP2040
[application](https://github.com/embassy-rs/embassy/blob/main/examples/boot/application/rp/memory.x)
and
[bootloader](https://github.com/embassy-rs/embassy/blob/main/examples/boot/bootloader/rp/memory.x)
examples.

| Region / symbols | Purpose |
| --- | --- |
| `BOOT2` | RP2040 second-stage boot code at the beginning of external flash. |
| Bootloader `FLASH` | The ProtoV bootloader image. This region exists only as executable `FLASH` in the bootloader linker script. |
| `BOOTLOADER_STATE` / `__bootloader_state_*` | Persistent [Embassy Boot](https://docs.rs/embassy-boot-rp/0.10.0/embassy_boot_rp/) update state. It records swap progress so update status is preserved and recovery can continue after power loss. |
| `ACTIVE` / application `FLASH` / `__bootloader_active_*` | The current application firmware image. The bootloader calls this region `ACTIVE`; the application links itself into the same addresses as `FLASH`. |
| `DFU` / `__bootloader_dfu_*` | Staging area for a new device firmware image before the bootloader copies or swaps it into `ACTIVE`. It is one erase page larger than the active image as required by the update algorithm. |
| `RESERVED` / `__reserved_*` | Currently unallocated space reserved for user-defined future features. ProtoV product features must not consume it. |
| `CONFIG` / `__config_*` | Space reserved for persistent user configuration support to be added in a future release. |
| `FACTORY` / `__factory_*` | Factory-programmed state, including serial identity, hardware revision, manufacturing date, flash identity, and attestation data. |
| `RAM` | RP2040 runtime memory; it is not part of persistent flash storage. |

> [!NOTE]
> The `RESERVED` partition is set aside for user applications and custom data.
> ProtoV product features must not allocate or depend on this region.

If a contribution requires a new NVM allocation, open an issue or discussion
before changing the layout. Partition changes can break bootloader and software
DFU compatibility and therefore require a major version increment. See the
[contribution guidelines](../CONTRIBUTING.md#breaking-changes-2xx).


## Keeping the layout in sync

Update all three files together. Run `just test` (here) or `just nvm::test` (repo root) after changes.

The only intentional difference between the two `.x` files: **`FLASH`** is the bootloader slot in one, the ACTIVE slot in the other. Symbol block is duplicated; `__bootloader_active_*` uses `ACTIVE` (bootloader) or `FLASH` (application).

## Features

| Feature | Linker input |
|---------|--------------|
| `bootloader` | `linker/memory-bootloader.x` |
| `application` | `linker/memory-application.x` |

## Usage

```toml
# Bootloader
protov-nvm = { path = "../protov-nvm", features = ["bootloader"] }

# Application
protov-nvm = { path = "../protov-nvm", features = ["application"] }
```

```rust
// firmware build.rs
fn main() {
    let nvm = std::env::var("DEP_PROTOV_NVM_OUT_DIR").unwrap();
    println!("cargo:rustc-link-search={nvm}");
    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
}
```

```bash
just test                  # from protov-nvm/
just nvm::test             # from repo root
just nvm::build-bootloader
just nvm::build-application
```

## Factory sector (`FACTORY`, 4 KiB @ `0x101FF000`)

Manufacturing identity is stored at the start of the factory partition:

| Field | Size | Notes |
|-------|------|-------|
| Magic | 4 | `PFAC` |
| Version | 1 | `1` |
| HW rev len / serial len | 2 | ASCII byte counts |
| Date | 4 | `year` (u16 LE), `month`, `day`, pad |
| HW revision | 8 | null-padded ASCII |
| Serial | 16 | null-padded ASCII |
| Signature | 64 | Ed25519 over serial bytes |

256 bytes are programmed (padded with `0xFF`); the rest of the 4 KiB sector is erased.

Rust API: [`factory`](src/factory.rs). Application firmware loads this at boot via
`protov_core::config::init_from_factory_flash()`.

To program a unit at manufacturing time, build or build-and-flash with the
`factory-program` feature and compile-time identity env vars:

| Variable | Description |
| --- | --- |
| `SERIAL_NUMBER` | Device serial (ASCII, max 16 chars) |
| `HARDWARE_REV` | Hardware revision (e.g. `A.1`) |
| `FACTORY_YEAR` / `FACTORY_MONTH` / `FACTORY_DAY` | Manufacturing date |

Signing — provide **one** of:

| Option | Variables |
| --- | --- |
| Precomputed signature | `SIGNATURE` — 128 hex digits (64-byte Ed25519) |
| HW key signing | `HW_PRIVATE_KEY` + `HW_PUBLIC_KEY` — `.pem` paths or inline PEM; `SIGNATURE` is derived by the recipe |

The signed payload is `{serial},{hw_revision},{YYYY-MM-DD}`. The bash helper
`encode_attestation_message` in [`scripts/sign-helpers.sh`](../scripts/sign-helpers.sh)
must stay in sync with [`encode_attestation_message`](src/attestation.rs) in this
crate. When HW keys are supplied, `just factory-run` / `just factory-build` sign
with OpenSSL Ed25519 (`-rawin`, no pre-hash) and verify before invoking Cargo.

```bash
SERIAL_NUMBER=550e8400 HARDWARE_REV=A.1 \
  FACTORY_YEAR=2026 FACTORY_MONTH=6 FACTORY_DAY=27 \
  HW_PRIVATE_KEY=path/to/protov_private_HW0.pem \
  HW_PUBLIC_KEY=path/to/protov_public_HW0.pem \
  just factory-run
```

See repo-root `just factory-run` / `just factory-build`.

