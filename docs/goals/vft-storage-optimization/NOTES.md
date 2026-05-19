# NOTES

## Chronological Notes

- 2026-05-19 The approved goal targets storage gas evidence, not dispatcher/routing instrumentation.
- 2026-05-19 Current candidate baseline is the optimized `SailsStatic` VFT path; new variants must beat it, not only `HashMap` or `BTreeMap`.
- 2026-05-19 Existing million-entry layouts already cover more than three candidate ideas: WAT actor, mixed actor, page-local, control actor, grouped pages64, and grouped pages128. They are enough to make a no-winner decision without adding another speculative variant.
- 2026-05-19 `--features gas-profile` changes wasm code shape and must not write production `bench_data.json`; profile runs are evidence artifacts only.
