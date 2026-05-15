# Sails IDL Embedding

Sails programs may embed their IDL text in a WebAssembly custom section named `sails:idl`.
This lets off-chain tooling recover the interface from program code without a separate registry lookup.

## Custom Section

The custom section payload is an envelope followed by IDL bytes:

| Offset | Field | Description |
|---:|---|---|
| 0 | `version` | Envelope version. Current value: `0x01`. |
| 1 | `flags` | Bit flags. Current bit 0 means deflate-compressed payload. Bits 1-7 are reserved. |
| 2.. | `content` | Raw UTF-8 IDL text or raw-deflate-compressed UTF-8 IDL text. |

Unknown envelope versions and unknown flag bits must be treated as "no usable IDL" rather than as hard decode failures.
Malformed WASM, corrupt deflate streams, oversized decoded output, and invalid UTF-8 are implementation errors and should be surfaced as typed failures.

## Size Limit

Decoded IDL content is capped at 1 MiB. Implementations that stream decompression must stop once the decoded byte count exceeds this limit.

## Extraction Rules

- Validate the WASM magic and version before walking sections.
- Walk custom sections without compiling or validating function bodies.
- The first `sails:idl` custom section wins.
- Decode UTF-8 with fatal error handling.
- If the compressed flag is set, use raw deflate.

These rules mirror `rs/idl-embed` and should be followed by new language ports.
