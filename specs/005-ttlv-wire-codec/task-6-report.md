# T006 Green Decoder Report

## Scope

T006 promotes the T005 private Red adapter to the production
`kmipkit_ttlv::codec::decode` API. It decodes exactly one complete item,
preserves represented values and ordered Structure children, applies assigned,
extension, Reserved, and unallocated Tag dispositions, enforces the default
16 MiB / 64 Structure / 100,000 Item limits, and returns payload-free errors
with safe offsets. Decoder-owned payload copies use fallible reservations
after type, available-input, parent-boundary, and default-limit checks.

The implementation is in DCO-signed Green commit
`6419a4304a05e3607e0c088d9d64eb9080f04817`; documentation and traceability
corrections are in DCO-signed follow-up
`71534fda7f74a14e1ecdb12187ed26b9ce51e54b`. The follow-up does not change
parser behavior. `CodecLimits` and configurable decoding remain T010 scope;
decoder helper refactoring remains T007 scope. No client/protocol API, unsafe
code, raw input retention, OASIS source edit, or generated-file edit was added.

## Test coverage

- The 33 private behavioral cases promoted from T005 exercise all eleven Item
  Types, fixed-width boundaries, bit preservation, Structure order and
  boundaries, Tag policy, and malformed framing through the production
  decoder.
- `tests/codec_api.rs` proves an external crate can import and call `decode`.
- Sixteen `tests/codec_negative.rs` cases cover malformed headers/values,
  unsupported types, unallocated and Reserved Tags, padding extents, payload
  error redaction, and default byte/depth/count limits.
- Existing model, API, tag allocation, zeroization, and doctests remain in the
  crate test run.

## Verification

The root agent independently ran these commands on Windows after the follow-up;
all passed:

- `cargo fmt --all --check`
- `cargo clippy -p kmipkit-ttlv --all-targets --all-features -- -D warnings`
- `cargo check -p kmipkit-ttlv --all-features`
- `cargo test -p kmipkit-ttlv` — 82 tests passed (33 private decoder, 1 public
  API, 16 negative, and existing unit/integration/trybuild/doctests).
- `cargo doc -p kmipkit-ttlv --no-deps`
- `git diff --check`

## Independent review

The independent re-review covered Green commit `6419a43` and follow-up
`71534fd`, and returned **PASS**. It verified that crate rustdoc describes the
bounded public decoder, UTF-8 and unsupported-Item-Type tests distinguish OASIS
`NR-002` from project `FR-005`, and error wording describes default rather than
configurable limits. The reviewer performed static review only and did not run
builds or tests. The parser review found no blocking correctness/security
issue or regression in the follow-up.

The complete review record is [`task-6-review.md`](task-6-review.md).

## Known limitation

Existing generic model constructors use `Box::new` and `Vec::push`; allocation
failure in those existing constructors may abort. T006 adds fallible decoder-
owned payload reservations but does not change or claim recovery from those
model allocations.
