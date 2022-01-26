# Ocular CLI

A CLI implementing the `ocular` rust crate.

## Getting Started

This CLI manages its own connections to multiple chains by caching data from the (chain registry)[]. Chain info gets written to a config path in your home directory by default at `~/.ocular/config.toml` the first time your run a command. It is recommended that you manage this file completely through the CLI and not attempt to make manual changes.

This application is authored using [Abscissa], a Rust application framework.

For more information, see:

[Documentation]

[Abscissa]: https://github.com/iqlusioninc/abscissa
[Documentation]: https://docs.rs/abscissa_core/

## Try it out

From inside the `ocular_cli` directory, run `cargo run -- chains agoric` (or use any other chain name from the registry).
