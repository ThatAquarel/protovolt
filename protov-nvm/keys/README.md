# Protov Firmware Signing Keys
 
This directory contains the **public** keys used to verify signed firmware
and trust manifests for Protov devices. All keys are Ed25519. Raw key files
are exactly 32 bytes (one key) or a multiple of 32 bytes (concatenated keys),
with no PEM/ASN.1 wrapper — just the raw key bytes.
 
**Only public keys live here.** The corresponding private keys are never
committed and are held offline / in CI secrets — see "Trust model" below.
 
## Files
 
| File | Size | Contents |
|---|---|---|
| `protov_public.key` | 32 bytes | **Master (root) public key.** Verifies the trust manifest below. Kept offline; almost never rotates. |
| `protov_public_CIx.key` | 96 bytes | Three concatenated 32-byte **CI signing public keys** (CI0, CI1, CI2, in that order). These verify individual signed firmware releases. |
 
## Trust model
 
```
MASTER KEY (offline, cold storage)
   │ signs
   V
TRUST MANIFEST  (lists which CI signing keys are currently valid)
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
- The **CI keys** are the day-to-day signers. Each release build is signed
  by whichever CI key is currently active. If a CI key is ever suspected
  compromised, it's dropped from a newly-signed trust manifest and firmware
  built with it stops being trusted — without needing physical device
  access.
- The device bootloader hardcodes the master public key, so it can verify
  the trust manifest, and in turn trusts whichever CI keys that manifest
  currently lists.
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
 
For a CI key, extract the 32 bytes you want first (e.g. CI0 = first 32
bytes of the concatenated file):
 
```bash
# CI0 = bytes 0-31, CI1 = bytes 32-63, CI2 = bytes 64-95
dd if=protov_public_CIx.key of=/tmp/ci0.key bs=1 skip=0  count=32
dd if=protov_public_CIx.key of=/tmp/ci1.key bs=1 skip=32 count=32
dd if=protov_public_CIx.key of=/tmp/ci2.key bs=1 skip=64 count=32
 
( printf '\x30\x2a\x30\x05\x06\x03\x2b\x65\x70\x03\x21\x00'; cat /tmp/ci0.key ) \
  > /tmp/ci0.der
openssl pkey -pubin -inform DER -in /tmp/ci0.der -outform PEM -out /tmp/ci0.pem
```
 
**2. Verify a signature (raw Ed25519, no pre-hashing):**
 
```bash
# Verify the trust manifest, signed by the master key
openssl pkeyutl -verify -pubin -inkey /tmp/protov_public.pem \
  -rawin -in trust_manifest.bin -sigfile trust_manifest.sig
 
# Verify a firmware image, signed by a CI key
openssl pkeyutl -verify -pubin -inkey /tmp/ci0.pem \
  -rawin -in firmware.bin -sigfile firmware.sig
```
 
A successful check prints `Signature Verified Successfully`. Anything else
means the file, key, or signature don't match — do not flash.
 
## Rotating or revoking a CI key
 
If a CI key needs to be replaced (routine rotation, or a suspected leak):
 
1. A new CI keypair is generated and its public key added here.
2. The master key (offline) signs a new trust manifest that includes the
   new key and/or omits a revoked one.
3. The new manifest ships as part of the next signed firmware release.
   Devices update their trusted key set only after verifying the new
   manifest against the hardcoded master key.
The master key itself is not expected to rotate under normal operation.