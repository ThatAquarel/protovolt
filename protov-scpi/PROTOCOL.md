# ProtoV SCPI wire protocol

This document is the **ground-truth reference** for the ProtoV SCPI line protocol as implemented in `protov-scpi`. It is derived directly from `parse.rs`, `policy.rs`, `block.rs`, `types.rs`, and `command.rs`. Device-side semantics (what a command *does*) live in `protov-core`; this crate owns **syntax**, **framing**, and **session policy** on the wire.

## Transport

- **Physical layer:** USB CDC serial (typical host tools) or any byte stream with the same line discipline.
- **Framing:** One command or response per line, terminated by ASCII `LF` (`0x0A`). Hosts may send `CRLF`; parsers trim leading/trailing whitespace before normalization.
- **Encoding:** ASCII command headers. Binary payload for firmware blocks uses IEEE 488.2 definite-length blocks (see [Binary blocks](#binary-blocks)).
- **Line limits:** Normalized commands fit in 256 bytes (`LINE_BUF`). Response text is capped at 512 bytes (`RESPONSE_BUF`) on the device.

## Normalization

Every command passes through `normalize_command` before parsing:

1. Trim leading and trailing whitespace.
2. Uppercase all ASCII letters (`ch1:volt?` → `CH1:VOLT?`).
3. Collapse runs of spaces and tabs to a single space.
4. Strip trailing spaces.

Empty lines after normalization are rejected (`parse_command` returns `None`).

## Identifiers

| Symbol | Accepted tokens | Meaning |
|--------|-----------------|---------|
| Output channel | `CH1`, `CH2` | Front-panel channels |
| Register channel | `CHA`, `CHB` | I²C rail A / B (maps to hardware channels) |
| Temperature slot | `CHA`, `CHB`, `MCU` | Channel A sense, channel B sense, MCU die |

## Numeric formats

| Type | Grammar | Range / notes |
|------|---------|---------------|
| Integer (slot) | Single digit `1`–`9` | `*SAV`, `*RCL`, `*DEL` |
| Integer (size) | One or more ASCII digits | `SYST:FWUP:STAR`; must fit `u32` |
| Float | Optional `+`/`-`, digits, optional `.` fraction | Channel setpoints; no exponent |
| Brightness | Decimal integer | `0`–`255` for `LCD:BRIG` / `LED:BRIG` |
| RGB triplet | `R,G,B` comma-separated decimals | Each component `0`–`255` |

## Command reference

Commands are listed in the form accepted **after normalization**. `?` suffix denotes a query (host expects a response line). Commands without `?` are mutations unless noted.

### IEEE 488.2 common commands

| Command | Type | Parsed as |
|---------|------|-----------|
| `*IDN?` | Query | Identity string (device-specific) |
| `*RST` | Mutation | Factory reset (silent on bus; errors via `SYST:ERR?`) |
| `*SAV n` | Mutation | Save setup to slot `n` (`n` = `1`–`9`) |
| `*RCL n` | Mutation | Recall setup from slot `n` |
| `*DEL n` | Mutation | Delete saved setup in slot `n` |

Slot commands require exactly one digit after the prefix (e.g. `*SAV 3`). `*SAV 0` or `*SAV 12` does **not** match; such lines fall through to [Unknown](#unknown-commands).

### System

| Command | Type | Notes |
|---------|------|-------|
| `SYST:ERR?` | Query | Error queue; see [Errors](#errors) |
| `SYST:VERS?` | Query | System / protocol version string |
| `SYST:IDAT?` | Query | Authenticated device identification; see [Device identification](#device-identification) |
| `SYST:LOC` | Mutation | Local front-panel control |
| `SYST:REM` | Mutation | Remote (SCPI) control |

### Device identification

| Command | Type | Response |
|---------|------|----------|
| `*IDN?` | Query | Full identity string (manufacturer, product, serial, firmware, hardware) |
| `SYST:IDAT?` | Query | Serial number, hardware revision, manufacturing date, flash unique ID, and Ed25519 serial attestation signature |

`*IDN?` returns a comma-separated identity line (see [IEEE 488.2 common commands](#ieee-4882-common-commands)).

`SYST:IDAT?` returns authenticated manufacturing identity for host verification against the
embedded HW trust manifest (`protov-nvm`):

```
{serial},{hw_version},{YYYY-MM-DD},#H{16 hex uppercase},#H{128 hex uppercase}
```

| Field | Description |
|-------|-------------|
| `serial` | Device serial number (ASCII, same token as field 3 of `*IDN?`) |
| `hw_version` | Hardware revision string (same token as field 5 of `*IDN?`) |
| `YYYY-MM-DD` | Manufacturing / attestation date (ISO 8601 calendar date) |
| `#H…` (16 hex) | 8-byte RP2040 flash unique ID |
| `#H…` (128 hex) | 64-byte Ed25519 signature over **`{serial},{hw_version},{YYYY-MM-DD}`** (UTF-8/ASCII, no pre-hash) |

Example (truncated):

```
550e8400,A.1,2026-06-27,#H5555555555555555,#H55…55
```

Hosts verify the signature with any public key listed in the root-signed HW manifest
(`PUBLIC_KEY_HWX` in `protov-nvm`). Firmware revision is intentionally omitted; use `*IDN?`
for the full five-field identity string.

Both identification queries are non-mutations and remain available during FWUP update mode.

Host tools may verify `SYST:IDAT?` responses with
`protov_nvm::verify_serial_attestation(serial, hw_version, (year, month, day), &signature_bytes)`
after parsing the comma-separated fields (or build the message with
`protov_nvm::encode_attestation_message`).

### Measurement

| Command | Type |
|---------|------|
| `MEAS:VOLT? CH1` | Query measured voltage on CH1 |
| `MEAS:VOLT? CH2` | Query measured voltage on CH2 |
| `MEAS:CURR? CH1` | Query measured current |
| `MEAS:CURR? CH2` | Query measured current |
| `MEAS:POW? CH1` | Query measured power |
| `MEAS:POW? CH2` | Query measured power |

Format: `MEAS:<KIND>? <CHANNEL>` where `<KIND>` is `VOLT`, `CURR`, or `POW`, and `<CHANNEL>` is `CH1` or `CH2`. A single space separates the `?` from the channel token.

### Channel setpoints and appearance

**Queries** — `CHn:<PARAM>?` (no trailing channel token):

| Command | Parameter |
|---------|-----------|
| `CH1:VOLT?` / `CH2:VOLT?` | Voltage setpoint |
| `CH1:CURR?` / `CH2:CURR?` | Current limit |
| `CH1:OVP?` / `CH2:OVP?` | Over-voltage protection |
| `CH1:OCP?` / `CH2:OCP?` | Over-current protection |
| `CH1:COLR?` / `CH2:COLR?` | RGB color (`R,G,B`) |
| `CH1:MODE?` / `CH2:MODE?` | Channel mode |

**Sets** — two space-separated tokens: `CHn:<PARAM> <value>`

| Command | Example |
|---------|---------|
| `CH1:VOLT <float>` | `CH1:VOLT 5.0` |
| `CH1:CURR <float>` | `CH1:CURR 2.5` |
| `CH1:OVP <float>` | `CH1:OVP 5.5` |
| `CH1:OCP <float>` | `CH1:OCP 3.0` |

**Color set** — `CHn:COLR R,G,B`:

```
CH1:COLR 234,67,53
CH2:COLR 66,133,244
```

### Output enable

| Command | Type | Format |
|---------|------|--------|
| `OUTP CH1,ON` | Mutation | Comma between channel and state |
| `OUTP CH1,OFF` | Mutation | |
| `OUTP CH2,ON` | Mutation | |
| `OUTP CH2,OFF` | Mutation | |
| `OUTP? CH1` | Query | Space after `?` |
| `OUTP? CH2` | Query | |
| `OUTP:RESET:PROT` | Mutation | Clear protection, all channels |
| `OUTP:RESET:PROT CH1` | Mutation | Clear protection, one channel |
| `OUTP:RESET:PROT CH2` | Mutation | |

### Brightness

| Command | Type |
|---------|------|
| `LCD:BRIG?` | Query LCD backlight `0`–`255` |
| `LCD:BRIG <n>` | Set LCD backlight |
| `LED:BRIG?` | Query LED brightness `0`–`255` |
| `LED:BRIG <n>` | Set LED brightness |

### Telemetry and diagnostics

| Command | Type |
|---------|------|
| `TELEM?` | Aggregated telemetry snapshot |
| `TEMP? CHA` | Temperature, channel A |
| `TEMP? CHB` | Temperature, channel B |
| `TEMP? MCU` | MCU temperature |
| `INP?` | Input source (PD / adapter) |
| `DIAG?` | Diagnostic summary |
| `INA226:REG? CHA` | INA226 register dump, rail A |
| `INA226:REG? CHB` | INA226 register dump, rail B |
| `TPS55289:REG? CHA` | TPS55289 register dump, rail A |
| `TPS55289:REG? CHB` | TPS55289 register dump, rail B |

**Informal aliases** (same parsed command as the formal register queries):

| Alias | Equivalent |
|-------|------------|
| `INA226 DUMP CHA` | `INA226:REG? CHA` |
| `INA226 DUMP CHB` | `INA226:REG? CHB` |
| `TPS55289 DUMP CHA` | `TPS55289:REG? CHA` |
| `TPS55289 DUMP CHB` | `TPS55289:REG? CHB` |

Note: informal forms use `A` / `B` after the `CH` prefix, not `CHA` / `CHB`.

### Firmware update (FWUP)

Constants (default; enable `limits` feature to align with `protov-nvm`):

| Constant | Value |
|----------|-------|
| `FWUP_MAX_IMAGE_SIZE` | 819 200 bytes (800 KiB) |
| `FWUP_PAGE_SIZE` | 4096 bytes |
| `FWUP_MAX_BLOCK_LEN` | 4096 bytes |
| `FWUP_SIGNATURE_LEN` | 64 bytes |

| Command | Type | Wire form |
|---------|------|-----------|
| `SYST:FWUP:STAT?` | Query | Session phase string (see below) |
| `SYST:FWUP:STAR <size>` | Mutation | Begin update; `<size>` = total image bytes |
| `SYST:FWUP:DATA …` | Mutation | Binary data line; see [FWUP DATA](#fwup-data) |
| `SYST:FWUP:APPL …` | Mutation | Apply signature; see [FWUP APPL](#fwup-appl) |
| `SYST:FWUP:ABOR` | Mutation | Abort session |

**Typical host sequence**

```
SYST:FWUP:STAR <image_size>     → OK or silent
SYST:FWUP:DATA #Nd<payload>     → OK (repeat)
SYST:FWUP:APPL #H<hex128>       → OK
SYST:FWUP:STAT?                 → until VERIFIED / FLASHING
```

#### FWUP STAT responses

| Value | Meaning |
|-------|---------|
| `IDLE` | No active session |
| `PREPARE,<total>` | Erasing / preparing flash |
| `RECV,<received>/<total>` | Receiving image bytes |
| `READY,<total>` | Image complete, awaiting signature |
| `ERROR` | Session failed |
| `VERIFIED,<total>` | Signature OK (device-specific follow-up) |
| `FLASHING` | Bootloader flashing (device-specific) |

#### FWUP DATA

ASCII prefix followed by a definite-length block and newline:

```
SYST:FWUP:DATA #44096<4096 bytes of payload>\n
```

The parser recognizes any line starting with `SYST:FWUP:DATA`; the `#Nd` header and raw payload are assembled by the transport layer. Valid payload length: `1`–`FWUP_MAX_BLOCK_LEN`.

#### FWUP APPL

```
SYST:FWUP:APPL #H<128 hex digits>\n
```

The hex block decodes to exactly **64 bytes** (Ed25519 signature). Optional `#H` prefix on the hex string is accepted by `decode_hex_block`.

## Binary blocks

ProtoV uses two IEEE 488.2 block styles.

### Definite-length decimal (`#Nd`)

```
#<N><d…d><payload>
```

- `<N>` — one ASCII digit `1`–`9`: count of length digits that follow.
- `<d…d>` — decimal length of payload in bytes.
- `<payload>` — raw bytes immediately after the length field.

Examples:

| Header | Payload length |
|--------|----------------|
| `#5128` | 128 |
| `#44096` | 4096 |

`parse_definite_block_at` returns `(header_byte_len, payload_byte_len)`.

### Definite-length hex (`#H…`)

Hex digits only, even count, max 256 hex chars (128 bytes). Used for the FWUP signature. A leading `#H` may be present or omitted in the hex argument.

## Response model

`protov-scpi` defines how responses are represented in firmware; typical bus behavior:

| Command class | Bus line |
|---------------|----------|
| Query (`…?`) | One line of ASCII text |
| Mutation | **Silent** (no line), or `OK` for FWUP handshakes |
| `*RST`, channel sets, `OUTP`, etc. | Silent; check `SYST:ERR?` on failure |

FWUP mutations (`STAR`, `DATA`, `APPL`, `ABOR`) may reply with `OK` or an empty line; host clients accept either.

### Errors

`SYST:ERR?` returns a single line:

```
<code>,"<message>"
```

Example: `0,"No error"`. Negative codes indicate faults; the queue is FIFO on the device.

## Session policy

Policy functions classify parsed commands for update mode and standby gating (enforced in `protov-core`, rules defined here).

### Mutations vs queries

**Queries** (not mutations): `*IDN?`, all `MEAS:?`, all `CHn:*?`, `OUTP?`, `LCD:BRIG?`, `LED:BRIG?`, `SYST:ERR?`, `SYST:VERS?`, `TELEM?`, `TEMP?`, `INP?`, `DIAG?`, `INA226:REG?`, `TPS55289:REG?`, `SYST:FWUP:STAT?`, and **Unknown** commands.

**Mutations** (everything else): includes `*RST`, `*SAV`/`*RCL`/`*DEL`, channel sets, `OUTP`, brightness sets, `SYST:LOC`/`SYST:REM`, `SYST:FWUP:STAR`, etc.

Mutations require **Standby** on the device unless in FWUP update mode.

### Update mode whitelist

When the device is in FWUP update mode, only these commands are accepted:

- `*IDN?`
- `SYST:ERR?`
- `SYST:VERS?`
- `SYST:FWUP:STAT?`
- `SYST:FWUP:DATA`
- `SYST:FWUP:ABOR`
- `SYST:FWUP:APPL`

`SYST:FWUP:STAR` starts update mode from normal operation (not whitelisted inside update mode).

### Active session requirement

`SYST:FWUP:DATA` and `SYST:FWUP:APPL` require an active FWUP session (after successful `SYST:FWUP:STAR`) when not already in update mode.

## Unknown commands

If no parser matches, the line is classified as **Unknown**: up to the first 64 bytes of the normalized command are retained. Unknown commands are treated as **non-mutations** for policy purposes; the device responds with an error via `SYST:ERR?` rather than a bus line.

## Host encoding

With the `std` feature, `encode_command_line`, `encode_fwup_data_line`, and `encode_fwup_appl_line` build wire lines from `ScpiCommand` values. **The parser in this document is authoritative** for what the device accepts; some encoder paths use alternate spellings (e.g. `SYST:BRIG:LCD` vs `LCD:BRIG`) and must not be used when talking to ProtoV firmware.

## Related crates

| Crate | Role |
|-------|------|
| `protov-scpi` | Parse, encode, block math, policy, host `ScpiClient` / WASM |
| `protov-core` | Command semantics, error queue, FWUP state machine |
| `protov-hal` | USB CDC transport, FWUP payload reassembly, flash backend |
