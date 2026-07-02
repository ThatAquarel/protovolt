# protov-bootloader

RP2040 **embassy-boot** bootloader for ProtoV, based on [Embassy's RP example](https://github.com/embassy-rs/embassy/tree/main/examples/boot/bootloader/rp).

Flash layout comes from [`protov-nvm`](../protov-nvm) (`linker/memory-bootloader.x`, `FLASH_SIZE`).

## Build / flash

```bash
just build                 # from protov-bootloader/
just bootloader::build       # from repo root
just bootloader::flash       # probe-rs via .cargo/config.toml
just bootloader::pkg         # target/protov-bootloader.uf2
```

Optional defmt:

```bash
cargo build -p protov-bootloader --release --features defmt
```

## Behaviour

On boot: init flash + watchdog, read ACTIVE/DFU/STATE partitions from linker symbols, run embassy-boot swap logic if needed, then jump to the application in ACTIVE (`0x10007000`).

The bootloader itself links into the ~24 KiB slot at `0x10000100`.
