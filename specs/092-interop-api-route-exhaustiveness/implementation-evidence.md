# Python implementation evidence

## Release artifact and source

- Python feature branch: `feature/KMIPKIT-0087-interop-python-live`.
- Common contract merge: `9632f116c`, merging contract head `4f157e50e`.
- Historical private Python package pin for this recorded run: `489fafc79ef89db2524eafa7a6c6482f5cf75d97`.
- Wheel SHA-256: `399a9de433a460bd95237e9142c6b19ca082bdc2a94c2d31fe93765cf508a369`.
- Shared catalog SHA-256: `e4a542bab4831a04e51f5bf04c9019da3f38845bc85839ae9b632dcc610a7d7f`.
- Shared route manifest SHA-256: `b3867835e3f1f69308a3030f83be87e2728af852da3a82414a78a4ea91482ac5`.

The wheel was built from the immutable release source archive, installed into
the Python 3.12 consumer environment, and verified against the pin and wheel
digest with `tools/verify_artifact.py`. The matrix did not import KMIPKit from
the source checkout.

## Red, Green, Refactor

1. **Red — `01c75c1da`** changed the controlled Batch Stop assertions to require
   successful decoding of the single returned failure response. With the
   previously installed wheel from `9a16cc90a12205101d22a38501a5e74c665df4dc`,
   the focused tests failed in seven assertions: one direct route and six
   TTLV/JSON/XML × raw TLS/HTTPS cells. Each failure recorded the library's
   rejection of that partial response.
2. **Green — `75ad6b7a0`** updated only the Python validator's private pin,
   rebuilt and installed the wheel from `489fafc`, and passed the focused
   tests. The complete Python consumer suite passed **132 tests** in 455.010
   seconds. The route matrix then verified **74,648/74,648** required cells:
   45,470 local cells and 29,178 controlled-peer network cells, with zero
   failures and zero pending coverage. Run ID:
   `python-matrix-7f813d39-2e6e-41fb-a92a-2c043a91f221`.
3. **Refactor — `2bb232d0f`** factored the common Batch Stop response evidence
   assertions into one helper. The two focused tests passed again across the
   direct route and six encoding/transport combinations.

The independent shared validator accepted the matrix with
`all_invocations_passed: true`, `complete_matrix: true`, 74,648 expected and
observed cells, zero duplicate events, and matching package, catalog, and
route-manifest identities. `closure.json` reports zero failed and zero
pending cells.

## Normative and result limits

The Batch Stop case cites OASIS KMIP 2.0 §11.5, Table 396, and KMIP 2.1 §11.5,
Table 434. The controlled peer returns one `Operation Failed` response after
the first operation and records the later operation as not executed. The
installed release artifact accepts the returned partial response under
KMIPKIT-0120. The test establishes client behavior against a controlled
fixture; it does not establish the behavior or compatibility of a vendor
server.

No Cosmian KMS, CipherTrust, or ESKM endpoint was available. Cross-language
parity is also pending the joint matrix using identical release artifacts and
fixtures. Those results are not inferred from this Python-only run.

## Follow-up: Stop omitted typed response accessors

The Batch Stop route matrix now treats the second `GetResponse` accessor as
not applicable when the controlled server correctly omits the unprocessed
second response. It records no invocation for that accessor and retains the
explicit `stop_unprocessed_response_omitted` reason. All returned response
items still require successful accessor execution.

Red `480809b27` adds a focused check for the omitted typed accessor; the
existing broad accessor test failed because it required those two versioned
cells to be invoked. Green `629cb19ad` requires exactly those two cells to be
not applicable and verifies zero invocation count and no invocation ID. The
two focused tests pass, and the complete Python consumer suite passes 135
tests in 278.391 seconds with the pinned wheel installed in an isolated
Python 3.12 environment (artifact SHA-256
`18043fee5e92893bf4d3ac495552121ac8edc7cbc7f6e77472143e2b6ccb7d78`).

References: OASIS KMIP 2.0 §11.5, Table 396; KMIP 2.1 §11.5, Table 434; and
KMIPKIT-0092-FR-007. This test adjustment records the omitted response without
claiming that an accessor ran for an item the server did not return.

## Follow-up: strict peer-state evidence

- **Red — `76cdb53f7`** adds assertions that Python Stop and Continue events
  include the before/after state hashes required by the shared evidence schema.
  The focused tests failed because those fields were absent. Exact-head CI for
  `10b66ad65` exposed the same schema failure at a Batch Continue cell.
- **Green — `eb93d6de5`** records the SHA-256 of the canonical controlled
  fixture object snapshot on both sides of each catalogued read-only batch
  exchange and includes the hashes in emitted events. The hashes are equal
  because these two scenarios do not mutate fixture state; ordered item
  effects remain recorded separately.
- **Refactor** documents the evidence boundary in the validator README. The
  full Python consumer suite passed all 132 tests in 361.981 seconds. The
  expanded catalog contains 74,656 Python route cells: 45,478 local and
  29,178 controlled-peer network cells. The Python 3.12.13 local matrix and
  independent validator passed all 74,656 cells with zero failures, pending
  cells or duplicate events. Run ID:
  `python-matrix-6cd71081-28dc-4fc5-a3ae-68298632eaf7`. The report used release
  commit `489fafc79ef89db2524eafa7a6c6482f5cf75d97`, artifact SHA-256
  `d07a7e45d6bf9ae576e3fcc60df788722e3f2446f75ca07a26f505c5f8049748`, catalog
  SHA-256 `55fcd64afba189ab7d04978fafb841fc73c9e5afc1ef25c93cb2eb04f4fba4a9`,
  and route-manifest SHA-256
  `3fe892f7b2804e8c1b03f9dc01c8cc71038829f7ed5aaf38d546fea90ddc980a`.

## Follow-up: Java Certificate Attributes generic TTLV view

- **Red — `b8d4ed715`** requires the public `TtlvItem.view()` route for the
  Certificate Attributes case in KMIP 2.0 and 2.1. The controlled-peer test
  failed because neither version had an invocation recorded.
- **Green — `4d14fd7fd`** builds the `Attribute` structure using the
  Certificate Subject CN Text String fixture, invokes `TtlvItem.view()` on
  the constructed and decoded items, and verifies their nested tags, types,
  and values. The focused Java controlled-peer route test passes both version
  cells and updates the event-count assertion to include them.
- **Refactor** records the fixture's source: OASIS KMIP 2.1 §4.6, Table 39
  defines Certificate Subject CN as Text String; §4.60 defines the Attribute
  structure and §11.56 defines its TTLV tags. The controlled-peer test passed
  in 2m15s. This adds local generic-TTLV evidence only; it does not claim that
  every Certificate Attributes policy branch or the complete Java matrix is
  covered.
- The shared evidence validator confirms 44,242/71,583 Java cells in this
  report: 44,226 invoked, 16 not applicable, zero failed, and 27,341 pending.
  The selected scope is complete; global matrix validation remains incomplete.
  Report run ID: `java-controlled-query-cd6c8317-2b8d-4df2-b3af-3153117e617c`.

## Follow-up: Java Cryptographic Parameters algorithm setters

- **Red — `3cc4f7c0f`** requires construction evidence for
  `CryptographicParameters.Builder.cryptographicAlgorithm` and
  `.digitalSignatureAlgorithm` under the corresponding Attribute cases for
  KMIP 2.0 and 2.1. The controlled-peer test failed on the missing
  Cryptographic Algorithm builder event.
- **Green — `406a3016b`** selects source-catalogued Enumeration values, invokes
  each public builder setter, builds the typed model, and verifies that its
  TTLV structure retains the selected tag and raw value. All four
  version-specific route assertions pass.
- **Refactor** documents that the Attribute cases are sourced from OASIS KMIP
  2.0/2.1 §§4.13 and 4.22, while the corresponding typed structure fields are
  defined in §4.16, Table 59. The focused Java controlled-peer test passed in
  2m23s. Selected report validation is 44,246/71,583 cells: 44,230 invoked,
  16 not applicable, zero failed, and 27,337 pending. Run ID:
  `java-controlled-query-b952d296-6108-40d7-afbd-f8284fcb5004`. The global
  Java matrix remains incomplete.

## Follow-up: Java Cryptographic Domain Parameters typed wrappers

- **Red — `0faf2b37a`** requires `TypedAttribute.create`,
  `TypedAttribute.fromItem`, and `AttributeSet.tryNew` evidence for
  Cryptographic Domain Parameters in KMIP 2.0 and 2.1. The controlled-peer
  test failed on the first missing typed wrapper.
- **Green — `2a1bdef9e`** constructs a source-shaped Structure containing
  Qlength as Integer and Recommended Curve as Enumeration, then verifies both
  fields through the installed typed attribute and AttributeSet APIs. All six
  versioned wrapper assertions pass.
- **Refactor** records OASIS KMIP 2.0/2.1 §4.14, Table 55 as the source for
  the structure and encodings. The focused Java controlled-peer test passed
  in 2m22s. Selected report validation is 44,252/71,583 cells: 44,236 invoked,
  16 not applicable, zero failed, and 27,331 pending. Run ID:
  `java-controlled-query-0c0feceb-966b-454d-a88b-5429617d1941`. The full Java
  matrix remains incomplete.
