# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What is Sails

Sails is a framework for building applications on [Gear Protocol](https://gear-tech.io/) / [Vara Network](https://vara.network/). It provides procedural macros (`#[program]`, `#[service]`, `#[export]`, `#[event]`, `#[sails_type]`), IDL generation, and multi-language client generation (Rust, TypeScript, Solidity). The main crate is published as `sails-rs` on crates.io.

## Repository Layout

- **`rs/`** — Core Rust crates (the `sails-rs` crate lives at `rs/` itself):
  - `rs/macros/`, `rs/macros/core/` — Procedural macros (`#[service]`, `#[program]`, `#[export]`)
  - `rs/idl-ast/`, `rs/idl-gen/`, `rs/idl-meta/`, `rs/idl-parser/`, `rs/idl-parser-v2/` — IDL generation and parsing
  - `rs/client-gen/`, `rs/client-gen-v2/`, `rs/client-gen-js/` — Client code generation (Rust, JS)
  - `rs/sol-gen/` — Solidity interface generation
  - `rs/type-registry/`, `rs/reflect-hash/` — Type registry and interface hashing
  - `rs/idl-embed/` — Embeds IDL into WASM binaries
  - `rs/cli/` — `sails-cli` (`cargo sails` subcommand)
  - `rs/ethexe/` — Separate workspace for ethexe-specific apps/tests (own `Cargo.toml`)
  - `rs/src/gstd/` — Gear standard library integration, syscall abstractions
  - `rs/src/client/` — Client environments (`GstdEnv`, `GclientEnv`, `GtestEnv`)
- **`js/`** — TypeScript/JS packages (pnpm workspace):
  - `js/` (root) — `sails-js` main library
  - `js/parser/`, `js/parser-idl-v2/` — IDL parsers (uses WASM-compiled Rust parser)
  - `js/cli/` — JS CLI for client generation
  - `js/types/`, `js/util/` — Shared types and utilities
- **`net/rs/`** — Separate Rust workspace for .NET-related client generation
- **`examples/`** — Example Sails applications (demo, redirect, rmrk, proxy, etc.)
- **`benchmarks/`** — Performance benchmarks

## Build Commands

### Rust

```bash
# Check formatting
cargo fmt --all --check

# Lint (what CI runs)
cargo clippy --workspace --all-targets --locked -- -D warnings

# Lint with ethexe feature
cargo clippy -p sails-rs --all-targets --locked --features ethexe -- -D warnings

# Lint ethexe workspace (separate workspace)
cargo clippy --workspace --all-targets --locked --manifest-path ./rs/ethexe/Cargo.toml -- -D warnings

# Run all workspace tests
cargo test --workspace --all-targets --locked --no-fail-fast -- --include-ignored

# Run tests for a single crate
cargo test -p sails-macros-core --locked

# Run a single test
cargo test -p sails-macros-core --locked -- test_name

# Run ethexe tests (separate workspace)
cargo test --workspace --all-targets --locked --no-fail-fast --manifest-path ./rs/ethexe/Cargo.toml

# Build WASM IDL parser (needed for JS tests)
cargo build -p sails-idl-parser-wasm --target wasm32-unknown-unknown --release
wasm-opt -O4 -o ./target/wasm32-unknown-unknown/release/sails_idl_v2_parser.wasm ./target/wasm32-unknown-unknown/release/sails_idl_parser_wasm.wasm
```

### JavaScript/TypeScript

```bash
pnpm install          # Install deps
pnpm build            # Build all JS packages
pnpm test             # Run JS tests (sails-js)
pnpm test:cli         # Run CLI tests
pnpm lint             # ESLint
pnpm format           # Prettier
```

## Key Architecture Concepts

### Dual Workspace Structure

The root `Cargo.toml` workspace contains most crates. The `rs/ethexe/` directory is a **separate Cargo workspace** with its own `Cargo.toml`. CI checks both independently.

### Macro Expansion Pipeline

`#[service]` and `#[program]` macros (in `rs/macros/core/`) generate:
1. An `Exposure` struct that wraps the service, implementing `Deref`/`DerefMut` to the underlying type
2. Request decoding/dispatch logic based on [Sails header](#sails-header)
3. Response encoding
4. Event emission methods (when `events = SomeEnum` is specified)

Macro tests use `insta` for snapshot testing (in `rs/macros/core/`) and `trybuild` for compile-fail tests.

### IDL v1

Maintained for compatibility with deployed programs and their client generation.
- Parser `rs/idl-parser/`.
- Grammar: **LALRPOP** (`grammar.lalrpop`) with a hand-written lexer.
- Produces its own AST (`rs/idl-parser/src/ast/`), independent from the shared `sails-idl-meta` / `sails-idl-ast` model used by the v2 toolchain.
- Used by the original `rs/client-gen/` and the JS toolchain.

### IDL v2

Current version.
- Parser `rs/idl-parser-v2/`.
- Grammar: **Pest** (`idl.pest`) — declarative PEG.
- `rs/idl-parser-v2` exposes the shared AST as `ast` via `pub use sails_idl_meta as ast;`, while `sails-idl-meta` re-exports the underlying types from `sails-idl-ast` with `pub use sails_idl_ast::*;`. This keeps one canonical type model across the v2 parser and the rest of the toolchain.
- **`no_std` + `alloc`** core with an opt-in `std` feature — the same crate runs in WASM and on host.
- **Preprocessor with `!@include` directives** (`rs/idl-parser-v2/src/preprocess/`): an `IdlLoader` trait with built-in `FsLoader` and `GitLoader` implementations, enabling multi-file IDLs assembled from local paths or `git://` URLs, with per-source deduplication.
- Consumed by `rs/client-gen-v2/`, `rs/sol-gen/`, and the JS toolchain.
- WASM bridge (`rs/idl-parser-wasm/`): C ABI (`parse_idl_to_json`) over `parse_idl`, compiled to `wasm32-unknown-unknown` and loaded by `js/parser/` so JS reuses the Rust grammar.

### Sails Header

Messages use a 16-byte binary header (magic "GM", version, interface ID, entry ID, route index) prepended to SCALE-encoded payloads. Interface and entry IDs are deterministic hashes derived from canonical IDL definitions.

### Async Architecture (gstd)

Gear Protocol uses an actor-model where programs communicate via asynchronous messages. Sails wraps this into a structured async pipeline:

**Runtime entry points** — The `#[program]` macro generates `extern "C"` functions (`init`, `handle`, `handle_reply`, `handle_signal`) that the Gear runtime calls. These are the WASM exports.

**Sync vs async dispatch** — Each service method is statically known to be sync or async at macro expansion time. The generated `Exposure` struct implements both `try_handle` (sync) and `try_handle_async` (async) methods. At runtime, `check_asyncness(interface_id, entry_id)` looks up the method and picks the right path. When the entire service has no async methods (`ASYNC = false`), the check short-circuits to sync. The `service_route_dispatch!` macro (`rs/src/gstd/macros.rs`) orchestrates this: it queries asyncness, then either calls `try_handle` directly or wraps `try_handle_async` in `gstd::message_loop()`.

**`message_loop`** — Gear's cooperative async executor (from `gstd`). It runs a future to completion within the gas-limited execution environment. Used for both async constructors (`program_ctor!` macro) and async message handling. Not a general-purpose runtime — it processes exactly one future per invocation.

**Client-side async (GstdEnv)** — When a Sails program calls another program, `GstdEnv` (`rs/src/client/gstd_env.rs`) creates a `MessageFuture` via `gstd::msg::send_bytes_for_reply`. The future resolves when the Gear runtime delivers the reply. Configurable via `.with_params()`: gas limit, value, wait timeout (`.up_to()`), reply deposit, and reply hooks. The `PendingCall<T, GstdEnv>` struct is a `Future` that encodes the call, sends the message, and decodes the reply.

**Three client environments** implement the `GearEnv` trait:
- `GstdEnv` — on-chain program-to-program calls via `gstd::msg`
- `GtestEnv` (`rs/src/client/gtest_env.rs`) — local testing via `gtest` crate
- `GclientEnv` (`rs/src/client/gclient_env.rs`) — off-chain RPC calls via `gclient`

All three share the same generated client code; only the environment differs.

**Reply/signal handling** — `handle_reply` delegates to `gstd::handle_reply_with_hook()` when the program has async methods (`ASYNC = true`). `handle_signal` similarly delegates to `gstd::handle_signal()`. These enable the runtime to wake pending futures when replies arrive or signals fire.

### Feature Flags on `sails-rs`

- `gstd` (default) — On-chain Gear standard library
- `gtest` — Test environment with `GtestEnv`
- `gclient` — Off-chain client via `gclient`
- `ethexe` — Ethereum execution layer support (Solidity keywords validation, payable methods)
- `build` — Combines `client-builder` + `wasm-builder` for build scripts
- `mockall` — Mock generation for testing

### Environment Variable

Set `__GEAR_WASM_BUILDER_NO_FEATURES_TRACKING=1` to disable WASM builder feature tracking (CI always sets this).

## Rust Edition & Toolchain

- Edition 2024, MSRV `1.91`
- WASM targets: `wasm32-unknown-unknown`, `wasm32v1-none`
- Dependencies on Gear crates are pinned to exact versions (`=1.10.0`)
- Several other deps are also pinned (parity-scale-codec, lalrpop, handlebars)

## Specs & References

- [docs/idl-v2-spec.md](docs/idl-v2-spec.md) — IDL v2 grammar, annotations, and semantics.
- [docs/sails-header-v1-spec.md](docs/sails-header-v1-spec.md) — Sails message header layout and wire format.
- [docs/interface-id-spec.md](docs/interface-id-spec.md) — how service interface IDs are computed at compile time.
- [docs/reflect-hash-spec.md](docs/reflect-hash-spec.md) — `ReflectHash` trait for structural type hashing.
- [docs/syscall-mapping-spec.md](docs/syscall-mapping-spec.md) — runtime mapping of Sails `Syscall`s onto Gear syscalls.
