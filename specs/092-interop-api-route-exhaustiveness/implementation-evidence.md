# Python implementation evidence

## Release artifact and source

- Python feature branch: `feature/KMIPKIT-0087-interop-python-live`.
- Common contract merge: `9632f116c`, merging contract head `4f157e50e`.
- Private Python package pin: `489fafc79ef89db2524eafa7a6c6482f5cf75d97`.
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
