# protov-hal

`protov-hal` is the RP2040 application firmware for ProtoV MINI. It connects the
hardware-independent state machine in
[`protov-core`](../protov-core/README.md) to the power converters, measurement
devices, display, buttons, USB, LEDs, watchdog, and flash.

Display rendering lives in [`protov-ui`](../protov-ui/README.md); this crate
implements `HalPlatform` (backlight + RGB LEDs) and an async display wrapper with
boot hold timers.

The crate builds the `protov` binary for `thumbv6m-none-eabi` with
[Embassy](https://embassy.dev/).

## Runtime overview

[`src/main.rs`](src/main.rs) initializes the RP2040 and firmware-update state,
starts USB before slower peripheral setup, configures both power channels, and
runs the main event loop. Hardware, interface, and SCPI events are passed to
`protov-core`; the resulting tasks are dispatched back to the hardware or UI.

Button polling runs on the second RP2040 core. Measurement, temperature, USB,
converter interrupt, and display work is coordinated with Embassy tasks and
channels.

> [!TIP]
> Start with [`src/main.rs`](src/main.rs), then follow an event into
> [`src/app.rs`](src/app.rs), [`src/task.rs`](src/task.rs), and
> [`src/hal/event.rs`](src/hal/event.rs). This shows the boundary between
> product policy and peripheral access.

## Important files

- [`src/main.rs`](src/main.rs) — startup, multicore setup, task spawning, and
  main event loop
- [`src/task.rs`](src/task.rs) — executes display, hardware, and DFU tasks
- [`src/ui_platform.rs`](src/ui_platform.rs) — `HalPlatform` (backlight, WS2812)
- [`src/hal/converter.rs`](src/hal/converter.rs) — TPS55289 output control
- [`src/hal/measure.rs`](src/hal/measure.rs) — INA226 voltage and current
  measurement
- [`src/hal/power/`](src/hal/power) — STUSB4500 setup and USB PD negotiation
- [`src/hal/temperature.rs`](src/hal/temperature.rs) — channel and MCU
  temperature acquisition
- [`src/hal/firmware.rs`](src/hal/firmware.rs) — signed SCPI DFU writes and
  verification
- [`src/scpi/usb.rs`](src/scpi/usb.rs) — USB CDC transport and update payload
  assembly
- [`src/hal/factory_program.rs`](src/hal/factory_program.rs) — manufacturing
  identity programming
- [`src/hal/watchdog.rs`](src/hal/watchdog.rs) — boot and runtime watchdog
- [`build.rs`](build.rs) — application linker setup and factory-record
  generation

Flash partitions and trusted keys are defined by
[`protov-nvm`](../protov-nvm/README.md), with the trust model documented in
[`protov-nvm/keys`](../protov-nvm/keys/README.md). SCPI syntax is defined by
[`protov-scpi`](../protov-scpi/README.md), and boot behavior is documented in
[`protov-bootloader`](../protov-bootloader/README.md).

## Build and flash

Hardware revision A.1 is the default:

```sh
just build
just run
```

Select another revision explicitly:

```sh
just build-a0
just build-a1
just build-a2
```

Enable exactly one hardware feature. See
[Development](../docs/development.md) for prerequisites,
[Flashing](../docs/flashing.md) for upload and debug options, and
[Release builds](../docs/releases.md) for signed artifacts.

The optional `demo` feature changes the startup presentation. The
`factory-program` feature enables one-shot manufacturing identity programming;
use `just factory-run` (flash) or `just factory-build` (build only).

Factory builds require identity env vars (`SERIAL_NUMBER`, `HARDWARE_REV`,
`FACTORY_YEAR`, `FACTORY_MONTH`, `FACTORY_DAY`) and either a precomputed
`SIGNATURE` or `HW_PRIVATE_KEY` + `HW_PUBLIC_KEY` PEM pair. The just recipes
source [`scripts/sign-helpers.sh`](../scripts/sign-helpers.sh), which signs the
canonical attestation message from `encode_attestation_message`
(`{serial},{hw_revision},{YYYY-MM-DD}` — must match
`protov_nvm::encode_attestation_message`) before Cargo runs. Do not enable
`factory-program` in normal firmware builds.

> [!WARNING]
> This crate directly controls power hardware. Confirm pin mappings, register
> values, limits, and calibration against the applicable datasheets and
> [hardware revision](../docs/repository.md#hardware-revisions). Protection
> changes must follow the
> [contribution requirements](../CONTRIBUTING.md#feature-changes-1x0).
