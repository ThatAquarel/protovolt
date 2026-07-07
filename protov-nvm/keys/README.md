# Protov Firmware Signing Keys

This directory contains the **public** keys used to verify signed firmware,
hardware serial attestations, and trust manifests for Protov devices. All keys
are Ed25519. Raw key files are exactly 32 bytes (one key) or a multiple of 32
bytes (concatenated keys), with no PEM/ASN.1 wrapper — just the raw key bytes.

**Only public keys live here.** The corresponding private keys are never
committed and are held offline / in CI secrets — see "Trust model" below.

## Files

| File | Size | Contents |
|---|---|---|
| `protov_public.key` | 32 bytes | **Master (root) public key.** Verifies trust manifests below. Kept offline; almost never rotates. |
| `protov_public_CIx.key` | 96 bytes | Three concatenated 32-byte **CI signing public keys** (CI0, CI1, CI2, in that order). These verify individual signed firmware releases. |
| `protov_public_CIx.key.sig` | 64 bytes | Root signature over `protov_public_CIx.key`. |
| `protov_public_HWx.key` | 96 bytes | Three concatenated 32-byte **hardware attestation public keys** (HW0, HW1, HW2, in that order). These verify signatures over device serial numbers and related manufacturing records. |
| `protov_public_HWx.key.sig` | 64 bytes | Root signature over `protov_public_HWx.key`. |

## Trust model

### Firmware releases (CIx)

```
MASTER KEY (offline, cold storage)
   │ signs
   V
CI TRUST MANIFEST  (protov_public_CIx.key + .sig)
   │ authorizes
   V
CI SIGNING KEY(S)  (live in GitHub Actions secrets, used by CI on release)
   │ signs
   V
FIRMWARE IMAGE  (what actually gets flashed to a device)
```

- The **master key** never touches CI or the internet. It only ever signs
  a new trust manifest when the set of valid CI keys changes (rotation or
  revocation).
- The **CI keys** are the day-to-day firmware signers. Each release build is
  signed by whichever CI key is currently active. If a CI key is ever suspected
  compromised, it's dropped from a newly-signed trust manifest and firmware
  built with it stops being trusted — without needing physical device access.
- The device bootloader hardcodes the master public key, so it can verify the
  CI manifest, and in turn trusts whichever CI keys that manifest currently
  lists.

### Hardware serial attestation (HWx)

```
MASTER KEY (offline, cold storage)
   │ signs
   V
HW TRUST MANIFEST  (protov_public_HWx.key + .sig)
   │ authorizes
   V
HW SIGNING KEY(S)  (held in manufacturing / provisioning tooling)
   │ signs
   V
SERIAL ATTESTATION  (device serial number and related manufacturing data)
```

- The **HW manifest** uses the same root key and manifest format as CIx (96
  bytes = three Ed25519 public keys), but authorizes a separate key set used
  only for manufacturing workflows.
- **HW keys** sign serial numbers (and related hardware identity records) at
  production time. Any verifier with the embedded root key can confirm that a
  serial was signed by one of the currently trusted HW keys.
- Rotating or revoking an HW key follows the same process as CI: the master
  key signs a new `protov_public_HWx.key` manifest; verifiers reject
  attestations from keys no longer listed.

## Verifying a signature with OpenSSL

OpenSSL's `pkeyutl` needs an Ed25519 public key wrapped in a minimal
DER/PEM structure (SubjectPublicKeyInfo), not the bare 32 raw bytes. The
prefix below is fixed for all Ed25519 keys — just prepend it to the raw key.

**1. Convert a raw 32-byte public key to PEM (one-time, per key):**

```bash
# Master key
( printf '\x30\x2a\x30\x05\x06\x03\x2b\x65\x70\x03\x21\x00'; cat protov_public.key ) \
  > /tmp/protov_public.der
openssl pkey -pubin -inform DER -in /tmp/protov_public.der \
  -outform PEM -out /tmp/protov_public.pem
```

For a CI or HW key, extract the 32 bytes you want first (e.g. slot 0 = first
32 bytes of the concatenated manifest file):

```bash
# CI0 / HW0 = bytes 0-31, slot 1 = bytes 32-63, slot 2 = bytes 64-95
dd if=protov_public_CIx.key of=/tmp/ci0.key bs=1 skip=0  count=32
dd if=protov_public_HWx.key of=/tmp/hw0.key bs=1 skip=0  count=32

( printf '\x30\x2a\x30\x05\x06\x03\x2b\x65\x70\x03\x21\x00'; cat /tmp/ci0.key ) \
  > /tmp/ci0.der
openssl pkey -pubin -inform DER -in /tmp/ci0.der -outform PEM -out /tmp/ci0.pem
```

**2. Verify a signature (raw Ed25519, no pre-hashing):**

```bash
# Verify a trust manifest, signed by the master key
openssl pkeyutl -verify -pubin -inkey /tmp/protov_public.pem \
  -rawin -in protov_public_CIx.key -sigfile protov_public_CIx.key.sig

openssl pkeyutl -verify -pubin -inkey /tmp/protov_public.pem \
  -rawin -in protov_public_HWx.key -sigfile protov_public_HWx.key.sig

# Verify a firmware image, signed by a CI key
openssl pkeyutl -verify -pubin -inkey /tmp/ci0.pem \
  -rawin -in firmware.bin -sigfile firmware.sig

# Verify a serial attestation, signed by an HW key
openssl pkeyutl -verify -pubin -inkey /tmp/hw0.pem \
  -rawin -in serial_record.bin -sigfile serial_record.sig
```

A successful check prints `Signature Verified Successfully`. Anything else
means the file, key, or signature don't match — do not trust the artifact.

## Rotating or revoking a CI or HW key

If a CI or HW key needs to be replaced (routine rotation, or a suspected leak):

1. A new keypair is generated and its public key added to the manifest file.
2. The master key (offline) signs the updated manifest (`*.key.sig`).
3. For CI keys, the new manifest ships as part of the next signed firmware
   release; devices refresh their trusted CI key set after verifying the
   manifest against the hardcoded root key.
4. For HW keys, manufacturing tooling picks up the new manifest so newly
   issued serial attestations use the current trusted key set.

The master key itself is not expected to rotate under normal operation.
