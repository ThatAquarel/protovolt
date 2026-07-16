[![ProtoV MINI][cover]][website]

# Power that catalyzes the workspace

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
See [development](docs/development.md) for prerequisites, tests, and common
commands. Run `just --list` to see every available recipe.

## Documentation

<!-- - [Product overview](docs/product.md)
- [Repository guide](docs/repository.md)
- [Development](docs/development.md)
- [Flashing](docs/flashing.md)
- [Release builds](docs/releases.md)
- [SCPI protocol](protov-scpi/PROTOCOL.md)
- [SCPI crate](protov-scpi/README.md)
- [Bootloader](protov-bootloader/README.md)
- [Flash layout and firmware verification](protov-nvm/README.md) -->

## Contributing

Bug reports and contributions are welcome in the
[GitHub repository](https://github.com/flakeblade/protov).

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
