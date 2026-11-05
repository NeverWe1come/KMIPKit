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
