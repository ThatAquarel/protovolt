# protov-scpi

Centralized ProtoV SCPI wire protocol: parse, encode, IEEE `#Nd` / `#H` blocks, update-mode policy, and optional host client.

## Features

| Feature | Description |
|---------|-------------|
| *(default)* | `no_std` codec (parse, block, policy, types) |
| `std` | `ScpiClient` + `Transport` trait |
| `limits` | FWUP constants from `protov-nvm` |
| `wasm` | `WasmScpiClient` for browsers (Web Serial via JS transport) |

Full wire-format reference: **[PROTOCOL.md](PROTOCOL.md)** (derived from `parse.rs`).

## Rust usage

```rust
use protov_scpi::{parse_command, ScpiCommand};

let cmd = parse_command("*IDN?").unwrap();
assert_eq!(cmd, ScpiCommand::IdnQuery);
```

Host client (enable `std`):

```rust
use protov_scpi::{IoTransport, ScpiClient};

let mut client = ScpiClient::new(IoTransport::new(&mut port));
client.query("*IDN?")?;
```

## WASM / Web Serial

Build (stable Rust; no wasm-pack):

```bash
just scpi-wasm-build
```

Output: `dist/wasm-scpi/` (`protov_scpi.js`, `protov_scpi_bg.wasm`, `package.json`).

On first run, installs `wasm-bindgen-cli` 0.2.126 if missing (matches workspace lockfile).

### Browser usage

Serve the repo root (or copy `dist/wasm-scpi/` into your app) and wire Web Serial into the transport callbacks:

```javascript
import init, { WasmScpiClient } from "/dist/wasm-scpi/protov_scpi.js";

await init();

const port = await navigator.serial.requestPort();
await port.open({ baudRate: 115200 });
const reader = port.readable.getReader();
const writer = port.writable.getWriter();
let lineBuf = "";

async function readLine(timeoutMs) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (lineBuf.includes("\n")) {
      const i = lineBuf.indexOf("\n");
      const line = lineBuf.slice(0, i).trim();
      lineBuf = lineBuf.slice(i + 1);
      return line;
    }
    const { value, done } = await reader.read();
    if (done) break;
    lineBuf += new TextDecoder().decode(value);
  }
  return null; // timeout
}

const client = new WasmScpiClient(
  (bytes) => writer.write(bytes),
  (timeoutMs) => readLine(timeoutMs),
  30000
);

console.log(client.query("*IDN?"));
```

`readLine` must resolve with a full line or `null` on timeout. `write` receives a `Uint8Array` command payload.

## Tests

```bash
just test-scpi
# or: cargo test -p protov-scpi --target $(rustc -vV | sed -n 's/host: //p') --features std
```
