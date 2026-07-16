# Compatibility

## USB-C power sources

ProtoV MINI uses the **STUSB4500** USB-C Power Delivery sink controller. It
negotiates power over the USB-C Configuration Channel (CC) pins and is designed
to work with most standards-compliant, fixed-supply USB PD sources.

The USB D+ and D− data lines are routed directly to the RP2040 for device
communication.

> [!NOTE]
> ProtoV MINI therefore does not support Qualcomm Quick Charge or
> other charging standards that negotiate voltage over the USB data lines.
> Power-source negotiation must use the USB-C CC pins.

## Tested power sources

The following list is not exhaustive and will expand as more sources are
tested.

| Source type | Manufacturer | Product and model | Test condition | Negotiated contract | RDO | Position |
| --- | --- | --- | --- | --- | --- | --- |
| USB PD | Anker | Laptop Charger 140W, 4-Port, PD 3.1 (A2697) | Port C3 | 20.0 V, 2.0 A, 40.0 W | `0x400320c8` | 4 |
| USB PD | Anker | Laptop Charger 140W, 4-Port, PD 3.1 (A2697) | Port C1 or C2 | 20.0 V, 5.0 A, 100.0 W | `0x4007d1f4` | 4 |
| USB PD | Dell | XPS 9315 45W Charger (DA45NM210) | USB-C | 20.0 V, 2.25 A, 45.0 W | `0x400384e1` | 4 |
| USB PD | Lenovo | ThinkPad X1 Carbon Gen 11 65W Charger (ADLX65YDC3E) | USB-C | 20.0 V, 3.25 A, 65.0 W | `0x40051545` | 4 |
| Standard USB | Anker | PowerCore 10K (A1229) | Standard/PD | 5.0 V, 3.0 A, 15.0 W | — | — |
| Standard USB | Dell | XPS 13 9360 laptop USB Type-A | Standard/PD | 5.0 V, 0.5 A, 2.5 W | — | — |

## Breadboards

ProtoV MINI's 2×5 output headers use a 2.54 mm pitch and align with the power
rails on common BB400- and BB830-style solderless breadboards. Generic,
inexpensive breadboards are being tested, but their contact material, grip,
resistance, and safe current capacity vary significantly.

The following higher-quality BusBoard Prototype Systems models provide more
consistent contacts and publish electrical ratings:

| Breadboard | Fit | Published rating | Notes |
| --- | --- | --- | --- |
| [BB400](https://www.busboard.com/BB400) | Direct fit | 36 V, 2 A | Compact 400-point board |
| [BB830](https://www.busboard.com/BB830) | Direct fit | 36 V, 2 A | Full-size 830-point board |

Stay within the breadboard manufacturer's rating and derate unknown or
inexpensive boards further. High contact resistance can cause voltage drop and
localized heating; stop using a connection if it becomes warm, loose, or
discolored. Use appropriately rated wiring, connectors, or a soldered PCB for
sustained high-current loads.

> [!CAUTION]
> Do not assume that a solderless breadboard can carry ProtoV MINI's full 5 A
> output.

## ProtoV App

[ProtoV App](https://protov.app) communicates with the device through the Web
Serial API. Browser implementation and adoption are tracked by
[Can I use: Web Serial](https://caniuse.com/web-serial).

| Operating system | Browser | Test status |
| --- | --- | --- |
| Windows | Chrome | Tested |
| Windows | Edge | Tested |
| Windows | Firefox | Tested with Web Serial-capable versions |
| Linux | Chrome | Tested |
| Linux | Firefox | Limited; behavior and device access can be unreliable |
| macOS / iOS | Safari | Not supported |

Browser support can change between releases. Use a current Chromium-based
browser when reliable device access is required.
