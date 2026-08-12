# protov-ui

Hardware-agnostic embedded-graphics UI for ProtoV MINI. This crate renders the
320×240 landscape panel from [`DisplayTask`](../protov-core/src/model/types.rs)
values in [`protov-core`](../protov-core/README.md); it does not talk to SPI,
backlight PWM, or WS2812 LEDs directly.

## Public API

| Module / type | Role |
|---------------|------|
| `DisplayGeometry` | Logical panel size (default 320×240) |
| `ChannelAppearance` | Channel colors and LED tints (impl for `ScpiState`) |
| `UiPlatform` | Backlight and RGB LED side effects |
| `NullPlatform` | No-op platform for simulation |
| `UiRenderer` | Sync draw state over any `DrawTarget<Color = Rgb565>` |
| `dispatch_display_task` | Sync task dispatcher (no boot timers) |

## Modules

- `boot`, `controls`, `navbar`, `settings` — screen drawing
- `theme`, `labels`, `fonts`, `layout` — shared styling and geometry
- `dispatch` — maps `DisplayTask` to renderer calls

## Build

`no_std` crate; builds for host check and embedded targets:

```sh
cargo build -p protov-ui --target x86_64-unknown-linux-gnu
cargo build -p protov-ui --target thumbv6m-none-eabi
```

Optional `demo` feature shows a demo-build banner on the boot screen.

## Extension points

- **Alternate panel size** — pass a custom `DisplayGeometry` to `UiRenderer::new`
- **Different host link indicator** — implement `UiPlatform::serial_connected`
- **Custom channel colors** — implement `ChannelAppearance` instead of using
  `ScpiState` directly

## Consumers

- [`protov-hal`](../protov-hal/README.md) — `HalPlatform` drives backlight and
  WS2812 LEDs; async wrapper adds boot hold timers
- [`protov-hal-mock`](../protov-hal-mock/README.md) — optional `display`
  feature renders to an SDL window via `NullPlatform`
