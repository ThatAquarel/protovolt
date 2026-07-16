[![ProtoV MINI][cover]][website]

# A power supply to catalyze prototyping

[![Release][release-badge]][release]
[![CI][ci-badge]][ci]
[![Chat][chat-badge]][chat]
[![Crowd Supply][cs-badge]][cs]

**ProtoV MINI** is a credit card-sized, dual-channel lab power supply for
electronics prototyping and field testing. It is powered by USB-C PD and runs
open firmware written in Rust.

![ProtoV MINI powering a breadboard](docs/res/protovolt-connected-on-breadboard.jpg)

## Getting Started


## Development

```sh
git clone https://github.com/flakeblade/protov.git
cd protov
rustup target add thumbv6m-none-eabi
cargo install just
just build
```
See [development](#development-1) for prerequisites, tests, and common
commands. Run `just --list` to see every available recipe.

## Documentation

### Guides

- [Device overview](docs/product.md)

### Development

Project structure
- [Repository index, and workspace modules](docs/repository.md)
- [Firmware roadmap](docs/roadmap.md)

Installation, building, and uploading
1. [Toolchain installation, and building](docs/development.md)
2. [Release bundling, and firmware signature](docs/releases.md)
3. [Flashing to hardware](docs/flashing.md)


### References

Hardware compatibility
- [Power delivery sources](docs/compatibility.md)
- [Breadboards](docs/compatibility.md)


<!-- - [Product overview](docs/product.md)
    
- [Flashing](docs/flashing.md)


- [SCPI protocol](protov-scpi/PROTOCOL.md)
- [SCPI crate](protov-scpi/README.md)
- [Bootloader](protov-bootloader/README.md)
- [Flash layout and firmware verification](protov-nvm/README.md) -->

## Contributing

Bug reports and contributions are welcome in the
[GitHub repository](https://github.com/flakeblade/protov).

> [!WARNING]
> Modifying or uploading firmware is done entirely at your own risk. An
> incompatible image can leave the device inoperable and may require USB
> BOOTSEL or SWD recovery. Bypassing current, voltage, temperature, power, or
> other protection limits can damage ProtoV MINI, connected equipment, or
> wiring and may create an electrical or fire hazard. Verify the hardware
> revision, understand every safety-related change, and test conservatively.

## License

[Eclipse Public License 2.0](LICENSE) © Alex Xia — [@ThatAquarel](http://github.com/thataquarel)

<p align="center">
  <a href="https://flakeblade.com">
    <img src="docs/res/logo/fbrd_logo.svg" alt="Flake & Blade Robotics Design" >
  </a>
  <br>
  A Flake &amp; Blade Robotics Design project
</p>

[fbrd]: https://flakeblade.com
[release-badge]: https://github.com/flakeblade/protov/actions/workflows/release.yml/badge.svg
[release]: https://github.com/flakeblade/protov/actions/workflows/release.yml
[ci-badge]: https://github.com/flakeblade/protov/actions/workflows/ci.yml/badge.svg
[ci]: https://github.com/flakeblade/protov/actions/workflows/ci.yml
[chat-badge]: https://img.shields.io/badge/chat-discussions-success.svg
[chat]: https://github.com/flakeblade/protov/discussions
[cs-badge]: https://img.shields.io/badge/Crowd-Supply-099?labelColor=555
[cs]: https://www.crowdsupply.com/flake-and-blade-robotics-design/protov-mini
[cover]: docs/res/logo/protov_mini_cover.svg
[website]: https://protov.app
