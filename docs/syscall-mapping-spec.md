# Sails `Syscall` Runtime Mapping Specification

## Purpose

This document specifies how `Syscall` maps to the underlying runtime API on `wasm32`.

Its goal is narrow: define the canonical correspondence between each `Syscall::*` method and the concrete `gcore::*` call it delegates to.

## Scope

This specification applies to the implementation in `rs/src/gstd/syscalls.rs`.

- On `target_arch = "wasm32"`, `Syscall` is a thin wrapper over `gcore`.
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
| `Syscall::message_id()` | `MessageId` | `gcore::msg::id()` |  |
| `Syscall::message_size()` | `usize` | `gcore::msg::size()` |  |
| `Syscall::message_source()` | `ActorId` | `gcore::msg::source()` |  |
| `Syscall::message_value()` | `ValueUnit` | `gcore::msg::value()` |  |
| `Syscall::reply_to()` | `Result<MessageId, gcore::errors::Error>` | `gcore::msg::reply_to()` |  |
| `Syscall::reply_code()` | `Result<gcore::errors::ReplyCode, gcore::errors::Error>` | `gcore::msg::reply_code()` |  |
| `Syscall::signal_from()` | `Result<MessageId, gcore::errors::Error>` | `gcore::msg::signal_from()` | only when `ethexe` is disabled |
| `Syscall::signal_code()` | `Result<Option<gcore::errors::SignalCode>, gcore::errors::Error>` | `gcore::msg::signal_code()` | only when `ethexe` is disabled |
| `Syscall::program_id()` | `ActorId` | `gcore::exec::program_id()` |  |
| `Syscall::block_height()` | `u32` | `gcore::exec::block_height()` |  |
| `Syscall::block_timestamp()` | `u64` | `gcore::exec::block_timestamp()` |  |
| `Syscall::value_available()` | `ValueUnit` | `gcore::exec::value_available()` |  |
| `Syscall::gas_available()` | `GasUnit` | `gcore::exec::gas_available()` |  |
| `Syscall::env_vars()` | `gcore::EnvVars` | `gcore::exec::env_vars()` |  |
| `Syscall::exit(inheritor_id)` | `!` | `gcore::exec::exit(inheritor_id)` |  |
| `Syscall::panic(data)` | `!` | `gcore::ext::panic(data)` |  |
| `Syscall::read_bytes()` | `Result<Vec<u8>, gcore::errors::Error>` | allocate `vec![0u8; gcore::msg::size()]`, then `gcore::msg::read(result.as_mut())` |  |
| `Syscall::system_reserve_gas(amount)` | `Result<(), gcore::errors::Error>` | `gcore::exec::system_reserve_gas(amount)` | only when `ethexe` is disabled |

## Behavioral Notes

### Message Context

The `message_*`, `reply_*`, and `signal_*` methods are wrappers over `gcore::msg::*` accessors and therefore read the currently executing message context.

`read_bytes()` also operates on the current message context. It first sizes a buffer from `gcore::msg::size()` and then fills it through `gcore::msg::read(...)`.

### Execution Context

The `program_id`, `block_height`, `block_timestamp`, `value_available`, `gas_available`, `env_vars`, `exit`, and `system_reserve_gas` methods are wrappers over `gcore::exec::*` accessors and control flow.

### Panic Surface

`Syscall::panic(data)` does not use `gcore::msg` or `gcore::exec`. It delegates to `gcore::ext::panic(data)`.

## Non-WASM Behavior

### Non-WASM with `std`

On non-`wasm32` targets with feature `std`:

- Most `Syscall` getters read from thread-local mock state.
- Matching `with_*` setters exist for tests, for example `with_message_id`, `with_message_source`, and `with_program_id`.
- `env_vars()` returns a constructed `gcore::EnvVars` value rather than delegating to `gcore::exec::env_vars()`.
- `exit()` and `panic()` call Rust `panic!` with diagnostic text instead of delegating to runtime syscalls.
- `read_bytes()` reads from thread-local mock state.
- `system_reserve_gas()` returns `Ok(())` when present, that is, when `ethexe` is disabled.

These behaviors are intentionally test-oriented and are not part of the `gcore` mapping defined above.

### Non-WASM without `std`

On non-`wasm32` targets without feature `std`, all `Syscall` methods are unimplemented and panic if called.

## Source of Truth

The source of truth for this mapping is the `wasm32` implementation in `rs/src/gstd/syscalls.rs`.
