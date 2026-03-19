# Sails `Syscall` to `gstd` Mapping Specification

## Purpose

This document specifies how `Syscall` maps to the underlying `gstd` runtime API on `wasm32`.

Its goal is narrow: define the canonical correspondence between each `Syscall::*` method and the concrete `gstd::*` call it delegates to.

## Scope

This specification applies to the implementation in `rs/src/gstd/syscalls.rs`.

- On `target_arch = "wasm32"`, `Syscall` is a thin wrapper over `gstd` and `gcore`.
- On non-`wasm32` targets with feature `std`, `Syscall` is a test/mock surface backed by thread-local state.
- On non-`wasm32` targets without feature `std`, `Syscall` methods are intentionally unimplemented.

Unless stated otherwise, the mappings below are normative only for the `wasm32` implementation.

## Type Aliases

The following Sails aliases are used by `Syscall`:

- `ValueUnit = u128`
- `GasUnit = u64`

See `rs/src/types.rs`.

## Normative Mapping

| `Syscall` method | Return type | Underlying call | `ethexe` gating |
| --- | --- | --- | --- |
| `Syscall::message_id()` | `MessageId` | `gstd::msg::id()` |  |
| `Syscall::message_size()` | `usize` | `gstd::msg::size()` |  |
| `Syscall::message_source()` | `ActorId` | `gstd::msg::source()` |  |
| `Syscall::message_value()` | `ValueUnit` | `gstd::msg::value()` |  |
| `Syscall::reply_to()` | `Result<MessageId, gcore::errors::Error>` | `gstd::msg::reply_to()` |  |
| `Syscall::reply_code()` | `Result<ReplyCode, gcore::errors::Error>` | `gstd::msg::reply_code()` |  |
| `Syscall::signal_from()` | `Result<MessageId, gcore::errors::Error>` | `gstd::msg::signal_from()` | only when `ethexe` is disabled |
| `Syscall::signal_code()` | `Result<Option<SignalCode>, gcore::errors::Error>` | `gstd::msg::signal_code()` | only when `ethexe` is disabled |
| `Syscall::program_id()` | `ActorId` | `gstd::exec::program_id()` |  |
| `Syscall::block_height()` | `u32` | `gstd::exec::block_height()` |  |
| `Syscall::block_timestamp()` | `u64` | `gstd::exec::block_timestamp()` |  |
| `Syscall::value_available()` | `ValueUnit` | `gstd::exec::value_available()` |  |
| `Syscall::gas_available()` | `GasUnit` | `gstd::exec::gas_available()` |  |
| `Syscall::env_vars()` | `gstd::EnvVars` | `gstd::exec::env_vars()` |  |
| `Syscall::exit(inheritor_id)` | `!` | `gstd::exec::exit(inheritor_id)` |  |
| `Syscall::panic(data)` | `!` | `gstd::ext::panic_bytes(data)` |  |

## Behavioral Notes

### Message Context

The `message_*`, `reply_*`, and `signal_*` methods are wrappers over `gstd::msg::*` accessors and therefore read the currently executing message context.

### Execution Context

The `program_id`, `block_height`, `block_timestamp`, `value_available`, `gas_available`, `env_vars`, and `exit` methods are wrappers over `gstd::exec::*` accessors and control flow.

### Panic Surface

`Syscall::panic(data)` does not use `gstd::msg` or `gstd::exec`. It delegates to `gstd::ext::panic_bytes(data)` and is intended for structured panic payloads.

## Non-WASM Behavior

### Non-WASM with `std`

On non-`wasm32` targets with feature `std`:

- Most `Syscall` getters read from thread-local mock state.
- Matching `with_*` setters exist for tests, for example `with_message_id`, `with_message_source`, and `with_program_id`.
- `env_vars()` returns a constructed `gstd::EnvVars` value rather than delegating to `gstd::exec::env_vars()`.
- `exit()` and `panic()` call Rust `panic!` with diagnostic text instead of delegating to runtime syscalls.

These behaviors are intentionally test-oriented and are not part of the `gstd` mapping defined above.

### Non-WASM without `std`

On non-`wasm32` targets without feature `std`, all `Syscall` methods are unimplemented and panic if called.

## Source of Truth

The source of truth for this mapping is the `wasm32` implementation in `rs/src/gstd/syscalls.rs`.
