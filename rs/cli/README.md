# &#x26F5; Sails CLI

`sails-cli` is the command-line interface for working with Sails projects.
It provides commands to scaffold a new Sails workspace, generate clients from
IDL, extract IDL from Rust crates, and produce Solidity bindings.

The binary is exposed as the Cargo subcommand `cargo sails`.

## Usage

Install the CLI:

```bash
cargo install sails-cli
```

Or install a prebuilt binary with `cargo-binstall`:

```bash
cargo binstall sails-cli
```

Show available commands:

```bash
cargo sails --help
```

Create a new Sails project:

```bash
cargo sails new my-ping
```

Create a project without network access and use a local `sails-rs` path:

```bash
cargo sails new my-ping --offline --sails-path ../rs
```

Generate Rust client code from IDL:

```bash
cargo sails client-rs path/to/app.idl
```

Generate TypeScript client code from IDL:

```bash
cargo sails client-js path/to/app.idl
```

Generate IDL from a Cargo manifest:

```bash
cargo sails idl --manifest-path path/to/Cargo.toml
```

Embed IDL into a WASM binary as a custom section:

```bash
cargo sails idl-embed --wasm path/to/app.opt.wasm --idl path/to/app.idl
```

Extract IDL from a WASM binary:

```bash
cargo sails idl-extract --wasm path/to/app.opt.wasm
```

Generate Solidity artifacts from IDL:

```bash
cargo sails sol --idl-path path/to/app.idl
```

## Generated Sails Project Parts

The `cargo sails new` command creates a workspace with a few distinct parts:

- Root crate: the workspace entry point that wires dependencies together and
  contains the build logic for the final WASM and IDL artifacts.
- `app` crate: the package that contains the program business logic and service
  implementation.
- `client` crate: the package that builds the generated Rust client interface
  for interacting with the program.
- `tests` directory: integration tests based on `gtest` (opt into `gclient` for live-node tests).

Typical generated-project workflow:

```bash
cd my-ping
cargo test
cargo build
```

If you need Ethereum-compatible contracts for Vara.ETH, create the workspace
with `--eth`.
