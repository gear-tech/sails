# Canonical Parity Vectors

Canonical parity vectors provide small, frozen JSON documents together with
their expected `interface_id` outputs.  The goal is to prove the hashing rules
are implemented consistently across toolchains (Rust, TypeScript, Python, …).

## Layout

```
vectors/
  canonical/
    simple_ping.json
    complex_multi.json
    manifest.json         # maps file name -> service -> expected interface_id
  README.md
```

Each canonical document is already normalized and includes the schema header
(`canon_schema`, `canon_version`, `hash`).  The manifest stores the interface id
in hexadecimal with a `0x` prefix.

## Updating vectors

1. Edit or add canonical JSON under `vectors/canonical/`.
2. Run `cargo run -p sails-cli -- sails idl-derive-id <file>` to obtain the
   canonical ids.
3. Update `manifest.json` so the expected id(s) match the CLI output.
4. Re-run the parity tests (see below).

## Available parity checks

- `cargo test -p sails-interface-id vectors` verifies the Rust implementation.
- `pnpm test:vectors` verifies the TypeScript implementation (requires
  installing JS dependencies via `pnpm install`).
- `python vectors/scripts/check_vectors.py` verifies the Python reference script
  (requires `pip install blake3`).
