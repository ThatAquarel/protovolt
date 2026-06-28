# ProtoV firmware workspace helpers (run from repo root)

default:
    @just --list

# Host target (override if needed: just test host_target=x86_64-apple-darwin)
host_target := env_var_or_default("HOST_TARGET", "x86_64-unknown-linux-gnu")

# Embedded target
embedded_target := "thumbv6m-none-eabi"

# Host unit tests (protov-core, A.1 profile)
test: test-a1

test-a0:
    cargo test -p protov-core --target {{host_target}} --features hw-a0,test-harness

test-a1:
    cargo test -p protov-core --target {{host_target}} --features hw-a1,test-harness

test-a2:
    cargo test -p protov-core --target {{host_target}} --features hw-a2,test-harness

# Lint host-testable core logic
clippy:
    cargo clippy -p protov-core --target {{host_target}} --features hw-a1 -- -D warnings

# Cross-compile firmware for RP2040 (production A.1)
build: build-a1

# Proto / A.0 boards
build-a0:
    cargo build -p protov-hal --target {{embedded_target}} --release --no-default-features --features hw-a0

# Production A.1 boards
build-a1:
    cargo build -p protov-hal --target {{embedded_target}} --release --features hw-a1

# Production A.2 boards
build-a2:
    cargo build -p protov-hal --target {{embedded_target}} --release --no-default-features --features hw-a2

# Generate .uf2 file from release build
pkg:
    elf2uf2-rs target/thumbv6m-none-eabi/release/protov target/protov.uf2

# Build and flash via probe-rs (see .cargo/config.toml runner)
run:
    cargo run -p protov-hal --target {{embedded_target}} --release
