# fe

Agent-friendly structured file editor for JSON and YAML.

## Install

Install with Homebrew:

```sh
brew install songjiahaocoding/tap/fe
```

This installs a prebuilt `fe` binary, so users do not need a Rust toolchain.

The v0.1 release can also be installed directly from GitHub with Cargo:

```sh
cargo install --git https://github.com/songjiahaocoding/fe --tag v0.1 --locked
```

To reinstall or upgrade the same Cargo tag:

```sh
cargo install --git https://github.com/songjiahaocoding/fe --tag v0.1 --locked --force
```

## Usage

```sh
fe get config.json '$.server.host' --raw
fe set config.json '$.server.port' 8080 --write
fe append config.json '$.plugins' '{"name":"auth","enabled":true}' --write
fe delete config.yaml '$.features.legacy' --write
```

Use JSON-formatted errors for scripts and coding agents:

```sh
fe --error-format json get config.json '$.missing.path'
```

## Distribution Notes

The `fe` crate name is already taken on crates.io, so v0.1 is distributed through Homebrew prebuilt binaries and GitHub plus `cargo install --git`.

A future crates.io package can use the package name `format-edit` while still installing a binary named `fe`, which would allow:

```sh
cargo install format-edit
```
