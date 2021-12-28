# phar.rs
[![GitHub actions](https://github.com/SOF3/phar.rs/workflows/CI/badge.svg)](https://github.com/SOF3/phar.rs/actions?query=workflow%3ACI)
[![crates.io](https://img.shields.io/crates/v/phar.svg)](https://crates.io/crates/phar)
[![crates.io](https://img.shields.io/crates/d/phar.svg)](https://crates.io/crates/phar)
[![docs.rs](https://docs.rs/phar/badge.svg)](https://docs.rs/phar)
[![GitHub](https://img.shields.io/github/stars/SOF3/phar?style=social)](https://github.com/SOF3/phar)

Rust library for PHP phar format.

See the [tests/reader.rs](./tests/reader.rs) and [tests/writer.rs](./tests/writer.rs) directory for example code.

## Web
[![GitHub actions](https://github.com/SOF3/phar.rs/actions/workflows/page.yml/badge.svg)](https://github.com/SOF3/phar.rs/actions/workflows/page.yml)
[![GitHub pages](https://img.shields.io/badge/GitHub-Pages-white)](https://sof3.github.io/phar.rs)

As a proof of concept, phar.rs is used to create a light webapp to view phar files from file upload.

### Building
Prerequisites:

- Rust default toolchain (1.57.0 stable, probably works with earlier versions too)
- Trunk (`cargo install trunk`)

To build the site, simply `cd web` and run `trunk build trunk-dev.html` for unoptimized build,
or `trunk build trunk-release.html`.
See https://trunkrs.dev for more information.

## CLI
W.I.P.
