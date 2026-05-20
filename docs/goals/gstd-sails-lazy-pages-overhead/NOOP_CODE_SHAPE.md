# Noop Code-Shape Pass

## Question

Is the remaining `noop_sails` floor inherent to Rust/gstd/Sails Header v1, or
is it mostly generated Sails framework shape?

## New Benchmark Row

Added `noop-sails-hot`, a benchmark-only Rust program that:

- reads the Sails Header v1 payload directly with `gcore::msg::with_read_on_stack_or_heap`
- checks the known noop interface id, route id, and entry id
- replies with the same 16-byte Sails header plus SCALE `bool`
- does not use the Sails program/service macros

It is not a production API. It is a Rust lower bound for generated Sails hot
dispatch.

## Gas Result

Artifact:
`benchmarks/target/gstd-sails-lazy-pages-overhead/noop-rust-hot/`

| Row | Median gas | Delta vs `noop_gstd` |
| --- | ---: | ---: |
| `noop_wat_sails_wire` | 437,134,646 | -7,731,014 |
| `noop_wat_raw` | 441,045,976 | -3,819,684 |
| `noop_gstd` | 444,865,660 | 0 |
| `noop_sails_hot` | 475,701,813 | 30,836,153 |
| `noop_sails` | 587,525,915 | 142,660,255 |

`noop_sails_hot` closes about 78% of the generated `noop_sails` gap versus
`noop_gstd`: `(142,660,255 - 30,836,153) / 142,660,255 = 78.4%`.

## Section-Size Result

| Program | Bytes | Functions | Code | Data | Exports |
| --- | ---: | ---: | ---: | ---: | ---: |
| `noop_gstd` | 737 | 4 | 590 | 0 | 3 |
| `noop_sails_hot` | 5,145 | 16 | 4,843 | 38 | 3 |
| `noop_sails` | 26,620 | 80 | 21,564 | 4,649 | 5 |

The generated row has 64 more functions than the Rust hot lower bound and
about 4.6KB of data dominated by panic/format strings. It also exports
`handle_reply` and `handle_signal`, while `noop_sails_hot` only exports
`handle`, `init`, and `__gear_stack_end`.

## Interpretation

The remaining noop gap is not caused by Sails Header v1 itself. A direct Rust
Sails-wire implementation still uses the same header/reply shape and lands much
closer to `noop_gstd` than generated `noop_sails`.

The next production candidate should be an opt-in generated hot dispatch path
for simple sync services. The target is not WAT; it is the Rust hot lower bound:
roughly 475M gas for noop before any further precharge reduction.

## Next Candidate

Prototype generated hot dispatch for services that meet all of these limits:

- sync methods only
- no base services
- no `reply_with_value`
- no `throws`
- simple SCALE params and replies
- Sails Header v1 preserved

The generated path should:

- avoid generic service exposure dispatch for eligible methods
- avoid panic/format-heavy unknown-input paths in the hot match arm
- encode replies directly for `bool`, `U256`, and empty replies
- keep the normal generated path for ineligible methods

Acceptance target: reduce `noop_sails - noop_gstd` by at least 25% while
preserving the VFT stack-read win.
