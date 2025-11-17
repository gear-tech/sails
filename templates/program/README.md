## The **{{ project-name }}** program

The program workspace includes the following packages:
- `{{ project-name }}` is the package allowing to build WASM binary for the program and IDL file for it. {% if with-client and with-gtest %} 
  The package also includes integration tests for the program in the `tests` sub-folder{% endif %}
- `{{ app-project-name }}` is the package containing business logic for the program represented by the `{{ service-struct-name }}` structure. {% if with-client %} 
- `{{ client-project-name }}` is the package containing the client for the program allowing to interact with it from another program, tests, or
  off-chain client.
{% endif %}

The `{{ app-project-name }}` package now keeps its canonicalization targets in `sails_services.in`. Both the build script and the
`sails_meta_dump` helper include that manifest by first defining a local `sails_services_manifest!` macro (that expands to either
`sails_build::service_paths!` or `sails_build::service_manifest!`) and then `include!`-ing the file. Adding or removing services only
requires editing that single file. The file stores the bare `services: [ ... ]` payload (optionally wrapped in braces) and may declare
witness aliases before the `services` block if a generic service needs to be instantiated with a concrete client type.

`build.rs` drives canonicalization exclusively at compile time via the `sails_build::BuildScript` helper:

```rust
BuildScript::new(SERVICE_PATHS)
    .manifest_path("sails_services.in")
    .meta_dump_features(&["sails-canonical", "sails-meta-dump"])
    .wasm_build(WasmBuildConfig::new("CARGO_FEATURE_WASM_BUILDER", || {
        let _ = sails_rs::build_wasm();
    }))
    .run()
    .expect("generate canonical interface constants");
```

When the `sails-canonical` feature is enabled (default), the `#[service]` macro includes the generated `INTERFACE_ID`, `ENTRY_META`, and canonical JSON constants. When building the host-only `sails_meta_dump` binary or running with `SAILS_CANONICAL_DUMP`, the macros fall back to zeroed stubs so compile times stay minimal; no runtime canonicalization path exists. Diagnostic metadata (`type_bindings`, display names) is emitted in `$OUT_DIR/sails_interface_consts/manifest.json`, keeping the hashed JSON purely structural while still providing human-readable names for tooling.
