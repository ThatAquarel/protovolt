# ProtoV firmware workspace helpers (run from repo root)

mod nvm 'protov-nvm/Justfile'
mod bootloader 'protov-bootloader/Justfile'

default:
    @just --list

# Host target (override if needed: just test host_target=x86_64-apple-darwin)
host_target := env_var_or_default("HOST_TARGET", "x86_64-unknown-linux-gnu")

# Embedded target
embedded_target := "thumbv6m-none-eabi"

elf := "target/" + embedded_target + "/release/protov"
release_dir := "dist"

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

# Sign release firmware (requires PRIVATE_KEY; optional PUBLIC_KEY for verify)
sign-firmware *args:
    just -f protov-hal/Justfile sign-all {{args}}

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

# Generate .uf2 from the current release ELF (default A.1 build)
pkg:
    elf2uf2-rs {{elf}} target/protov.uf2

# Build and flash via probe-rs (see .cargo/config.toml runner)
run:
    cargo run -p protov-hal --target {{embedded_target}} --release

# Host-only ProtoV MINI WebSocket simulator
build-mock:
    cargo build -p protov-hal-mock --target {{host_target}} --release --features hw-a1

run-mock:
    cargo run -p protov-hal-mock --target {{host_target}} --release --features hw-a1

test-mock:
    cargo test -p protov-hal-mock --target {{host_target}} --features hw-a1

test-nvm:
    @just nvm::test

fmt:
    cargo fmt --all

checks: test-a0 test-a1 test-a2 test-mock test-nvm clippy

# Build + package all hardware profiles (version e.g. 1.0.0, without v prefix)
release-bundle version:
    just _release-profile hw-a0 A.0 {{version}}
    just _release-profile hw-a1 A.1 {{version}}
    just _release-profile hw-a2 A.2 {{version}}
    just _release-manifest {{version}}
    just _release-archive {{version}}

_release-profile feature revision version:
    #!/usr/bin/env bash
    set -euo pipefail
    feature="{{feature}}"
    revision="{{revision}}"
    version="{{version}}"
    target="{{embedded_target}}"
    elf="target/${target}/release/protov"
    out="dist/${revision}"
    base="protov-${version}-${revision}"
    mkdir -p "${out}"
    export RUSTFLAGS="-C link-arg=-Map=${out}/${base}.map"
    case "${feature}" in
      hw-a1)
        cargo build -p protov-hal --target "${target}" --release --features hw-a1
        ;;
      *)
        cargo build -p protov-hal --target "${target}" --release --no-default-features --features "${feature}"
        ;;
    esac
    cp "${elf}" "${out}/${base}.elf"
    cp "${elf}.d" "${out}/${base}.d"
    arm-none-eabi-objcopy -O binary "${elf}" "${out}/${base}.bin"
    elf2uf2-rs "${elf}" "${out}/${base}.uf2"

_release-manifest version:
    #!/usr/bin/env bash
    set -euo pipefail
    version="{{version}}"
    git_sha="$(git rev-parse HEAD)"
    git_tag="$(git describe --tags --exact-match 2>/dev/null || true)"
    {
      echo "ProtoV MINI firmware release ${version}"
      echo "git_commit=${git_sha}"
      echo "git_tag=${git_tag}"
      echo "target={{embedded_target}}"
      echo "profiles=A.0,A.1,A.2"
      echo "artifacts=.elf,.uf2,.bin,.map,.d"
    } > dist/MANIFEST.txt
    (cd dist && find A.0 A.1 A.2 -type f | sort | xargs sha256sum) > dist/SHA256SUMS

_release-archive version:
    tar czf dist/protov-firmware-{{version}}.tar.gz -C dist A.0 A.1 A.2 MANIFEST.txt SHA256SUMS
