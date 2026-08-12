# protov-hal-mock

`protov-hal-mock` is a host-side ProtoV MINI simulator. It runs the
[`protov-core`](../protov-core/README.md) product logic without RP2040 hardware
and exposes WebSocket endpoints for
[`protov_app`](https://github.com/flakeblade/protov_app), integration tests,
and protocol development.

## Run

```sh
just build-mock
just run-mock
```

The default listeners are:

- `ws://127.0.0.1:8765` — SCPI transport
- `ws://127.0.0.1:8766` — JSON control and state management

Override them with `--bind`, `--scpi-port`, and `--control-port`, or the
`PROTOV_MOCK_SCPI_PORT` and `PROTOV_MOCK_CTRL_PORT` environment variables.

> [!TIP]
> Use the mock for application and SCPI work before connecting real power
> hardware. It follows the same core state machine and is faster to reset,
> inspect, and automate.

## UI state renders

YAML presets under [`states/`](states) describe full device snapshots (SCPI
channels, interface fields, power input, and UI mode). Extend
[`states/default.yaml`](states/default.yaml) for deltas.

Export headless PNG catalog images for documentation:

```sh
just render-ui
```

This writes PNGs to [`docs/res/ui/`](../docs/res/ui/) and refreshes
[`docs/ui-states.md`](../docs/ui-states.md).

## How it works

The server maintains four independent mock-device slots. A SCPI WebSocket
connection acquires an available slot and releases it when disconnected. The
control endpoint can inspect, load, reset, or release slots with JSON messages.
Ready-made snapshots for common states live under [`states/`](states).

- [`src/main.rs`](src/main.rs) — command-line options and server lifecycle
- [`src/server.rs`](src/server.rs) — listener setup and shutdown
- [`src/pool.rs`](src/pool.rs) — mock-device slots and identities
- [`src/device.rs`](src/device.rs) — simulated device state and command handling
- [`src/ws/scpi.rs`](src/ws/scpi.rs) — SCPI WebSocket sessions
- [`src/ws/control.rs`](src/ws/control.rs) — JSON control WebSocket
- [`src/state/`](src/state) — snapshots and control request schema
- [`src/scpi/`](src/scpi) — framing, routing, tasks, and simulated DFU
- [`src/telemetry.rs`](src/telemetry.rs) — generated measurements
- [`src/render.rs`](src/render.rs) — headless UI render pipeline (`render-ui` feature)

SCPI commands follow the
[ProtoV wire protocol](../protov-scpi/PROTOCOL.md). The mock uses the
[`protov-scpi`](../protov-scpi/README.md) parser and host-compatible types.
Control requests support `ping`, `status`, `reset`, `load`, and `release_all`;
their JSON schema is defined in
[`src/state/schema.rs`](src/state/schema.rs).

## Test

```sh
just test-mock
```

The integration tests under [`tests/`](tests) cover connections, command
dispatch, state isolation, and firmware-update behavior. Select a matching
hardware profile with `hw-a0`, `hw-a1`, or `hw-a2`; A.1 is the default.

See [Development](../docs/development.md) for the full workspace workflow and
[Contributing](../CONTRIBUTING.md) before changing shared behavior.
