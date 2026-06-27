# ProtoV firmware workspace helpers (run from repo root)

default:
    @just --list

# Host unit tests (protov-core on x86)
test:
    cargo test -p protov-core --all-features

# Lint host-testable core logic
clippy:
    cargo clippy -p protov-core --all-features -- -D warnings

# Cross-compile firmware for RP2040
build:
    cargo build -p protov-hal --target thumbv6m-none-eabi --release

# Build and flash via probe-rs (see .cargo/config.toml runner)
run:
    cargo run -p protov-hal --target thumbv6m-none-eabi --release
