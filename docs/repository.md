# Repository guide

The hardware is designed with [KiCad 10](https://www.kicad.org/). The firmware
uses the [Embassy](https://embassy.dev/) embedded framework.

| Path | Purpose |
| --- | --- |
| `protov-core/` | Host-testable product logic and state machines |
| `protov-hal/` | RP2040 firmware, drivers, and user interface |
| `protov-bootloader/` | A/B firmware bootloader |
| `protov-nvm/` | Flash layout, factory data, and firmware verification |
| `protov-scpi/` | SCPI protocol, host client, and browser WASM build |
| `protov-hal-mock/` | Host-side WebSocket device simulator |
| `hardware/` | KiCad design files |
| `docs/` | Project documentation and product images |
| `scripts/` | Firmware upload and release-signing helpers |
| `res/` | Project branding |

## Hardware revisions

Hardware revisions A.0, A.1, and A.2 use the `hw-a0`, `hw-a1`, and `hw-a2`
Cargo features. Development commands build A.1 by default.

Each crate keeps detailed implementation documentation next to its source.
Start at the [documentation section](../README.md#documentation).

To determine your hardware revision, and thus the appropriate release to flash, match the details of your hardware to the following table.

| Revision | Stage | Shipment Dates | PCB Logotype | Flash Size | Firmware Size | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| A.0 | Prototyping | 2025 Q2 - 2026 Q1 | `protovolt mini` | 128 MBit (16 MByte) | --- | Faulty CH A/B kelvin sense.
| A.1 | Development | 2026 Q1 - 2026 Q2 | `protovolt mini` | 128 MBit (16 MByte) | --- | Faulty CH A/B kelvin sense (shunt value compensated in software).
| A.2 | Production | TBD |  `protov mini` | 16 MBit (2 MByte) | ~291 kByte (v1.7.3) |

## Software revisions

ProtoV follows [Semantic Versioning](https://semver.org/). All Rust packages share the same version defined in the workspace [Cargo.toml](../Cargo.toml).

- **Major (`x.0.0`)** — breaking changes. ProtoV MINI is stable on v1, and no major
  breaking changes are expected.
- **Minor (`1.x.0`)** — backward-compatible features. Many features are still
  planned; see the [project roadmap](roadmap.md).
- **Patch (`1.0.x`)** — backward-compatible bug fixes. Expect frequent fixes;
  small quality-of-life changes may not receive a standalone release.
