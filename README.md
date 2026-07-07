# fe

`fe` is a small CLI for editing structured config files safely from scripts and coding agents.

Instead of asking an agent to rewrite JSON or YAML with brittle text patches, `fe` lets it read, set, append, insert, and delete values by path. The file is parsed, the target node is changed, and the result is serialized back as valid JSON or YAML.

## Why

Coding agents are good at reasoning about intent, but plain text editing is a poor interface for structured files. A tiny formatting mistake can break a config, and string replacement can hit the wrong key when the same text appears twice.

`fe` gives agents a sharper tool:

- Address values with JSONPath-style selectors like `$.server.port` or `$.plugins[0].enabled`
- Update JSON and YAML without hand-editing commas, indentation, or brackets
- Create missing object and array paths when setting values
- Get script-friendly failures with stable JSON error output
- Infer JSON/YAML format from the extension or from file contents
- Write edits back by default, with `--dry-run` or `--stdout` for previews

The result is fewer broken config edits, less retry traffic, and faster agent loops.

## Install

Install with Homebrew:

```sh
brew install songjiahaocoding/tap/fe
```

This installs a prebuilt `fe` binary, so users do not need a Rust toolchain.

You can also install v0.2.0 directly from GitHub with Cargo:

```sh
cargo install --git https://github.com/songjiahaocoding/fe --tag v0.2.0 --locked
```

## Quick Start

Read a value:

```sh
fe get config.json '$.server.host' --raw
```

Set a value:

```sh
fe set config.json '$.server.port' 8080
```

Create a nested value:

```sh
fe set config.yaml '$.database.primary.url' '"postgres://localhost/app"'
```

Append to an array:

```sh
fe append config.json '$.plugins' '{"name":"auth","enabled":true}'
```

Delete a key:

```sh
fe delete config.yaml '$.features.legacy'
```

Preview an edit without writing the file:

```sh
fe set config.json '$.server.port' 8080 --dry-run
```

Get machine-readable errors:

```sh
fe --error-format json get config.json '$.missing.path'
```

Example error:

```json
{"error":"path_not_found","message":"path not found: $.missing.path","path":"$.missing.path"}
```

## Commands

- `get` reads one or more values
- `exists` checks whether a path exists
- `set` replaces or creates a value
- `delete` removes an object key or array element
- `append` adds a value to an array
- `insert` inserts a value before an array index

## Path Support

`fe` supports a practical JSONPath-style subset for agent workflows:

```text
$.server.host
$["service-name"].port
$.plugins[0].enabled
$.plugins[-1].name
$.plugins[*].name
```

Wildcard paths are supported for reads. Mutating commands require deterministic paths so agents do not accidentally edit many nodes at once.

## Distribution Notes

The `fe` crate name is already taken on crates.io, so fe is distributed through Homebrew prebuilt binaries and GitHub plus `cargo install --git`.

A future crates.io package can use the package name `format-edit` while still installing a binary named `fe`:

```sh
cargo install format-edit
```
