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


## C Export request and response payload variants

- Normative scope: OASIS KMIP v2.0 §6.1.18 Tables 202–203 and v2.1
  §6.1.18 Tables 217–218. This slice exercises all ten catalogued Export
  request-field cases and the four required successful response fields:
  Object Type, Unique Identifier, Attributes, and Any Object.
- Installed artifact: `libkmipkit_ffi.so` SHA-256
  `c93eb8f7daa60d3b6832d297887e99cd2101e4f9baf2d04f55ce4042ba4bb7a2`;
  catalog SHA-256
  `08c379fbad85d218a46edea6e0dab44ddd78b6c352ac21dc869b51793a53d6e9`;
  route manifest SHA-256
  `3fe892f7b2804e8c1b03f9dc01c8cc71038829f7ed5aaf38d546fea90ddc980a`.
- Request Red — `4d4174e79`: the focused test observed zero checks because the
  ten request-field cases had no controlled-peer recipes. Green — `367006d57`:
  the installed C consumer constructs each request field with valid catalogued
  values and exercises present and omitted variants over KMIP 2.0/2.1, TTLV,
  JSON, and XML, raw TLS and HTTPS. The focused test passes 120/120 peer checks.
- Response Red — `42f8238e4`: the focused test observed zero checks because the
  eight field/version cases had no controlled-peer recipes. Green — `488fdf799`:
  the fixture verifies each complete response and then removes exactly one
  required field for the negative variant. The installed consumer rejects the
  response after receiving it (`category=2 cause=2 delivery=3`). The focused
  test passes 96/96 peer checks: 48 complete responses and 48 expected
  rejections across all six encoding/transport combinations.
- Generic TTLV Red — `8a69dc91b`: the four versioned local cases had no
  generic fixtures. Green — `208380cb0`: derive Key Wrap Type from its catalogued
  enumeration and represent Any Object with a catalogued Symmetric Key
  structure. The installed ABI roundtrip test passes for all four cases and
  exercises both `kmipkit_ttlv_item_create` and `kmipkit_ttlv_decode`.
- Refactor/documentation commit records the per-cell scope and immutable
  artifact identities. The full route matrix after this Green reports 66,736
  expected, 41,683 invoked, 0 failed, and 25,053 pending cells (run ID
  `c-route-19dc4d35-1e4d-44d0-8d38-c7573e1226fc`). It contains no pending Export
  cells; the other 25,053 cells remain outside these Export slices.
  Verification: both payload peer tests (120 request and 96 response checks),
  the generic TTLV roundtrip test, and full CTest passed. CTest passed 7/7 in
  90.64 seconds, including 162 route-matrix contract tests; `python3 -m
  py_compile` and `git diff --check` passed. These results do not close the full C route matrix
  or KMIPKIT-0092-FR-003/FR-009.

## C isolated Encrypt route

For KMIPKIT-0092-FR-003, FR-005, FR-008, and FR-009, `test-op-encrypt` now
constructs a typed Create/Activate/Encrypt/Revoke/Destroy batch using the
installed public C ABI. The peer checks AES-128 creation attributes and
Encrypt-only usage, ID Placeholder references, explicitly supplied AES/ECB/No
Padding parameters, exact fixture input/response Data bytes, response item IDs
and typed response fields, then verifies the object is revoked and deleted.
The consumer checks a 16-byte Data field, not an exact ciphertext: the remote
Create uses a random key and the fixture vector is not cryptographically tied
to it. The case is
executed for KMIP 2.0/2.1, TTLV/JSON/XML, and raw TLS/HTTPS. The focused test
passed all 12 peer exchanges. CTest passed 7/7, including 164 route-matrix
contract tests and both controlled TLS peers. This adds one complete positive
operation slice; it does not satisfy the remaining pending routes or the full
FR-003/FR-005 matrix.

## C isolated Decrypt route

`test-op-decrypt` now uses the installed C ABI for a typed
Create/Activate/Decrypt/Revoke/Destroy batch. The controlled peer checks the
Decrypt usage mask, ID Placeholder target, AES/ECB/No Padding parameters,
exact fixture ciphertext input and response Data bytes, response IDs, and final
object deletion. The consumer checks a typed 16-byte plaintext field. Since
Create generates a random key, the deterministic fixture does not assert
cryptographic correctness. OASIS references are KMIP v2.0/v2.1 §6.1.11, §6.1.1, §6.1.40/§6.1.44,
§6.1.15, §6.1, and §§8.2–8.4, 8.6. Encrypt and Decrypt focused tests passed
12/12 checks each. CTest passed 7/7 with 165 route-matrix contract tests and
both controlled TLS peers. Remaining C and cross-language matrix routes stay
pending under FR-003/FR-005.
