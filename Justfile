# ProtoV firmware workspace helpers (run from repo root)

default:
    @just --list

# Host target (override if needed: just test host_target=x86_64-apple-darwin)
host_target := env_var_or_default("HOST_TARGET", "x86_64-unknown-linux-gnu")

# Embedded target
embedded_target := "thumbv6m-none-eabi"

# Host unit tests (protov-core)
test:
    cargo test -p protov-core --target {{host_target}} --all-features

# Lint host-testable core logic
clippy:
    cargo clippy -p protov-core --target {{host_target}} --all-features -- -D warnings

# Cross-compile firmware for RP2040
build:
    cargo build -p protov-hal --target {{embedded_target}} --release

# Generate .uf2 file from release build
pkg:
    elf2uf2-rs target/thumbv6m-none-eabi/release/protov target/protov.uf2

# Build and flash via probe-rs (see .cargo/config.toml runner)
run:
    cargo run -p protov-hal --target {{embedded_target}} --release
