# KMIPKIT-0055 implementation evidence

## TDD

- RED commit `d295346c6`: `cargo test -p kmipkit-client --lib json_encoding -- --nocapture`
  failed as expected: 1 passed / 5 failed against the inert codec. Valid
  structures, scalar forms, masks, limits and official replay failed with
  InvalidDocument; malformed inputs were already rejected.
- During implementation, the malformed-object nesting regression was observed
  failing with InvalidDocument instead of LimitExceeded before bounded lexical
  recursion was added. The XSD 24:00 regression was observed failing before its
  calendar conversion was corrected.
- GREEN: `cargo test -p kmipkit-client --lib json_encoding`: 12/12 passed,
  including the exact mandatory fixture's four semantic messages and 100
  QuickCheck cases comparing all original raw payloads after roundtrip.
- The mandatory fixture exposed a missing Query Function family mapping in
  0054. The parent supplied reviewed generator fixes (original commits
  d1ca945b3, 61df55166, 80133d08f), cherry-picked without manual generated edits.
- Client source audit initially rejected private type/import aliases. The
  codec now uses explicit types; the original audit passes unchanged.

## Security and ownership

Wire input byte length is checked before scanning. Item count, Structure
array depth, and an independent lexical recursion bound are checked before
descent. Duplicate object keys, trailing data, noninteger numeric syntax,
unknown keys and malformed scalar values are rejected. Payload owners are
created only after the complete bounded lexical pass. Any temporary decoded
string or byte vector is zeroizing, including error exits. Exact payloads move
into the existing model owners. Model boxes/Structure child-vector allocations
retain the existing model's infallible allocation boundary; variable payload
and wire-buffer reservations are fallible.

Encoding counts under the same limits before reserving the exact output
capacity. The second pass writes into that fixed zeroizing allocation without
growth. Scalar numeric formatting temporaries also have zeroizing owners.
Metadata normalization contains only checked-in catalog names. Errors retain
only one of four common categories, with no input/parser-source retention.

The private codec is not dispatched by the client yet; KMIPKIT-0057 owns that
integration. No public API manifest or generated API declarations change.
See traceability.md and the fixture README for normative and provenance detail.

## Refactor and verification

The REFACTOR step separates the bounded writer from parsing and indexes the
checked-in catalog once in an immutable OnceLock. Indexed lookups avoid
scanning/normalizing the full catalog for each received Item. The private
NameKind enum excludes unsupported metadata selector states. The existing
source inventory audit passes unchanged. A final boundary test accepts 64
Structures and rejects 65, and an exact wire-byte cap accepts the complete
emitted document. The pinned fixture's committed bytes retain the official
SHA-256 under its local -text attribute.

Fresh focused coverage after refactor: 780/820 new production lines covered
(95.12%), no exclusions. Files: json_encoding.rs 387/410; names.rs 98/102;
scalars.rs 147/154; writer.rs 139/145; wire_codec_common.rs 9/9.
The artifact `.superpowers/sdd/0055-json/coverage.json` was produced with:
`cargo llvm-cov -p kmipkit-client --lib --json --output-path .superpowers/sdd/0055-json/coverage.json -- json_encoding`.

`cargo test -p kmipkit-client --all-features --quiet` passed the complete
client package (696 passed), including compile-fail boundary tests, integration tests and
doctests; 3 pre-existing tests remain explicitly ignored. Following refactor,
the unit suite passed 622/622, and the final focused suite passed 15/15.
`cargo fmt --all --check`, `cargo clippy -p kmipkit-client --all-targets --all-features -- -D warnings`,
and `git diff --check` pass. The native AWS-LC build emitted its existing
stdalign/NASM probe warnings but completed successfully.

Scope self-review: this branch adds only private JSON code/common errors,
crate-local fixture tests/dev dependencies, and 0055 documentation, plus the
parent-supplied 0054 generator correction. It changes no client dispatch,
manifest, public generated API, transport, or upstream source copy. Full
workspace/cross-language/integration coverage gates belong to the coordinated
0057 verification; focused coverage is not a workspace coverage claim.


The literal normative JSON wire samples embedded in Profiles §5.5.4.1 are now
pinned separately from the XML semantic descriptor. The codec decodes and emits
semantic equivalents of both deterministic requests and the ResponseTooLarge
response, preserving the table's original ProtocolVersion 1.4 and timestamp.
The comparison independently normalizes only permitted Structure/Integer/date
representations. The descriptor's successful Query response remains variable
scenario evidence, without treating historical Template as an assigned 2.1
name. Fixture provenance, extraction utility, hashes and limitations are in the
crate-local fixture README; the upstream HTML remains unchanged.
