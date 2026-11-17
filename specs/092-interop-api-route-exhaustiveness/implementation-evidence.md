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

## C isolated MAC route

`test-op-mac` uses the installed public C ABI to create and activate an
HMAC-SHA256 key with explicit MAC Generate usage, send an HMAC-SHA256 MAC
request, then revoke and destroy the key. The controlled peer checks the exact
request tree and deterministic 32-byte MAC response fixture. The C consumer
checks the typed response accessor, UID, and 32-byte MAC Data field. The key is
randomly generated by the server, so the fixture does not assert the MAC's
cryptographic validity. OASIS references are KMIP v2.0 §6.1.31 and v2.1
§6.1.32. All 12 version/encoding/transport checks pass; full CTest passes 7/7
with 166 route-matrix contract tests. The other pending operation routes remain
open under FR-003/FR-005.

## C isolated MAC Verify route

`test-op-mac-verify` uses the installed C ABI to create an HMAC-SHA256 key with
explicit MAC Verify usage, activate it, verify fixed data and MAC Data with
explicit HMAC-SHA256 parameters, then revoke and destroy the key. The peer
checks the full ordered request and returns the typed Unique Identifier and
Validity Indicator enum value `Valid`; the consumer checks both typed fields.
The key is randomly generated, so the deterministic fixture does not establish
cryptographic validity. OASIS references are KMIP v2.0 §6.1.32 and v2.1
§6.1.33. Red `b8a55dba9` recorded the missing route; Green `e36920afc` added the
C path and controlled peer. The focused test passes all 12 version/encoding/
transport checks; CTest passes 7/7 with 167 route-matrix contract tests. The
other pending operation routes remain open under FR-003/FR-005.

## C isolated Sign and Signature Verify routes

The authoritative test-catalog generator now models the normative key types:
Create Key Pair with Sign usage on the Private Key for `test-op-sign`, and
Create Key Pair with Verify usage on the Public Key for
`test-op-signature-verify`. The setup activates the selected key and cleanup
revokes/destroys it and destroys its pair. This corrects the prior generic
SymmetricKey recipe based on OASIS KMIP v2.0/v2.1 §12.1 Table 61, which says Sign
is valid for PGP Key or Private Key and Verify for PGP Key, Certificate, or
Public Key. The generated cases include §12.1 alongside their operation
sections.

The installed C client executes ordered Create Key Pair/Activate/Sign or
Signature Verify/Revoke/Destroy/Destroy batches with ID selectors for the
respective pair members. Both keys use RSA-2048; the Private Key has Sign usage
and the Public Key has Verify usage. The peer checks the exact request fields,
result identifiers, typed 256-byte signature or `Valid` response, item IDs, and
state cleanup. Since the key pair is random and the verify signature input is
not derived from it, these fixtures verify request/response structure and API
behavior only, not cryptographic correctness. Red `e80371257`, `2782a91dc`, and
`5a855d802` demonstrated the misassigned fixture and missing C routes; Green
`bd6d9b0c8` and `389ac4465` corrected the generated recipe and implemented both
routes. Each focused route passed 12/12 combinations; catalog checks passed;
CTest passed 7/7. Other route cells remain pending under FR-003/FR-005.

## C direct Create Key Pair route

`test-op-create-key-pair` exercises OASIS KMIP v2.0/v2.1 §6.1.9 through the
installed C client. The catalog generator now specifies cleanup of both
objects returned by Create Key Pair. The C consumer creates an RSA-2048 pair
with explicit Sign and Verify usage masks, then destroys the Private Key and
Public Key by their Create Key Pair identifier selectors. It validates the
typed pair response, distinct identifiers, each successful result, and the
identifier returned by both Destroy operations. The controlled peer validates
the complete request fields and response mapping, including the unique item IDs
and final state. This is protocol-shape and lifecycle evidence; generated key
material is not inspected or cryptographically validated.

Red `36034c7a6` added the failing catalog and route requirements. Green
`89bd02b79` updated the generator and generated catalog, implemented the C
route, and added the controlled-peer request/response checks. The focused route
passed 12/12 KMIP-version, encoding, and transport combinations; Sign and
Signature Verify also passed 12/12 each. The normative inventory passed
26/26, catalog `--check` passed, and CTest passed 7/7 in 96.00 seconds,
including both controlled TLS peers. These results add one C operation route;
other C route cells and cross-language FR-003/FR-005 coverage remain pending.

## C Certify Public Key route

The installed C consumer now exercises OASIS KMIP v2.0/v2.1 §6.1.6 Certify
with the paired Public Key as target. Its generated catalog recipe creates an
RSA-2048 pair with Certificate Sign usage on the Private Key, activates that
signer, requests an X.509 certificate, destroys the certificate using ID
Placeholder, then revokes and destroys the pair. The X.509 Certificate Type is
encoded directly in the request Attributes structure. OASIS §12.1 Table 495
places Certificate Sign usage on a Private Key.

Red `109ff223e` required the route and `b86d6d708` added the signer lifecycle
expectation. Green `eabd01402` implemented the generator-backed recipe, typed C
request/results, and controlled-peer request/response and lifecycle checks. The
route passed all 12 version/encoding/transport cells, both controlled TLS peer
tests passed, the normative inventory passed 27/27, catalog `--check` passed,
and CTest passed 7/7 in 97.46 seconds. This is protocol-shape and lifecycle
evidence; the fixture does not validate certificate or signature cryptography.
Other C routes and cross-language FR-003/FR-005 coverage remain pending.
