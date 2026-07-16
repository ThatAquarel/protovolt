# Hardware

ProtoV MINI is an open-hardware, credit card-sized dual-channel power supply.
For electrical and mechanical specifications, see the
[product overview](product.md#specifications).

## Design resources

The editable hardware design is available in the repository's
[`hardware/`](../hardware/) directory and is created with
[KiCad](https://www.kicad.org/).

| Resource | Location |
| --- | --- |
| ProtoV MINI KiCad project | [`hardware/mini/`](../hardware/mini/) |
| Main schematic | [`mini.kicad_sch`](../hardware/mini/mini.kicad_sch) |
| PCB project settings | [`mini.kicad_pro`](../hardware/mini/mini.kicad_pro) |
| Custom footprints and 3D models | [`protovolt_mini.pretty/`](../hardware/mini/protovolt_mini.pretty/) |
| Hardware revision guide | [Repository guide](repository.md#hardware-revisions) |

> [!TIP]
> Open `mini.kicad_pro` in KiCad to load the complete project and its
> hierarchical schematics.

## Hardware overview

![ProtoV MINI hardware overview](res/info.png)

The overview render identifies ProtoV MINI's connectors, display, controls,
output channels, and other user-accessible hardware.

## Dimensions

![ProtoV MINI dimensions](res/dimensions.png)

ProtoV MINI measures **85.5 × 54 × 18 mm**. Additional specifications are
listed in the [product overview](product.md#specifications).

> [!NOTE]
> The renders are visual references. Use the KiCad design files when exact
> mechanical placement or manufacturing data is required.

For first-time setup, continue to the [quick-start guide](quick_start.md). For
tested power supplies, breadboards, browsers, and operating systems, see the
[compatibility guide](compatibility.md).
