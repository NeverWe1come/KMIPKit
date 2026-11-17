# KMIPKIT-0092 Traceability

| Requirement | Implementation target | Verification |
|---|---|---|
| KMIPKIT-0092-FR-001 | `interop/shared/coverage/route-capabilities.json`, route capability schema, `route_models.py`, and `coverage_matrix.py` | Source-backed route inventory binds typed attribute markers to exact elements, maps Register/Revoke request and batch-item constructors for every language, covers the Batch Error Continuation Option builder across four languages (§§8.2, 9.6, 11.5), limits batch result readers to KMIP 2.0/2.1 response Batch Count and Batch Item fields (§§8.5–8.6, 9.5, 9.7), keeps §11.34 Object Type Enumeration targets separate from §4.36 Object Type Attribute routes, prevents attribute wrappers from claiming managed-object structures, scopes Secret Data constructor/parser routes to their own object tree and excludes Cryptographic Algorithm and Cryptographic Length as required by KMIP 2.0/2.1 §3.1 Table 11, scopes Key Wrapping Data setters to their exact §3.3 Tables 14–17 member and prevents Encryption Key Information and MAC/Signature Key Information Cryptographic Parameters routes from crossing sibling structures, limits Rust `KeyMaterial::byte_string` to the generic Key Material tag because KMIP 2.0/2.1 §11.25 distinguishes ByteString and Structure formats, excludes checked-`Tag` constructors and Item-view accessors only when reserved/unused allocations cannot yield an Item, retains RawTag construction and decoder-rejection rows, and binds async operations to exact public executors; installed artifact probes for all four languages |
| KMIPKIT-0092-FR-002 | `coverage_matrix.py`, `validate_route_evidence.py`, and language CI follow-ups | Negative tests reject missing target/version/route/action cells, duplicate or unplanned evidence, and incorrect scenario axes; expected NotSent cases require zero transport calls and omit impossible response decoding; async request cells use per-operation senders; actual full language CI is pending |
| KMIPKIT-0092-FR-003 | Per-language recipe and local TTLV suites; C `route_matrix.py` and `test_route_matrix.py`; Rust language recipe evidence | C installed-ABI coverage parses complete Request Messages (root, structures, Protocol Version, Batch Count, Operation, Request Payload, Unique Batch Item ID, Maximum Response Size, optional Header fields, and batch options) and complete Response Messages (root, structures, optional Header and Batch Item fields including a valid Failure/Result Reason/Result Message shape) for KMIP 2.0/2.1. Exact C installed-consumer run `43571020018` at head `52561206` records 41,051/66,736 invoked cells, zero failed, and 25,685 pending, all `validator_recipe_pending`. Five catalogued Cancel Cancellation Result values, Mandatory asynchronous Create, omitted-indicator synchronous Create, Optional-indicator synchronous Create, explicit Prohibited-indicator synchronous Create, the KMIP 2.1 Create/Query/Poll/Destroy lifecycle, the KMIP 2.1 Create/Process/Poll/Destroy route, and the KMIP 2.1 Query filter intersection route execute in applicable cells. The local continuation after Green commit `26d5f047` adds direct and standalone C Adjust Attribute checks across KMIP 2.0/2.1 and all six encoding/transport pairs; its fixture validates the selected identifier, vendor attribute reference, Increment type, and value 5. Full local CTest passed 7/7, including both controlled TLS peers. Exact-head CI and the complete installed route matrix remain open. The C continued-batch conformance test now requires both direct and standalone Get Attribute List constructors to pass and verifies unchanged peer state on each send; the focused test and `c_validator_route_matrix_contracts` pass at commit `d76df104`. The C Adjust Attribute test at commit `26d5f047` also passes direct and standalone request constructors across KMIP 2.0/2.1 and all six encoding/transport pairs; its exact-head matrix run is pending. Other async branches, response-accessor routes, and the broader positive-operation matrix remain pending. The local candidate run `c-route-c4e86961-430d-49a4-ab47-5aed58a24602`, using the pinned release artifact `489fafc79ef89db2524eafa7a6c6482f5cf75d97` with digest `c93eb8f7daa60d3b6832d297887e99cd2101e4f9baf2d04f55ce4042ba4bb7a2`, records 41,730 of 66,736 cells invoked, zero failed, and 25,006 pending; the ten Delegated Login request/response field cases are covered. This does not establish exact-head CI completion. Full applicability remains open across all languages, values, batches, async cases, and boundaries. C installed-ABI coverage parses complete Request Messages (root, structures, Protocol Version, Batch Count, Operation, Request Payload, Unique Batch Item ID, Maximum Response Size, optional Header fields, and batch options) and complete Response Messages (root, structures, optional Header and Batch Item fields including a valid Failure/Result Reason/Result Message shape) for KMIP 2.0/2.1. Exact C installed-consumer run `43571020018` at head `52561206` records 41,051/66,736 invoked cells, zero failed, and 25,685 pending, all `validator_recipe_pending`. Five catalogued Cancel Cancellation Result values, Mandatory asynchronous Create, omitted-indicator synchronous Create, Optional-indicator synchronous Create, explicit Prohibited-indicator synchronous Create, the KMIP 2.1 Create/Query/Poll/Destroy lifecycle, the KMIP 2.1 Create/Process/Poll/Destroy route, and the KMIP 2.1 Query filter intersection route execute in applicable cells. The local continuation after Green commit `26d5f047` adds direct and standalone C Adjust Attribute checks across KMIP 2.0/2.1 and all six encoding/transport pairs; its fixture validates the selected identifier, vendor attribute reference, Increment type, and value 5. Full local CTest passed 7/7, including both controlled TLS peers. Exact-head CI and the complete installed route matrix remain open. The C continued-batch conformance test now requires both direct and standalone Get Attribute List constructors to pass and verifies unchanged peer state on each send; the focused test and `c_validator_route_matrix_contracts` pass at commit `d76df104`. The C Adjust Attribute test at commit `26d5f047` also passes direct and standalone request constructors across KMIP 2.0/2.1 and all six encoding/transport pairs; its exact-head matrix run is pending. Other async branches, response-accessor routes, and the broader positive-operation matrix remain pending. Full applicability remains open across all languages, values, batches, async cases, and boundaries. Rust directly executes `KeyBlock::with_key_value`, `KeyBlock::with_key_compression_type`, `KeyBlock::with_key_wrapping_data`, `KeyValue::try_new`, `KeyValue::try_from_item`, all three `KeyMaterial` constructors/parsers, and Key Wrapping Data, Encryption Key Information, and MAC/Signature Key Information constructors and raw/typed Cryptographic Parameters setters for every applicable manifest target with local TTLV roundtrips; the full Rust matrix and remaining languages remain pending |
| KMIPKIT-0092-FR-004 | `validate_report()`, `validate_parity_collection()`, route evidence schema, language peer fixtures | Constructed and sent request trees must match; tree hashes are compared across routes and languages; batch events retain ordered operation names and peer-observed response counts; full language events are pending |
| KMIPKIT-0092-FR-005 | Controlled KMIP peer contracts in each language validator | The C controlled peer checks Mandatory asynchronous Create as Pending with a server-issued correlation and required Response Payload, omitted-indicator Create as synchronous Success with no correlation, and Optional- and Prohibited-indicator Create as synchronous Success without an unnecessary Poll, across all six encoding/transport pairs and both versions; exercises all five catalogued Cancel results; verifies Create(Pending) → Poll(Pending) → Cancel(Canceled) with one terminal result; verifies Create(Pending) → Query(exact correlation) → Poll(Success with Create payload) → authenticated Destroy cleanup; verifies KMIP 2.1 Create(Pending) → Process(exact correlation, Prohibited indicator) → Poll(Success with Create payload) → authenticated Destroy cleanup; and verifies a KMIP 2.1 Query that intersects Create-operation and correlation filters to return only the matching outstanding Create, excluding a completed Create and another pending operation. Process Success has an empty Response Payload per §6.1.39 Table 279. The Optional Pending branch and other supported network rows and lifecycle branches remain pending. |
| KMIPKIT-0092-FR-006 | Language artifact installers, route manifest, and evidence validator | Exact `release/1.0.0` commit and per-language installed artifact digest are checked for all four artifacts; comparable runtime reports are pending |
| KMIPKIT-0092-FR-007 | `validation-results.schema.json`, route evidence schema, aggregator, parity validator | Per-run output retains transport, encoding, API, construction route, execution route, and artifact axes; tests cover same-server/config comparisons |
| KMIPKIT-0092-FR-008 | Report writers and redaction helpers | Secret and raw-message negative fixtures |
| KMIPKIT-0092-FR-009 | Capability manifest and per-language validator route registries | Common matrix fails missing target routes unless the exact public API limitation is source-backed; Secret Data constructor and parser target sets are limited to their own object subtree and omit the two Key Block fields excluded by KMIP 2.0/2.1 §3.1 Table 11. Exact C CI run `43539660499` on head `5adcf0c` had 39,313 invoked cells, 27,423 `validator_recipe_pending`, and zero failed cells; both red steps were caused by pending route recipes. The latest exact C installed-consumer run `43571020018` at head `52561206` has 41,051/66,736 invoked cells, zero failed, and 25,685 pending, all `validator_recipe_pending`; the matching completeness validator now fails on missing routes beginning with `test-async-unknown-correlation-rejected`. The gate remains open. |
| KMIPKIT-0092-FR-010 | Pinned OASIS inventory, catalog generator, and normative inventory tests | Source-backed inventory tests cover versioned operations, objects, attributes, TTLV types, enum values, flags, field branches, and defined boundary/error classes |

### C Encrypt and Decrypt operation routes

For KMIPKIT-0092-FR-003, FR-005, and FR-009, the catalogued C `test-op-encrypt`
and `test-op-decrypt` cases invoke their typed constructors and response
accessors through the installed ABI. The controlled peer verifies the AES
known-answer data, ordered Register/Activate/Encrypt/Decrypt/Revoke/Destroy
operations, and restored object state over KMIP 2.0/2.1, TTLV/JSON/XML, and raw
TLS/HTTPS. Normative references: KMIP 2.0/2.1 §§6.1.17 Encrypt, 6.1.11 Decrypt,
6.1.39/6.1.43 Register, 6.1.1 Activate, 6.1.40/6.1.44 Revoke, 6.1.15 Destroy,
8.2–8.6, 9.5–9.8, and 4.50/4.57.

- Red `5d30c6208` failed with zero checks before operation-case dispatch was
  registered.
- Green `82019abca` passed the focused test: 24 controlled peer exchanges and
  76/76 typed C route cells. Integration test commit `d57e11dd2` updates the
  controlled-peer operation-count contract. Full CTest passed 7/7 in 94.61
  seconds. The exhaustive local matrix invoked 42,636/66,692 cells, with zero
  failed and 24,056 pending recipe cells; exact-head hosted CI remains pending.
  No cross-language parity claim is made.

### Certify Public Key fixture correction

For KMIPKIT-0092-FR-003 and FR-005, `test-op-certify` creates an RSA
Public/Private Key Pair with Create Key Pair (§6.1.9), uses the returned
Public Key identifier in a separate Certify request (§6.1.6), and destroys
the generated Certificate and both keys. The generator-owned test catalog
records this fixture and teardown. Local evidence includes the focused
12-cell route test, `test_route_matrix.py` 172/172, and both controlled TLS
peers 2/2. The exact-head CI attempt for the preceding branch head
`f25f281f7` was run `43606292882`; it failed route-matrix completeness with
42,074 of 66,736 cells invoked and 24,662 pending `validator_recipe_pending`
cells. This correction has not yet been pushed or checked by hosted CI.

The common contract establishes planned obligations, schema behavior, and
artifact availability. It does not claim that any validator has executed the
full matrix. FR-003, FR-005, and the runtime portion of FR-006 remain open until
the four independent language branches emit evidence from actual public API
calls and controlled-peer exchanges.

### C batch route follow-up

For KMIPKIT-0092-FR-003 and FR-009, the C `test-batch-continue-after-failure`
fixture now invokes both direct Get Attribute List construction and conversion
from `GetAttributeListRequest`, across KMIP 2.0/2.1 and TTLV/JSON/XML over raw
TLS/HTTPS. Its focused test checks 60 peer exchanges and 36 request-based
construct/send/decode cells. This closes only that scenario slice: the latest
exact-head CI on parent commit `3a21e925` still reports 25,723 C validator
recipe cells pending, so the exhaustive route gate remains open.

For FR-003 and FR-009, C now executes the read-only
`test-batch-count-mismatch-rejected` scenario through the installed typed batch
constructors. Twelve controlled-peer checks across both protocol versions,
three encodings, and two transports assert that transmitted Batch Count
matches the two request items. This exercises the typed API's derived count;
it does not create or send an arbitrary malformed message. The broader C
matrix and all-language evidence remain open.

For FR-003, FR-005, and FR-009, C now executes the Archive request route using
the installed public ABI. The controlled peer checks the KMIP 2.0/2.1 request
and response shapes, ID Placeholder resolution, returned Unique Identifier,
ordered Revoke/Destroy cleanup, and observable object state across 12
version/encoding/transport combinations. The focused Archive test and full
local CTest suite pass. This closes only the Archive C route slice; the full C
route matrix, exact-head CI, and Rust/Java/Python parity remain open.

The latest local C matrix run `c-route-08336150-bcb0-4ee3-9ce9-e9c5ed058a9c`
invoked 41,922/66,736 cells with zero failed events and 24,814 validator
recipes still pending. It records 44 passed Certify events, 42 passed Create
Key Pair events, and all 42 Archive route events passed. This run uses
installed release artifact `489fafc79ef89db2524eafa7a6c6482f5cf75d97`
(SHA-256 `980e85fdf6982b52360895b9c0713fbef42b294d22fbeb7d71136fe32235af65`)
and is local evidence, not exact-head CI or a complete matrix.

The Archive continuation also fixes stale catalog provenance after a rebase:
the CMake validator now reconfigures when the shared catalog changes, keeping
the compiled report hash aligned with the inputs used by the route runner.
The provenance regression and both controlled TLS peer suites pass.

For KMIPKIT-0092-FR-003, FR-005, and FR-009, the C `test-op-create-key-pair`
route now exercises the installed `kmipkit_client_batch_item_create_key_pair`
constructor and `kmipkit_client_batch_item_response_create_key_pair` accessor
under OASIS KMIP v2.0/v2.1 §6.1.9. The controlled peer validates both returned
identifiers and the separate two-item Destroy cleanup request across 12
version/encoding/transport combinations. All 12 focused checks pass and the
peer's object-state digest returns to its initial value. The full matrix,
exact-head CI, and equivalent Rust/Java/Python routes remain open.

For KMIPKIT-0092-FR-003, FR-005, and FR-009, C now executes Certify through the
installed typed ABI under OASIS KMIP v2.0/v2.1 §6.1.6, Tables 179 and 180. The
controlled route creates an RSA-2048 Public Key, supplies X.509 Certificate
Type, checks the generated Certificate Unique Identifier and complete response
TTLV, then destroys both objects. All 12 version/encoding/transport checks
pass and the fixture returns to its initial state. The generated catalog now
uses the required Public Key fixture for Certify. Full matrix and
Rust/Java/Python parity remain open.

For KMIPKIT-0092-FR-003, FR-005, and FR-009, C now executes Sign through the
installed typed ABI under OASIS KMIP v2.0 §6.1.49, Tables 302–304, and KMIP
v2.1 §6.1.55, Tables 334–336. Across 24 version/encoding/transport checks,
the controlled peer validates Create Private Key → Activate → Sign → Revoke →
Destroy, explicit SHA-256/RSA parameters, the typed response Unique Identifier
and Signature Data, and restoration of fixture state. The focused test and
full local CTest suite pass 7/7. The full C route matrix, exact-head CI, and
Rust/Java/Python parity remain open.

For KMIPKIT-0092-FR-003, FR-005, and FR-009, the C Archive route also executes
the four KMIP 2.0/2.1 §6.1.4 payload fixtures from Tables 158, 159, 173, and
174 through the installed public ABI. The request fixtures exercise both
Unique Identifier presence and omission; the response fixtures decode and
check the required successful Unique Identifier. The controlled peer runs all
six TTLV/JSON/XML and raw TLS/HTTPS combinations for each applicable version:
24 checks passed, with Create setup, Revoke/Destroy cleanup, and final peer
state restored. The request's omitted identifier passes a null optional
identifier pointer to `kmipkit_client_batch_item_archive`; no raw TTLV message
is sent. The full C route matrix, exact-head CI, and Rust/Java/Python parity
remain open.

### C Archive payload variants development evidence

- Red commit `0e1a24c60`: the new focused test failed with zero checks because
  the four normative local-model Archive payload cases had no controlled-peer
  recipes.
- Green commit `5395be5eb`: the C route now sends optional Unique Identifier
  both present and omitted through the installed `kmipkit_client_batch_item_archive`
  ABI, and validates required Archive success identifiers for both protocol
  versions, all three encodings, and raw TLS/HTTPS.
- Verification: the focused test passed 24/24 controlled-peer checks; full
  `ctest --test-dir interop/c-kmip-validator/build --output-on-failure` passed
  7/7, including the complete route-matrix contract suite. Artifact provenance
  verification passed for pinned release commit
  `489fafc79ef89db2524eafa7a6c6482f5cf75d97` with SHA-256
  `912974f981dba88a89224c923275d8af3f3284cf50a45bfe29d2c3d33dfc4764`.
- This is a bounded Archive slice. The full C route matrix, exact-head CI, and
  Rust/Java/Python parity remain open.

### C request-converted Get Attribute List batch variant

For KMIPKIT-0092-FR-003, FR-005, and FR-009, the C
`test-batch-create-activate-destroy` route now runs the Get Attribute List item
through both `kmipkit_client_batch_item_get_attribute_list` and
`kmipkit_client_batch_item_get_attribute_list_from_request`. OASIS KMIP 2.0
§§6.1.8, 6.1.1, 6.1.21, 6.1.40, 6.1.15, 8.2–8.6, 9.5–9.8, and 4.50, and KMIP
2.1 §§6.1.8, 6.1.1, 6.1.21, 6.1.44, 6.1.15, 8.2–8.6, 9.5–9.8, and 4.57
govern the ordered Create/Activate/Get Attribute List/Revoke/Destroy batch and
its ID Placeholder usage. The controlled peer validated 24 exchanges across
both versions, three encodings, two transports, and both construction routes.
The full local matrix now records 42,190/66,736 invoked cells, zero failed,
and 24,546 pending in run `c-route-25ebc1e0-3153-40f3-a26e-f16553b6beb4`,
using pinned library commit `489fafc79ef89db2524eafa7a6c6482f5cf75d97`
with artifact SHA-256
`980e85fdf6982b52360895b9c0713fbef42b294d22fbeb7d71136fe32235af65`. This
route slice is covered; the exhaustive matrix, exact-head CI, and parity for
the other languages remain open.

### C ordered placeholder batch options and response accessors

For KMIPKIT-0092-FR-003, FR-005, and FR-009, the C installed consumer now
executes ordered Create/Activate batches with `Batch Order Option` explicitly
set to `true` and omitted, across KMIP 2.0/2.1, TTLV/JSON/XML, and raw TLS/HTTPS.
The fixtures validate ID Placeholder resolution after Create, response item
order and identifiers, response-header TTLV, response metadata v1/v2, typed
Create/Activate response accessors, and absence of asynchronous correlation.
They then use separate Revoke/Destroy cleanup requests. The normative scope is
OASIS KMIP 2.0/2.1 §§6.1.1, 6.1.8, 8.2–8.6, and 9.5–9.8; ID Placeholder is
defined in §6.1 and uses the Unique Identifier value in KMIP 2.0 §4.50 / KMIP
2.1 §4.57. Red/Green commits are `14c14b9ac`/`5e7a48981` for route
construction and `feb6d2ac5`/`66c2b39f4` for response accessors. Both cases
pass 24 controlled version/encoding/transport combinations and account for
240 passing local matrix events. The four matrix cells for generic raw response
TTLV remain pending because this is a successful Create/Activate response and
the repository security invariant allows that accessor only for the narrowly
specified successful MAC, MAC Verify, Sign, and Signature Verify shape
failures. The full matrix and Rust/Java/Python evidence remain open.

### C single-item batch response accessors

For KMIPKIT-0092-FR-003 and FR-009, the C installed consumer now sends the
single-item Get request with Batch Error Continuation Option and Batch Order
Option omitted, then validates the typed Symmetric Key response, response
metadata, response-header TTLV, Unique Batch Item ID, and absence of
asynchronous correlation. OASIS KMIP 2.0/2.1 §§6.1.19, 8.2, 8.5–8.6, and 9.5,
9.7 define the Get operation and message/batch shapes. The caller-owned Any
Object result is released through the documented zeroizing release path and
does not enter diagnostics. Red/Green commits are `5536660b0`/`f697ad925`; the
focused controlled-peer test passes 12 version/encoding/transport exchanges,
and full local CTest passes 7/7 in 104.44 seconds.

### C Undo continuation and rollback response

For KMIPKIT-0092-FR-003, FR-005, and FR-009, the installed C consumer sends a
typed Create/Get batch using Undo continuation. The controlled peer confirms
that the later Get failure changes the earlier Create result to Operation
Undone and restores the fixture state. The Operation Undone response retains
its Create payload because OASIS KMIP 2.0/2.1 §8.6 requires Response Payload
when Result Status is not Failure. Red commit `382982aff` added the failing
route requirement; the Green implementation and verification evidence are
recorded in `tasks.md`. The focused route passed all 12 combinations, CTest
passed 7/7, and the subsequent full C matrix run records zero failed cells
with 24,242 pending recipe cells; matrix completeness remains open.


### C Continue failed-item response accessors

For KMIPKIT-0092-FR-003, FR-005, and FR-009, the installed consumer checks the
failed Get response in a Continue batch through the typed result, metadata,
response-header, pending-correlation, and raw-TTLV-absence accessors. The
following successful Get Attribute List response is checked through typed
identifiers and attribute references and its operation-specific payload TTLV
accessor.
OASIS KMIP 2.0/2.1 §§8.2–8.6 and 9.5–9.7 define continuation and batch
response behavior; §11.5 defines Object Not Found. Red/Green commits are
`371057393`/`196fad2e4` and `52757ebb8`/`f0012f2b6`; the focused route passes
24 constructor, protocol, encoding, and transport checks. Full CTest passed
7/7 in 95.87 seconds; exact-head CI and remaining route matrix coverage remain
open.

### C Create Split Key operation route

KMIPKIT-0092-FR-003, FR-005, and FR-009 are exercised for
`test-op-create-split-key` through the installed C ABI and controlled peers.
The request uses an AES-128 Symmetric Key, three parts, threshold two, and XOR;
the consumer validates the response count and all three typed Unique
Identifiers, then destroys every returned object. OASIS sources: KMIP 2.0
§6.1.10 Tables 178–179 and KMIP 2.1 §6.1.10 Tables 193–194.

- Red: `e00665d1e`; the focused test failed with no controlled-peer checks.
- Green: `420a7c597`; the typed C request/response route and stateful cleanup
  recipe produced 12 passing version/encoding/transport checks.
- Full local CTest passed 7/7, including the contract suite and both KMIP 2.0
  and 2.1 controlled TLS peers. Exact-head hosted CI, full matrix completion,
  and Rust/Java/Python parity remain outstanding.

### C forward ID Placeholder rejection

KMIPKIT-0092-FR-002 plans no response decoder when the catalog authorizes a
local batch-construction rejection; KMIPKIT-0092-FR-003 and FR-009 map the C
installed route and zero-transport evidence. The C consumer constructs
`Activate(ID Placeholder from item 2)` followed by Create, requires the
Validation/Invalid Input/Not Sent outcome, and confirms the peer receives no
connection. OASIS KMIP 2.0/2.1 §§8.3 and 9.7 are the catalogued references.
Red `120855254` and Green `800798706` pass the focused matrix and C route tests;
the local route test records 24 not-sent item events over both versions and all
six wire paths. Full CTest passed 7/7; shared coverage matrix passed 177 tests
and route evidence validation passed 33 tests. The full local C matrix records
42,560/66,736 cells invoked, zero failed, and 24,132 pending validator recipes.
The complete C route matrix and exact-head CI remain open.

### C Export route slice

For KMIPKIT-0092-FR-003 and FR-009, `test-op-export` now exercises all 44
catalogued C cells: local generic TTLV decode/item construction, the installed
typed Export constructor through six encoding/transport combinations for each
protocol version, and both typed and generic response accessors. The controlled
peer checks OASIS KMIP v2.0 §6.1.18 Tables 202–203 and v2.1 §6.1.18 Tables
217–218, including all four required Export response payload fields, then
observes Create → Export → Revoke → Destroy and cleanup. All 12 peer checks pass.
The ten request-payload cases pass 120 present/omitted checks; the eight
response-payload cases pass 96 checks, including rejection after one required
field is removed. Generic TTLV construction and decode cover every Export
payload cell; the exact matrix report has zero Export pending cells and 25,053
pending cells overall. This closes only these Export operation and payload
slices; the FR-003/FR-009 gates remain open.

The common serializer/response-reader route assertion loads the API manifest
from the exact `library_commit` recorded in `route-capabilities.json`, so
functions added to a later release cannot be required from an older installed
artifact. This is part of the FR-006 artifact provenance check.

### C Encrypt route follow-up

For KMIPKIT-0092-FR-003, FR-005, FR-008, and FR-009, the C
`test-op-encrypt` route checks OASIS KMIP v2.0/v2.1 §6.1.17 plus the ordered
Activate, Revoke, and Destroy cleanup clauses (§6.1.1, §6.1.40/§6.1.44,
§6.1.15) and batch/ID Placeholder rules (§6.1, §§8.2–8.4, 8.6). The installed
C client sends the AES-128 Create/Activate/Encrypt/Revoke/Destroy request;
the controlled peer validates the exact fixture input/response Data bytes,
explicit AES/ECB/No Padding parameters, IDs, typed response fields, and final
object deletion. The consumer checks typed response presence and length; the
randomly generated remote key is not the key associated with the fixture
vector, so no cryptographic correctness is inferred. The focused test passed
all 12 version/encoding/transport combinations, and CTest
passed 7/7 with 164 C route-matrix contract tests. No request data is included
in evidence. Other network rows remain `validator_recipe_pending`; this route
does not close the full matrix or the cross-language FR-003/FR-005 gates.

### C Decrypt route follow-up

For KMIPKIT-0092-FR-003, FR-005, FR-008, and FR-009, the C `test-op-decrypt`
route checks OASIS KMIP v2.0/v2.1 §6.1.11 and its ordered key lifecycle:
Activate (§6.1.1), Revoke (§6.1.40/§6.1.44), Destroy (§6.1.15), and batch/ID
Placeholder processing (§6.1, §§8.2–8.4, 8.6). The installed client sends an
AES-128 Decrypt-only key request with explicit AES/ECB/No Padding parameters;
the peer validates exact fixture ciphertext input and response Data bytes, IDs,
and deletion. The consumer checks the typed 16-byte Data result. The random
remote key is not associated with the fixture bytes, so this is not a
cryptographic correctness test.
Both Encrypt and Decrypt passed all 12 version/encoding/transport exchanges;
CTest passed 7/7 with 165 C route-matrix contract tests. Other route cells
remain pending, so this does not close the full FR-003/FR-005 gates.

### C MAC route follow-up

For KMIPKIT-0092-FR-003, FR-005, FR-008, and FR-009, C
`test-op-mac` exercises KMIP v2.0 §6.1.31 and v2.1 §6.1.32 through the
installed client API. It creates an HMAC-SHA256 Symmetric Key with explicit
MAC Generate usage, activates it, sends MAC with explicit HMAC-SHA256
Cryptographic Parameters, and revokes/destroys it. The peer checks the exact
request tree, deterministic MAC Data response bytes, item IDs, and final
deletion; the C consumer checks the typed response, UID, and 32-byte MAC Data
length. Since the server generates a random key, this controlled response does
not prove cryptographic MAC validity. The focused test passes all 12 cells;
CTest passes 7/7 including 166 matrix-contract tests. Other C route cells and
cross-language FR-003/FR-005 remain pending.

### C MAC Verify route follow-up

For KMIPKIT-0092-FR-003, FR-005, FR-008, and FR-009, C
`test-op-mac-verify` exercises OASIS KMIP v2.0 §6.1.32 and v2.1 §6.1.33 with a
typed ordered Create/Activate/MAC Verify/Revoke/Destroy batch. It uses an
HMAC-SHA256 key with explicit MAC Verify usage, explicit HMAC-SHA256 parameters,
fixed data and MAC Data, and ID Placeholder references. The peer validates the
exact request tree and lifecycle, and returns the created UID plus the typed
Validity Indicator `Valid`; the consumer checks both fields. The server-created
key is random and is not associated with the fixed MAC, so this fixture does
not claim cryptographic validity. The focused test passed all 12
version/encoding/transport exchanges; CTest passed 7/7 with 167 matrix-contract
tests. Other operation and cross-language route cells remain pending under
FR-003/FR-005.

### C Sign and Signature Verify route follow-up

For KMIPKIT-0092-FR-003, FR-005, FR-008, and FR-009, the installed C consumer
now covers OASIS KMIP v2.0 §6.1.49/§6.1.50 and v2.1 §6.1.55/§6.1.56. The
authoritative catalog generator uses Create Key Pair rather than SymmetricKey:
§12.1 Table 61 permits Sign usage for PGP Key or Private Key and Verify usage
for PGP Key, Certificate, or Public Key. The request creates RSA-2048 keys
with separate masks, activates the selected pair member, sends the explicitly
selected SHA-256 with RSA operation, and cleans up both keys. The peer validates
all request attributes, pair-specific identifiers, typed results, item IDs,
and final state. Sign's response is checked for a 256-byte signature field;
Signature Verify's `Valid` result is a typed test-fixture value. Neither fixture
establishes cryptographic validity because the private key is generated by the
server and the verification input is not derived from the pair. Catalog Red
`e80371257`, Sign Red `2782a91dc`, and Signature Verify Red `5a855d802` preceded
catalog Green `bd6d9b0c8` and route Green `389ac4465`. Both focused routes passed
all 12 version/encoding/transport cells; CTest passed 7/7. Other C and
cross-language route cells remain pending under FR-003/FR-005.

### C Create Key Pair route follow-up

The installed C route for `test-op-create-key-pair` covers OASIS KMIP v2.0 and
v2.1 §6.1.9. It constructs the typed Create Key Pair request with RSA-2048 and
explicit usage masks, reads both typed response identifiers, requires them to
be distinct, and destroys each returned object using the corresponding
Create Key Pair Private Key or Public Key selector. The controlled peer checks
the ordered three-item batch, exact attribute structures, response fields,
Unique Batch Item IDs, and final state. Red `36034c7a6` recorded the missing
catalog cleanup and C route; Green `89bd02b79` added generator-backed catalog
cleanup and the installed C route. The focused route passes all 12
version/encoding/transport cells; the normative inventory passes 26/26, the
catalog generation check passes, and CTest passes 7/7. Key material is not
inspected or cryptographically validated. Remaining C and cross-language
operation routes remain pending under FR-003/FR-005.

### C Certify route follow-up

The installed C `test-op-certify` route covers OASIS KMIP v2.0/v2.1 §6.1.6;
the Certificate Sign usage assignment follows §12.1 Table 495. It creates an
RSA-2048 pair with explicit Certificate Sign usage on the Private Key, activates
that key, certifies the paired Public Key with the X.509 Certificate Type in
Attributes, destroys the certificate through ID Placeholder, then revokes and
destroys both keys. The client checks the typed certificate identifier and
the generic response TTLV identifier through the public Certify TTLV accessor.
This covers the C client execution and typed lifecycle
checks traced by FR-003/FR-005, with the route catalog and controlled TLS peer
as executable evidence. Red `109ff223e` and `b86d6d708` preceded Green
`eabd01402`. The focused matrix passed 12/12; both controlled TLS peer tests
passed; the normative inventory passed 27/27; catalog `--check` passed; and
CTest passed 7/7. The peer validates protocol shape and lifecycle, not
certificate or signature cryptography. Other C and cross-language route cells
remain pending.

### C Continue response accessors

For FR-003 and FR-005, the C Continue fixture invokes the typed Get result
accessor for its failed Get, verifies that the Response Payload is omitted,
and reads the successful Get Attribute List through typed and TTLV accessors.
The controlled response and result are checked for both protocol versions and
six encoding/transport combinations under OASIS KMIP v2.0/v2.1 §§6.1.19,
6.1.21, 8.2–8.6, and 9.5–9.7. This closes the response-accessor slice for
that fixture; four local Continue-option constructor cells and the wider
matrix remain pending.
