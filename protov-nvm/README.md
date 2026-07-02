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

Update all three files together. Run `just test-nvm` after changes.

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
just nvm::test
just nvm::build-bootloader
just nvm::build-application
```
