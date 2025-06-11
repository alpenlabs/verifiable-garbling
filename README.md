# Alpen Labs Rust Workspace Template

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache-blue.svg)](https://opensource.org/licenses/apache-2-0)
[![ci](https://github.com/alpenlabs/rust-template/actions/workflows/lint.yml/badge.svg?event=push)](https://github.com/alpenlabs/rust-template/actions)
[![docs](https://img.shields.io/badge/docs-docs.rs-orange)](https://docs.rs/rust-template)

This repo is a template for easy setup of a Rust workspace project within
[`AlpenLabs` GitHub organization](https://github.com/alpenlabs).
If you are looking for the single crate template, you can find it at
[`alpenlabs/rust-template](https://github.com/alpenlabs/rust-template).

- It comes with a preconfigured `.justfile` for common tasks.
- Licensing is taken care of, with dual MIT-Apache 2.0 licenses.
- Continuous Integration is already set up with the common GitHub actions jobs
hardened with [`zizmor`](https://docs.zizmor.sh).
- Dependabot is enabled to automatically bump Rust and GitHub actions dependencies monthly.
- There are 1 pull request template and 2 issues templates for bug reports and feature requests.
- Proper lints for code maintainability are added to `Cargo.toml`.
- If you need to publish crates to `crates.io`, you can use the `just publish` command,
  and it will be automatically triggered by CI on every new tag release.
  You just need to add a crates.io token to the `CARGO_REGISTRY_TOKEN` repository secret variable.

This template has a lot of `CHANGEME` placeholders that you should replace with your own values.
Please do a repository-wide search and replace all occurrences of `CHANGEME` with your own values.

## Features

- Feature 1
- Feature 2

## Usage

```rust
// How to use the library/binary.
```

## Contributing

Contributions are generally welcome.
If you intend to make larger changes please discuss them in an issue
before opening a PR to avoid duplicate work and architectural mismatches.

For more information please see [`CONTRIBUTING.md`](/CONTRIBUTING.md).

## License

This work is dual-licensed under MIT and Apache 2.0.
You can choose between one of them if you use this work.
