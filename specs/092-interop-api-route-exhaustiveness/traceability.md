# KMIPKIT-0092 Traceability

| Requirement | Implementation target | Verification |
|---|---|---|
| KMIPKIT-0092-FR-001 | `interop/shared/coverage/route-capabilities.json`, route capability schema, `route_models.py`, and `coverage_matrix.py` | Source-backed route inventory binds typed attribute markers to exact elements, maps Register/Revoke request and batch-item constructors for every language, covers the Batch Error Continuation Option builder across four languages (§§8.2, 9.6, 11.5), limits batch result readers to KMIP 2.0/2.1 response Batch Count and Batch Item fields (§§8.5–8.6, 9.5, 9.7), keeps §11.34 Object Type Enumeration targets separate from §4.36 Object Type Attribute routes, prevents attribute wrappers from claiming managed-object structures, scopes Secret Data constructor/parser routes to their own object tree and excludes Cryptographic Algorithm and Cryptographic Length as required by KMIP 2.0/2.1 §3.1 Table 11, scopes Key Wrapping Data setters to their exact §3.3 Tables 14–17 member and prevents Encryption Key Information and MAC/Signature Key Information Cryptographic Parameters routes from crossing sibling structures, limits Rust `KeyMaterial::byte_string` to the generic Key Material tag because KMIP 2.0/2.1 §11.25 distinguishes ByteString and Structure formats, excludes checked-`Tag` constructors and Item-view accessors only when reserved/unused allocations cannot yield an Item, retains RawTag construction and decoder-rejection rows, and binds async operations to exact public executors; installed artifact probes for all four languages |
| KMIPKIT-0092-FR-002 | `coverage_matrix.py`, `validate_route_evidence.py`, and language CI follow-ups | Negative tests reject missing target/version/route/action cells, duplicate or unplanned evidence, and incorrect scenario axes; expected NotSent cases require zero transport calls and omit impossible response decoding; async request cells use per-operation senders; actual full language CI is pending |
| KMIPKIT-0092-FR-003 | Per-language recipe and local TTLV suites; C `route_matrix.py` and `test_route_matrix.py` | C installed-ABI coverage parses complete Request Messages (root, structures, Protocol Version, Batch Count, Operation, Request Payload, Unique Batch Item ID, Maximum Response Size, optional Header fields, and batch options) and complete Response Messages (root, structures, optional Header and Batch Item fields including a valid Failure/Result Reason/Result Message shape) for KMIP 2.0/2.1. Exact C installed-consumer run `43571020018` at head `52561206` records 41,051/66,736 invoked cells, zero failed, and 25,685 pending, all `validator_recipe_pending`. Five catalogued Cancel Cancellation Result values, Mandatory asynchronous Create, omitted-indicator synchronous Create, Optional-indicator synchronous Create, explicit Prohibited-indicator synchronous Create, the KMIP 2.1 Create/Query/Poll/Destroy lifecycle, the KMIP 2.1 Create/Process/Poll/Destroy route, and the KMIP 2.1 Query filter intersection route execute in applicable cells. The local continuation after Green commit `26d5f047` adds direct and standalone C Adjust Attribute checks across KMIP 2.0/2.1 and all six encoding/transport pairs; its fixture validates the selected identifier, vendor attribute reference, Increment type, and value 5. Full local CTest passed 7/7, including both controlled TLS peers. Exact-head CI and the complete installed route matrix remain open. The C continued-batch conformance test now requires both direct and standalone Get Attribute List constructors to pass and verifies unchanged peer state on each send; the focused test and `c_validator_route_matrix_contracts` pass at commit `d76df104`. The C Adjust Attribute test at commit `26d5f047` also passes direct and standalone request constructors across KMIP 2.0/2.1 and all six encoding/transport pairs; its exact-head matrix run is pending. Other async branches, response-accessor routes, and the broader positive-operation matrix remain pending. The local candidate run `c-route-c4e86961-430d-49a4-ab47-5aed58a24602`, using the pinned release artifact `489fafc79ef89db2524eafa7a6c6482f5cf75d97` with digest `c93eb8f7daa60d3b6832d297887e99cd2101e4f9baf2d04f55ce4042ba4bb7a2`, records 41,730 of 66,736 cells invoked, zero failed, and 25,006 pending; the ten Delegated Login request/response field cases are covered. This does not establish exact-head CI completion. Full applicability remains open across all languages, values, batches, async cases, and boundaries. |
| KMIPKIT-0092-FR-004 | `validate_report()`, `validate_parity_collection()`, route evidence schema, language peer fixtures | Constructed and sent request trees must match; tree hashes are compared across routes and languages; batch events retain ordered operation names and peer-observed response counts; full language events are pending |
| KMIPKIT-0092-FR-005 | Controlled KMIP peer contracts in each language validator | The C controlled peer checks Mandatory asynchronous Create as Pending with a server-issued correlation and required Response Payload, omitted-indicator Create as synchronous Success with no correlation, and Optional- and Prohibited-indicator Create as synchronous Success without an unnecessary Poll, across all six encoding/transport pairs and both versions; exercises all five catalogued Cancel results; verifies Create(Pending) → Poll(Pending) → Cancel(Canceled) with one terminal result; verifies Create(Pending) → Query(exact correlation) → Poll(Success with Create payload) → authenticated Destroy cleanup; verifies KMIP 2.1 Create(Pending) → Process(exact correlation, Prohibited indicator) → Poll(Success with Create payload) → authenticated Destroy cleanup; and verifies a KMIP 2.1 Query that intersects Create-operation and correlation filters to return only the matching outstanding Create, excluding a completed Create and another pending operation. Process Success has an empty Response Payload per §6.1.39 Table 279. The Optional Pending branch and other supported network rows and lifecycle branches remain pending. |
| KMIPKIT-0092-FR-006 | Language artifact installers, route manifest, and evidence validator | Exact `release/1.0.0` commit and per-language installed artifact digest are checked for all four artifacts; comparable runtime reports are pending |
| KMIPKIT-0092-FR-007 | `validation-results.schema.json`, route evidence schema, aggregator, parity validator | Per-run output retains transport, encoding, API, construction route, execution route, and artifact axes; tests cover same-server/config comparisons |
| KMIPKIT-0092-FR-008 | Report writers and redaction helpers | Secret and raw-message negative fixtures |
| KMIPKIT-0092-FR-009 | Capability manifest and per-language validator route registries | Common matrix fails missing target routes unless the exact public API limitation is source-backed; Secret Data constructor and parser target sets are limited to their own object subtree and omit the two Key Block fields excluded by KMIP 2.0/2.1 §3.1 Table 11. Exact C CI run `43539660499` on head `5adcf0c` had 39,313 invoked cells, 27,423 `validator_recipe_pending`, and zero failed cells; both red steps were caused by pending route recipes. The latest exact C installed-consumer run `43571020018` at head `52561206` has 41,051/66,736 invoked cells, zero failed, and 25,685 pending, all `validator_recipe_pending`; the matching completeness validator now fails on missing routes beginning with `test-async-unknown-correlation-rejected`. The gate remains open. |
| KMIPKIT-0092-FR-010 | Pinned OASIS inventory, catalog generator, and normative inventory tests | Source-backed inventory tests cover versioned operations, objects, attributes, TTLV types, enum values, flags, field branches, and defined boundary/error classes |

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
