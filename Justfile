# ProtoV firmware workspace helpers (run from repo root)

mod nvm 'protov-nvm/Justfile'
mod bootloader 'protov-bootloader/Justfile'

default:
    @just --list

# Host target (override if needed: just test host_target=x86_64-apple-darwin)
host_target := env_var_or_default("HOST_TARGET", "x86_64-unknown-linux-gnu")
embedded_target := "thumbv6m-none-eabi"

elf := "target/" + embedded_target + "/release/protov"
release_dir := "dist"

# Host unit tests (protov-core, A.1 profile)
test: test-a1 test-scpi

test-scpi:
    cargo test -p protov-scpi --target {{host_target}} --features std

scpi-wasm-build:
    bash protov-scpi/scripts/build-wasm.sh

test-a0:
    cargo test -p protov-core --target {{host_target}} --features hw-a0,test-harness

test-a1:
    cargo test -p protov-core --target {{host_target}} --features hw-a1,test-harness

test-a2:
    cargo test -p protov-core --target {{host_target}} --features hw-a2,test-harness

# Lint host-testable core logic
clippy:
    cargo clippy -p protov-core --target {{host_target}} --features hw-a1 -- -D warnings

# Sign release firmware (requires PRIVATE_KEY and PUBLIC_KEY in CI; optional locally).
sign-firmware *args:
    just -f protov-hal/Justfile sign-all {{args}}

# Sign protov-{version}-{revision}.bin for each profile (requires PRIVATE_KEY + PUBLIC_KEY).
release-sign version:
    just _release-sign A.0 {{version}}
    just _release-sign A.1 {{version}}
    just _release-sign A.2 {{version}}

_release-sign revision version:
    #!/usr/bin/env bash
    set -euo pipefail
    in_ci=0
    if [[ -n "${CI:-}" || -n "${GITHUB_ACTIONS:-}" ]]; then
      in_ci=1
    fi
    if [[ -z "${PRIVATE_KEY:-}" ]]; then
      if (( in_ci )); then
        echo "release-sign: PRIVATE_KEY is required in CI" >&2
        exit 1
      fi
      echo "release-sign: PRIVATE_KEY not set; skipping {{revision}}" >&2
      exit 0
    fi
    if [[ -z "${PUBLIC_KEY:-}" ]]; then
      echo "release-sign: PUBLIC_KEY is required when signing" >&2
      exit 1
    fi
    bin="$(pwd)/dist/{{revision}}/protov-{{version}}-{{revision}}.bin"
    if [[ ! -f "$bin" ]]; then
      echo "release-sign: missing firmware binary: $bin" >&2
      exit 1
    fi
    just -f protov-hal/Justfile sign-release-bin "$bin"

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

# Program the factory identity sector from compile-time env vars, then flash.
# Required env: SERIAL_NUMBER HARDWARE_REV FACTORY_YEAR FACTORY_MONTH FACTORY_DAY SIGNATURE
factory-program *args:
    cargo run -p protov-hal --target {{embedded_target}} --release --features factory-program {{args}}

# Query *IDN? and SYST:IDAT? on a USB CDC serial port (e.g. /dev/ttyACM0).
scpi-id port:
    #!/usr/bin/env bash
    set -euo pipefail
    port="{{port}}"
    if [[ ! -e "$port" ]]; then
      echo "serial port not found: $port" >&2
      exit 1
    fi
    stty -F "$port" 115200 cs8 -cstopb -parenb -ixon -crtscts raw -echo min 0 time 5
    exec 3<>"$port"
    scpi_query() {
      local cmd="$1"
      printf '%s\n' "$cmd" >&3
      if ! IFS= read -r -u 3 -t 3 response; then
        echo "timeout waiting for response to ${cmd}" >&2
        return 1
      fi
      printf '%s\n' "$response"
    }
    echo "*IDN?"
    scpi_query "*IDN?"
    echo
    echo "SYST:IDAT?"
    scpi_query "SYST:IDAT?"

# Host-only ProtoV MINI WebSocket simulator
build-mock:
    cargo build -p protov-hal-mock --target {{host_target}} --release --features hw-a1

run-mock:
    cargo run -p protov-hal-mock --target {{host_target}} --release --features hw-a1

test-mock:
    cargo test -p protov-hal-mock --target {{host_target}} --features hw-a1

test-nvm:
    @just nvm::test

coverage_dir := "dist/coverage"
coverage_html_dir := coverage_dir + "/html"
coverage_lcov_path := coverage_dir + "/lcov.info"

# LLVM source coverage (cargo-llvm-cov) for host-testable workspace crates.
# Embedded-only protov-hal and protov-bootloader have no host tests and are excluded.
coverage: coverage-html

coverage-html dir=coverage_html_dir:
    #!/usr/bin/env bash
    set -euo pipefail
    mkdir -p "{{dir}}"
    cargo llvm-cov --workspace \
        --exclude protov-hal --exclude protov-bootloader \
        --target {{host_target}} \
        --features hw-a1,test-harness \
        --html --output-dir "{{dir}}"
    echo "Coverage report: {{dir}}/index.html"

coverage-lcov path=coverage_lcov_path:
    #!/usr/bin/env bash
    set -euo pipefail
    mkdir -p "$(dirname "{{path}}")"
    cargo llvm-cov --workspace \
        --exclude protov-hal --exclude protov-bootloader \
        --target {{host_target}} \
        --features hw-a1,test-harness \
        --lcov --output-path "{{path}}"
    echo "Coverage lcov: {{path}}"

coverage-summary:
    cargo llvm-cov --workspace \
        --exclude protov-hal --exclude protov-bootloader \
        --target {{host_target}} \
        --features hw-a1,test-harness \
        --summary-only --text

fmt:
    cargo fmt --all

checks: test-a0 test-a1 test-a2 test-mock test-nvm test-scpi clippy fmt

# Build + package all hardware profiles (version e.g. 1.0.0, without v prefix)
release-bundle version:
    just _release-profile hw-a0 A.0 {{version}}
    just _release-profile hw-a1 A.1 {{version}}
    just _release-profile hw-a2 A.2 {{version}}
    just release-sign {{version}}
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
      echo "artifacts=.elf,.uf2,.bin,.hash.bin,.sign.bin,.map,.d"
    } > dist/MANIFEST.txt
    (cd dist && find A.0 A.1 A.2 -type f | sort | xargs sha256sum) > dist/SHA256SUMS

_release-archive version:
    tar czf dist/protov-firmware-{{version}}.tar.gz -C dist A.0 A.1 A.2 MANIFEST.txt SHA256SUMS
