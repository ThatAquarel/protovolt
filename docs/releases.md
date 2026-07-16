# Release builds

Create a release bundle with a version number that does not include a `v`
prefix:

```sh
just release-bundle 1.7.3
```

The command builds A.0, A.1, and A.2 firmware and writes the artifacts,
manifest, checksums, and archive under `dist/`.

Release packaging requires `arm-none-eabi-objcopy` and `elf2uf2-rs`.

## Signing

> [!IMPORTANT]
> Signing is not required for development builds flashed through USB BOOTSEL or
> SWD. A matching firmware signature is required only when an image is
> delivered through [DFU over SCPI](../protov-scpi/PROTOCOL.md#firmware-update-fwup).

Set `PRIVATE_KEY` and `PUBLIC_KEY` to include signed firmware in a local
release:

```sh
PRIVATE_KEY=... PUBLIC_KEY=... just release-bundle 1.7.3
```

To sign the current default build only:

```sh
just build
PRIVATE_KEY=... PUBLIC_KEY=... just sign-firmware
```

The [`protov-nvm` documentation](../protov-nvm/README.md) describes the flash
layout, and the [signing-key documentation](../protov-nvm/keys/README.md)
describes the Ed25519 key files, root-signed CI key manifest, verification
format, rotation, and revocation.

> [!CAUTION]
> Never commit private signing keys. Pull requests that add or modify trusted
> public keys will be rejected. Changes to the firmware signing or verification
> workflow will be thoroughly reviewed to ensure backwards compatibility.

### Verification path

The device handles SCPI DFU in `protov-hal`. During
[`FirmwareCtx::dfu_verify_apply`](../protov-hal/src/hal/firmware.rs#L143-L171),
it first verifies the embedded CI key manifest and then calls
[`verify_and_mark_updated(key, &signature, len)`](../protov-hal/src/hal/firmware.rs#L153-L167)
for each authorized release key. The update is marked ready only when one of
those keys verifies the supplied image signature. The
[bootloader](../protov-bootloader/README.md#behaviour) applies that verified
image on reboot.

[ProtoV App](https://protov.app) and the
[`protov_app` repository](https://github.com/flakeblade/protov_app) use signed
firmware artifacts for web-based DFU. These artifacts are pulled directly from the CI release of this repository. The app submits the image and signature;
the device checks them against its embedded trusted keys before accepting the
update.

### Custom keys and unsigned firmware

The recommended way to distribute custom firmware over SCPI DFU is to maintain
your own trust chain:

1. Follow the [documented raw-key and manifest format](../protov-nvm/keys/README.md#files).
2. Replace the root key and CI manifest in `protov-nvm`, then sign the manifest
   with your own root private key. Adding a public key alone is insufficient:
   the CI manifest must match the root embedded in your firmware.
3. Build and install that initial custom firmware through
   [USB BOOTSEL](flashing.md#usb) or [SWD](flashing.md#swd).
4. Sign future releases with an authorized private release key and configure
   your own build of [`protov_app`](https://github.com/flakeblade/protov_app)
   to provide the matching image and signature. The existing
   `verify_and_mark_updated` path will then enforce your keys.

You may instead modify your firmware to bypass signature verification. The
stock signed-DFU path cannot install that first modified image, so it must be
loaded through [USB BOOTSEL](flashing.md#usb) or [SWD](flashing.md#swd).
Afterward, a custom DFU implementation can accept unsigned images.

> [!CAUTION]
> Disabling verification removes firmware-authenticity protection. Also,
> `verify_and_mark_updated` both verifies **and** marks the image for boot; do
> not simply delete the call without providing the required update-state
> handling. Any verifier-free firmware or update flow is used entirely at your
> own risk.
