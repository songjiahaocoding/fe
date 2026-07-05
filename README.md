# fe

Agent-friendly structured file editor for JSON and YAML.

## Install

The v0.1 release can be installed directly from GitHub with Cargo:

```sh
cargo install --git https://github.com/songjiahaocoding/fe --tag v0.1 --locked
```

This installs the `fe` executable into Cargo's install bin directory, usually `~/.cargo/bin`.

To reinstall or upgrade the same tag:

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

The `fe` crate name is already taken on crates.io, so the immediate v0.1 distribution channel is GitHub plus `cargo install --git`.

A future crates.io package can use the package name `format-edit` while still installing a binary named `fe`, which would allow:

```sh
cargo install format-edit
```
