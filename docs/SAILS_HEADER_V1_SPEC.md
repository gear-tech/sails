# Sails Header v1 Specification (Draft)

## Abstract

Sails Header defines a deterministic, userspace envelope for Sails-based Gear/Vara asynchronous messages. Every Sails message begins with a 16-byte base header (extensions optional) that encodes a magic prefix, version, payload offset, and three routing identifiers: a 64-bit `interface_id`, a 16-bit `entry_id`, and an 8-bit `route_idx`. The header lives entirely within the message payload, requiring no runtime or consensus changes, and enables off-chain tooling and cross-program interoperability by exposing canonical interface metadata.

## Goals

- **Interoperability.** Provide a uniform wire format so any program or tool can interpret interface- and entry-level information from the header alone.
- **Off-chain decodability.** Allow explorers, debuggers, and SDKs to decode Sails payloads without executing WASM by relying on the header’s identifiers.
- **Deterministic identifiers.** Tie `interface_id` and `entry_id` to the canonical interface definition so identical interfaces across languages receive identical IDs.
- **Compile-time embedding.** Make the identifiers available at compile time (e.g., via macros) so a program can embed routing data directly in the binary.
- **Backward compatibility.** Maintain the existing Gear message format; Sails Header occupies payload bytes only.

### Non-Goals

- Altering node runtime, consensus, or native message layout.
- Resurrecting deprecated metadata formats.

## Terminology

| Term                | Meaning                                                                                                            |
| ------------------- | ------------------------------------------------------------------------------------------------------------------ |
| Program             | A deployed Sails program (Gear/Vara WASM module) with one or more interfaces.                                      |
| Service / Interface | A logical API defined in IDL, consisting of functions and events.                                                  |
| Route               | A named service instance. Programs may expose multiple instances per interface.                                    |
| `interface_id`      | 64-bit identifier derived from the interface hashing spec (service name excluded).                                 |
| `entry_id`          | 16-bit identifier for an interface entry (function/event), assigned deterministically.                             |
| `route_idx`         | One-byte route index. `0x00` denotes the default instance. Non-zero indices are mapped via a per-program manifest. |

## Header Layout

```
Byte offset  Field          Size (bytes)  Description
0–1          Magic          2             ASCII "GM" (0x47 0x4D)
2            Version        1             Header version (0x01)
3            Header length  1             Total header size in bytes; base header = 0x10
4–11         Interface ID   8             64-bit ID from Interface hash
12–13        Entry ID       2             Little-endian 16-bit entry identifier
14           Route Index    1             Unsigned byte; 0 = default route
15           Reserved       1             Must be set to 0x00 in v1 (no error flags). Future specs may repurpose this byte.
>15          Extensions     variable      Present only if `header length` > 0x10
```

### Semantics

- **Magic.** Indicates presence of Sails Header. Readers should verify `0x47 0x4D` before interpreting the remaining bytes.
- **Version.** Drives compatibility. Version 1 defines the format in this document; future versions may extend the header.
- **Header length.** Allows optional extensions to follow the base fields. Payload data starts at offset `header length`.
- **Interface ID.** Deterministic identifier computed via the hashing spec (see below). The ID never depends on the textual service name.
- **Entry ID.** Deterministically assigned `u16` per interface by sorting entry names lexicographically (ties resolved by canonical signature). Implementations may freeze the assignment in lockfiles to preserve wire compatibility.
- **Route Index.** Identifies which service instance is targeted; mapping from integer to route name is program-specific.
- **Reserved byte.** Always zero in v1 (no error flag semantics). Future revisions may reinterpret this byte. Receivers MUST treat non-zero values as invalid for v1 headers.
- **Extensions.** Optional, structured as a TLV stream (see below). `header length` MUST cover the base header plus all extension bytes.

## Identifier Derivation

### Interface ID

See [REFLECT_HASH_SPEC](REFLECT_HASH_SPEC.md)

### Entry ID

**Commands and queries**

1. Collect all commands and queries defined in the interface.
2. Sort by name (lexicographically).
3. Assign `entry_id` sequentially starting at zero. Implementations MAY pin these assignments externally (e.g., lockfile) if backwards compatibility is critical.

**Events**

1. Collect all events defined in the interface.
2. Sort by name (lexicographically).
3. Assign `entry_id` sequentially starting at zero. Implementations MAY pin these assignments externally (e.g., lockfile) if backwards compatibility is critical.

### Route Index

`route_idx` values are assigned by the program author. `0x00` is the default instance. Additional instances are mapped via a manifest or registry that downstream tooling can inspect.

### Extension Framing (TBD)

Extensions appear immediately after the base header (offset 16) and continue until `header length` bytes have been consumed. Each extension record uses a Tag-Length-Value format:

```
struct Extension {
  type_id: u8;    // 0 reserved
  flags:   u8;    // extension-specific flags/version (0 if unused)
  length:  u16;   // little-endian payload length in bytes
  data:    [u8; length];
}
```

Parsing rules:

1. Continue reading extensions until `header length` bytes are consumed. If no extension bytes are present, `header length` equals 0x10.
2. Each extension MUST fit entirely within `header length`; otherwise the header is invalid.
3. Unknown `type_id`s MUST be skipped using the declared `length`, ensuring forward compatibility.
4. `type_id = 0` is reserved and MUST NOT appear on the wire.
5. Implementations MAY standardize specific `type_id`s (e.g., correlation IDs) and document their payload structure separately.

## Usage Requirements

- Programs SHOULD compute `interface_id` and `entry_id` at compile time (e.g., using macros or IDL generators) and embed them as constants. This ensures routing does not depend on runtime hashing.
- When sending a message:
  1. Fill the 16-byte base header.
  2. Append any extensions (optional).
  3. Append the SCALE-encoded payload immediately after `header length`.
- Receivers MUST examine the magic + version to interpret the header. They MAY reject messages with unknown versions or with a header shorter than `0x10`.
- Off-chain tools (explorers, RPC gateways) can read the same header to classify messages without executing WASM.

## Determinism Checklist

Implementations MUST ensure:

1. Service names never influence `interface_id`.
2. Ordering of extends/functions/events/types is stable (sort rules above).
3. `entry_id` assignment methodology is documented; ideally frozen via manifest for upgraded interfaces.

## Validation Checklist

A receiver that detects a potential header SHOULD apply at least the following checks:

1. **Magic:** The first two bytes are `0x47 0x4D`; otherwise treat the payload as legacy/unheadered.
2. **Version:** `version == 0x01`. Unknown versions may be rejected or parsed according to future specs.
3. **Header length:** `hlen >= 0x10` and `hlen <= payload_length`. Reject if the length is smaller than the base header or extends beyond the payload.
4. **Reserved byte:** For v1 the byte at offset 15 MUST be zero. Non-zero values indicate incompatible behavior unless a future version redefines it.
5. **Extensions:** If `hlen > 0x10`, ensure each TLV record fits within the declared header length; malformed TLVs invalidate the header.

Once the header passes validation, proceed to decode routing identifiers and payload.

## Security Considerations

- Headers with invalid magic, unsupported versions, non-zero reserved byte (in v1), or header lengths that extend beyond the payload MUST be rejected before processing.
- Extension parsing must respect `length` fields strictly; implementations SHOULD cap accepted header lengths to prevent resource exhaustion.
- Tooling should treat `interface_id`, `entry_id`, and `route_idx` as untrusted inputs - only use them after successful validation and cross-checking against known manifests.

## Examples

### Example A – Default Route Call

Assume `interface_id = 0xA1B2C3D4E5F60718` and `entry_id = 0x0200` (`Foo::Bar`), default route (`route_idx = 0x00`), base header only.

```
47 4D  01   0B      18 07 F6 E5 D4 C3 B2 A1  02 00     00     00
^magic ^ver ^hlen  ^interface_id LE          ^entry_id ^route ^reserved
```

Payload bytes follow immediately after byte offset 11.

### Example B – Alternate Route (reserved byte zero)

`interface_id = 0x0123456789ABCDEF`, `entry_id = 0x0500`, `route_idx = 0x02`, reserved byte remains zero.

```
47 4D 01 0B  EF CD AB 89 67 45 23 01  05 00  02  00
```

## Frequently Asked Questions

**Q: How do I map `route_idx` back to route names?**  
Provide a manifest that associates each route name with a 1-byte index. Tooling can distribute this manifest alongside the program binary or IDL.

**Q: Can I include additional metadata in the header?**  
Yes. Increase `header length` and append extension fields. Receivers that understand the extension can parse it; others should skip to `header length` before reading the payload.

**Q: How do off-chain tools verify the header?**  
Check the magic/version, read `interface_id` and `entry_id`, and consult a registry or IDL manifest to interpret the payload.

## Normative References

1. RFC 8785 - JavaScript Object Notation (JSON) Canonicalization Scheme.
2. BLAKE3 cryptographic hash function specification.
3. RFC 7914-style domain separation guidelines (domain-specific hashing).
4. SCALE Codec Specification – defines the serialization format used for the payload that follows the header.
5. Gear Protocol Documentation – describes the underlying asynchronous messaging context in which Sails Header operates.

---

This document is intentionally implementation-neutral. Any language or framework can adopt the Sails Header provided it implements the canonical hashing rules and the binary layout defined above.\*\*\*
