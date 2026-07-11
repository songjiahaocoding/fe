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

You can also install v0.3 directly from GitHub with Cargo:

```sh
cargo install --git https://github.com/songjiahaocoding/fe --tag v0.3 --locked
```

## Quick Start

Read a value:

```sh
fe get config.json '$.server.host' --raw
```

Check the installed version:

```sh
fe --version
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
fe preview set config.json '$.server.port' 8080
```

`preview` works for every structured mutation, including a single deterministic
edit with no wildcard or regular expression. It prints the minimal unified diff
that the corresponding write command would produce and never writes the file:

```sh
fe preview delete config.yaml '$.features.legacy'
fe preview append config.json '$.plugins' '{"name":"auth"}'
fe preview insert config.json '$.plugins[0]' '{"name":"core"}'
```

The existing `--dry-run` / `--stdout` option remains available when the complete
serialized document is more useful than a diff.

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
- `preview` shows the exact diff without writing
- `batch` applies one structured operation to multiple nodes or files

## Batch Editing

Use `--file` more than once to edit explicit files:

```sh
fe batch set \
  --file config/dev.json \
  --file config/prod.json \
  '$.services[*].enabled' false
```

Or select files below a directory:

```sh
fe batch put \
  --root ./configs \
  --include '**/*.yaml' \
  --exclude '**/vendor/**' \
  '$.services[*]' timeout 30
```

Available batch operations:

```sh
# Set matched values
fe batch set --file config.json '$.services[*].enabled' false

# Add a key/value pair to matched objects
fe batch put --file config.json '$.services[*]' timeout 30

# Overwrite an existing key, or only add missing keys
fe batch put --file config.json '$.services[*]' timeout 30 --overwrite
fe batch put --file config.json '$.services[*]' timeout 30 --if-missing

# Delete matched nodes or object members selected by key
fe batch delete --file config.json '$.services[*].legacy'
fe batch delete --file config.json '$.services[*]' --key-regex '^x-legacy-'

# Replace text inside matched string values
fe batch replace --file config.json '$.services[*].image' '^old/' 'new/'

# Append to matched arrays
fe batch append --file config.json '$.groups[*].members' '"agent"'
```

Preview any batch operation by inserting `preview` before `batch`:

```sh
fe preview batch delete \
  --root ./configs \
  --include '**/*.yaml' \
  '$.services[*]' \
  --key-regex '^x-legacy-'
```

Preview prints the files that would change and their minimal diffs. It never
writes files.

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
