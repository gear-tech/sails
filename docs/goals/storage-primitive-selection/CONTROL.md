# Storage Primitive Selection Control

Target workload: VFT token transfers on Gear/Sails.

Capacity target: token address tables must be planned for 1-2 million entries.
Small fixed maps are not VFT defaults.

Promotion rule: recommend a primitive only after a consistent benchmark win over
`HashMap` or `BTreeMap` on transfer-critical operations, plus million-capacity
static-map evidence.

Non-superior variants: keep internal, benchmark-only, or experimental.

Listing in default score: no, but list regressions must be documented.

Million-entry loop: do not run on every iteration; reserve it for final evidence
or layout-specific hypotheses.

Pivot approval: required before promoting control-byte, page-local, or grouped
layouts as default public primitives.
