# Flashing

## Option 1: DFU over SCPI

For normal firmware updates, use [ProtoV App](https://protov.app). It downloads
the signed release for your hardware revision and sends it to the device over
USB. The device verifies the signature before applying the update.

See [Release builds](releases.md) for signing details and instructions for
using your own keys and ProtoV App build.

### Manual SCPI upload

You can download the matching `.bin` and `.sign.bin` files from
[GitHub Releases](https://github.com/flakeblade/protov/releases), then upload
them with [`scripts/fwup_upload.py`](../scripts/fwup_upload.py):

```sh
pip install pyserial
python scripts/fwup_upload.py \
  --port /dev/ttyACM0 \
  --firmware dist/A.1/protov-1.7.3-A.1.bin \
  --signature dist/A.1/protov-1.7.3-A.1.sign.bin
```

Replace the port, version, paths, and hardware revision as needed.

> [!CAUTION]
> Prefer ProtoV App. The manual uploader is a low-level development tool and
> provides fewer safeguards against selecting the wrong artifacts.

## Option 2: USB BOOTSEL

USB BOOTSEL bypasses the installed application and copies a UF2 image through
the RP2040 ROM bootloader.

> [!IMPORTANT]
> Use a UF2 built for the correct hardware revision. Incorrect pin mappings,
> flash layouts, or power-control behavior can damage the device or connected
> hardware. Determine your revision from the
> [hardware revision table](repository.md#hardware-revisions) before building
> or downloading an image.

Generate the UF2 for the required revision:

```sh
just build-a0    # or build-a1 / build-a2
just pkg
```

See [Release builds](releases.md) for complete release bundles. A generated
development image is written to `target/protov.uf2`.

<!-- TODO: Add image showing the UBOOT jumper being shorted. -->

To enter BOOTSEL mode:

1. Unplug USB from ProtoV MINI.
2. Short the two pads of the `UBOOT` jumper.
3. Keep the jumper shorted while plugging in USB.
4. Wait a few seconds for the RP2040 mass-storage
   device to enumerate. Remove the short.
5. Copy the correct `.uf2` file to the mounted drive. The device disconnects
   and reboots automatically after the copy completes.

> [!TIP]
> A multimeter in current-measurement mode can be used as a temporary dead
> short: touch one probe to each `UBOOT` pad before connecting USB. This can be
> easier than holding a small wire or tweezers. Remove the probes after
> connecting USB, and immediately return the multimeter lead and selector to
> voltage mode. Leaving it configured for current measurement can short the
> next circuit you test and blow the meter fuse.

To package the bootloader itself as UF2, run `just bootloader::pkg`.

## Option 3: SWD debug probe

SWD is intended for development, debugging, and device recovery. The wiring
principles are shown on page 18 of Raspberry Pi's
[Getting started with Raspberry Pi Pico](https://pip-assets.raspberrypi.com/categories/610-raspberry-pi-pico/documents/RP-008276-DS-2-getting-started-with-pico.pdf#page=18).

Compatible probes supported by [`probe-rs`](https://probe.rs/docs/getting-started/probe-setup/)
include:

- [Raspberry Pi Debug Probe](https://www.raspberrypi.com/documentation/microcontrollers/debug-probe.html)
  (CMSIS-DAP)
- A Raspberry Pi Pico running the Raspberry Pi `debugprobe` firmware
  (CMSIS-DAP)
- SEGGER J-Link probes
- ST-Link V2 and V3 probes
- Other CMSIS-DAP-compatible probes

Use SWD at 3.3 V logic levels. Connect the probe to the three pads beside the
crystal:

- `D` — SWD data
- `G` — ground
- `<` — SWD clock

Soldering temporary wires to these pads is strongly recommended. Holding loose
probe wires against the small pads is difficult and can cause an unreliable
connection during flashing.

<!-- TODO: Add image showing soldered SWD wires and a connected debug probe. -->

Build and flash the firmware for the correct revision:

```sh
just run
```

The default `just run` recipe targets A.1. Use the appropriate Cargo hardware
feature when developing for another revision.

Bootloader commands are separate:

```sh
just bootloader::build
just bootloader::flash
```

## Debugging the bootloader and main firmware

> [!IMPORTANT]
> Do not leave a debug-profile bootloader installed when using `probe-rs`
> logging or debugging on the main firmware. The debugger can remain associated
> with the bootloader's vector-table and debug context instead of attaching
> correctly after control passes to the main firmware.

Debug the [bootloader](../protov-bootloader/README.md) in debug mode when
needed. Before debugging the main firmware, rebuild and flash the bootloader in
release mode:

```sh
just bootloader::build
just bootloader::flash
just run
```

The repository's bootloader recipes include `--release`. Flashing that release
bootloader restores the expected layout and allows `probe-rs` to attach to and
log the main firmware normally.
