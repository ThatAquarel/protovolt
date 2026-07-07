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

To program a unit at manufacturing time, build and flash with the `factory-program` feature
and compile-time env vars (`SERIAL_NUMBER`, `HARDWARE_REV`, `FACTORY_YEAR`, `FACTORY_MONTH`,
`FACTORY_DAY`, `SIGNATURE`). See repo-root `just factory-program`.

