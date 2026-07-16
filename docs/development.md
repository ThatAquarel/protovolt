# Development

## Prerequisites

- A current Rust toolchain
- [`just`](https://github.com/casey/just)
- `probe-rs` for SWD flashing and debugging
- `elf2uf2-rs` for USB flash images
- The `thumbv6m-none-eabi` Rust target
- GNU Arm Embedded `objcopy` for complete release bundles

Install the Rust-based tools and target:

```sh
rustup target add thumbv6m-none-eabi
cargo install just probe-rs-tools elf2uf2-rs
```

## Build firmware

```sh
just build       # A.1, the default
just build-a0
just build-a1
just build-a2
```

## Check changes

```sh
just checks      # all tests, lint, and formatting
just test        # core A.1 and SCPI tests
just clippy
just fmt
```

Individual test recipes are available for every hardware revision and
workspace component. Run `just --list` for the complete list.

## Simulator and browser transport

```sh
just build-mock
just run-mock
just scpi-wasm-build
```

The simulator exposes a WebSocket SCPI bridge for development and end-to-end
testing without hardware.

## Query a connected device

```sh
just scpi-id /dev/ttyACM0
```

The command queries the device identity over USB serial. Replace the port with
the path used by your system.
