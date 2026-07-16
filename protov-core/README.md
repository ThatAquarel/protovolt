# protov-core

`protov-core` contains the hardware-independent product logic shared by the
RP2040 firmware, host tests, and simulator. It is `no_std` outside tests and
does not access peripherals directly.

Events enter [`AppCore`](src/app/mod.rs), which updates product state and emits
typed display or hardware tasks. [`protov-hal`](../protov-hal/README.md)
executes those tasks on the device;
[`protov-hal-mock`](../protov-hal-mock/README.md) executes equivalent 
behavior on a host.

## Important files

- [`src/app/mod.rs`](src/app/mod.rs) — main state machine, event handling, SCPI
  behavior, and task generation
- [`src/model/`](src/model) — shared events, states, channel types, and tasks
- [`src/config/`](src/config) — product identity, defaults, limits, telemetry,
  and provisioned factory data
- [`src/config/hardware/`](src/config/hardware) — revision-specific analog and
  calibration constants
- [`src/protection.rs`](src/protection.rs) — voltage, current, and temperature
  protection decisions
- [`src/pd/`](src/pd) — USB PD source-capability parsing and selection
- [`src/scpi/`](src/scpi) — device-side SCPI state, responses, telemetry, and
  integration with [`protov-scpi`](../protov-scpi/README.md)
- [`src/dfu/`](src/dfu) — transport-independent firmware-update session state

The complete wire protocol lives in the
[SCPI protocol reference](../protov-scpi/PROTOCOL.md).

## Hardware profiles

Enable exactly one of `hw-a0`, `hw-a1`, or `hw-a2`. The profiles correspond to
the revisions in the
[hardware revision table](../docs/repository.md#hardware-revisions).

```sh
just test-a0
just test-a1
just test-a2
```

Use `test-harness` for host integration tests. The `simulator` feature includes
that harness and is used by `protov-hal-mock`.

Behavioral tests are colocated with their modules. The public-API harness lives
in [`tests/integration.rs`](tests/integration.rs) with support code in
[`tests/support/`](tests/support). Generate a coverage report with:

```sh
just coverage
```

> [!TIP]
> Put product behavior in `protov-core` whenever it can be expressed without
> RP2040 peripherals. It becomes easier to test across every hardware revision
> and reuse in the simulator.

> [!IMPORTANT]
> Changes to limits or protection behavior require tests and electrical
> justification. See the
> [contribution requirements](../CONTRIBUTING.md#feature-changes-1x0).

See [Development](../docs/development.md) for workspace commands and
[`protov-hal`](../protov-hal/README.md) for the device-side implementation.
