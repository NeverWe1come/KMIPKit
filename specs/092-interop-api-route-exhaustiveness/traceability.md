# KMIPKIT-0092 Traceability

| Requirement | Implementation target | Verification |
|---|---|---|
| KMIPKIT-0092-FR-001 | `interop/shared/coverage/route-capabilities.json`, route capability schema, `route_models.py`, and `coverage_matrix.py` | Source-backed route inventory binds typed attribute markers to exact elements, maps Register/Revoke request and batch-item constructors for every language, covers the Batch Error Continuation Option builder across four languages (§§8.2, 9.6, 11.5), limits batch result readers to KMIP 2.0/2.1 response Batch Count and Batch Item fields (§§8.5–8.6, 9.5, 9.7), keeps §11.34 Object Type Enumeration targets separate from §4.36 Object Type Attribute routes, prevents attribute wrappers from claiming managed-object structures, scopes Secret Data constructor/parser routes to their own object tree and excludes Cryptographic Algorithm and Cryptographic Length as required by KMIP 2.0/2.1 §3.1 Table 11, scopes Key Wrapping Data setters to their exact §3.3 Tables 14–17 member and prevents Encryption Key Information and MAC/Signature Key Information Cryptographic Parameters routes from crossing sibling structures, limits Rust `KeyMaterial::byte_string` to the generic Key Material tag because KMIP 2.0/2.1 §11.25 distinguishes ByteString and Structure formats, excludes checked-`Tag` constructors and Item-view accessors only when reserved/unused allocations cannot yield an Item, retains RawTag construction and decoder-rejection rows, and binds async operations to exact public executors; installed artifact probes for all four languages |
| KMIPKIT-0092-FR-002 | `coverage_matrix.py`, `validate_route_evidence.py`, and language CI follow-ups | Negative tests reject missing target/version/route/action cells, duplicate or unplanned evidence, and incorrect scenario axes; expected NotSent cases require zero transport calls and omit impossible response decoding; async request cells use per-operation senders; actual full language CI is pending |
| KMIPKIT-0092-FR-003 | Per-language recipe and local TTLV suites; C `route_matrix.py` and `test_route_matrix.py`; Rust language recipe evidence | C installed-ABI coverage parses complete Request Messages (root, structures, Protocol Version, Batch Count, Operation, Request Payload, Unique Batch Item ID, Maximum Response Size, optional Header fields, and batch options) and complete Response Messages (root, structures, optional Header and Batch Item fields including a valid Failure/Result Reason/Result Message shape) for KMIP 2.0/2.1. Exact C installed-consumer run `43571020018` at head `52561206` records 41,051/66,736 invoked cells, zero failed, and 25,685 pending, all `validator_recipe_pending`. Five catalogued Cancel Cancellation Result values, Mandatory asynchronous Create, omitted-indicator synchronous Create, Optional-indicator synchronous Create, explicit Prohibited-indicator synchronous Create, the KMIP 2.1 Create/Query/Poll/Destroy lifecycle, the KMIP 2.1 Create/Process/Poll/Destroy route, and the KMIP 2.1 Query filter intersection route execute in applicable cells. The local continuation after Green commit `26d5f047` adds direct and standalone C Adjust Attribute checks across KMIP 2.0/2.1 and all six encoding/transport pairs; its fixture validates the selected identifier, vendor attribute reference, Increment type, and value 5. Full local CTest passed 7/7, including both controlled TLS peers. Exact-head CI and the complete installed route matrix remain open. The C continued-batch conformance test now requires both direct and standalone Get Attribute List constructors to pass and verifies unchanged peer state on each send; the focused test and `c_validator_route_matrix_contracts` pass at commit `d76df104`. The C Adjust Attribute test at commit `26d5f047` also passes direct and standalone request constructors across KMIP 2.0/2.1 and all six encoding/transport pairs; its exact-head matrix run is pending. Other async branches, response-accessor routes, and the broader positive-operation matrix remain pending. Full applicability remains open across all languages, values, batches, async cases, and boundaries. | Rust directly executes `KeyBlock::with_key_value`, `KeyBlock::with_key_compression_type`, `KeyBlock::with_key_wrapping_data`, `KeyValue::try_new`, `KeyValue::try_from_item`, all three `KeyMaterial` constructors/parsers, and Key Wrapping Data, Encryption Key Information, and MAC/Signature Key Information constructors and raw/typed Cryptographic Parameters setters for every applicable manifest target with local TTLV roundtrips; the full Rust matrix and remaining languages remain pending |
| KMIPKIT-0092-FR-004 | `validate_report()`, `validate_parity_collection()`, route evidence schema, language peer fixtures | Constructed and sent request trees must match; tree hashes are compared across routes and languages; batch events retain ordered operation names and peer-observed response counts; full language events are pending |
| KMIPKIT-0092-FR-005 | Controlled KMIP peer contracts in each language validator | The C controlled peer checks Mandatory asynchronous Create as Pending with a server-issued correlation and required Response Payload, omitted-indicator Create as synchronous Success with no correlation, and Optional- and Prohibited-indicator Create as synchronous Success without an unnecessary Poll, across all six encoding/transport pairs and both versions; exercises all five catalogued Cancel results; verifies Create(Pending) → Poll(Pending) → Cancel(Canceled) with one terminal result; verifies Create(Pending) → Query(exact correlation) → Poll(Success with Create payload) → authenticated Destroy cleanup; verifies KMIP 2.1 Create(Pending) → Process(exact correlation, Prohibited indicator) → Poll(Success with Create payload) → authenticated Destroy cleanup; and verifies a KMIP 2.1 Query that intersects Create-operation and correlation filters to return only the matching outstanding Create, excluding a completed Create and another pending operation. Process Success has an empty Response Payload per §6.1.39 Table 279. The Optional Pending branch and other supported network rows and lifecycle branches remain pending. |
| KMIPKIT-0092-FR-006 | Language artifact installers, route manifest, and evidence validator | Exact `release/1.0.0` commit and per-language installed artifact digest are checked for all four artifacts; comparable runtime reports are pending |
| KMIPKIT-0092-FR-007 | `validation-results.schema.json`, route evidence schema, aggregator, parity validator | Per-run output retains transport, encoding, API, construction route, execution route, and artifact axes; tests cover same-server/config comparisons |
| KMIPKIT-0092-FR-008 | Report writers and redaction helpers | Secret and raw-message negative fixtures |
| KMIPKIT-0092-FR-009 | Capability manifest and per-language validator route registries | Common matrix fails missing target routes unless the exact public API limitation is source-backed; Secret Data constructor and parser target sets are limited to their own object subtree and omit the two Key Block fields excluded by KMIP 2.0/2.1 §3.1 Table 11. Exact C CI run `43539660499` on head `5adcf0c` had 39,313 invoked cells, 27,423 `validator_recipe_pending`, and zero failed cells; both red steps were caused by pending route recipes. The latest exact C installed-consumer run `43571020018` at head `52561206` has 41,051/66,736 invoked cells, zero failed, and 25,685 pending, all `validator_recipe_pending`; the matching completeness validator now fails on missing routes beginning with `test-async-unknown-correlation-rejected`. The gate remains open. |
| KMIPKIT-0092-FR-010 | Pinned OASIS inventory, catalog generator, and normative inventory tests | Source-backed inventory tests cover versioned operations, objects, attributes, TTLV types, enum values, flags, field branches, and defined boundary/error classes |

The common contract establishes planned obligations, schema behavior, and
artifact availability. It does not claim that any validator has executed the
full matrix. FR-003, FR-005, and the runtime portion of FR-006 remain open until
the four independent language branches emit evidence from actual public API
calls and controlled-peer exchanges.

### C Batch Order and ID Placeholder cases

For KMIPKIT-0092-FR-003, FR-005, and FR-009, the C installed consumer now
executes the catalogued Create → Activate cases with explicit Batch Order
Option true and with the option omitted. The controlled TLS peer checks
request/response order, Unique Batch Item ID correlation, ID Placeholder
resolution, and cleanup. OASIS KMIP 2.0/2.1 §§8.1–8.6, 9.5–9.8, 9.21, 6.1.1,
and 6.1.8 are the catalogued references; §9.8 defines the omitted option's
effective value as true. Both scenarios pass all 24 version/encoding/transport
cells. This is one route slice only: the full C matrix and the cross-language
FR-003/FR-005 gates remain open.

### C two-item batch read constructors

For KMIPKIT-0092-FR-003, FR-005, and FR-009, the C
`test-batch-count-matches-items` and
`test-batch-order-false-correlates-response-items` scenarios execute both the
direct Get Attribute List builder and the standalone `GetAttributeListRequest`
builder followed by `kmipkit_client_batch_item_get_attribute_list_from_request`.
The controlled peer checks Batch Count, response count, and Unique Batch Item
ID correlation; it returns the unordered case's two responses in reverse
order. OASIS KMIP 2.0/2.1 §§6.1.19, 6.1.21, 8.2–8.6, and 9.5–9.8 are the
catalogued references. The two cases pass 48 constructor/transport checks.
This closes only these two batch-construction slices; the full C and
cross-language matrix gates remain open.

### C batch route follow-up

For KMIPKIT-0092-FR-003 and FR-009, the C `test-batch-continue-after-failure`
fixture now invokes both direct Get Attribute List construction and conversion
from `GetAttributeListRequest`, across KMIP 2.0/2.1 and TTLV/JSON/XML over raw
TLS/HTTPS. Its focused test checks 60 peer exchanges and 36 request-based
construct/send/decode cells. This closes only that scenario slice: the latest
exact-head CI on parent commit `3a21e925` still reports 25,723 C validator
recipe cells pending, so the exhaustive route gate remains open.

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
destroys both keys. This covers the C client execution and typed lifecycle
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

### C Archive route follow-up

For FR-003 and FR-005, the installed C consumer sends Archive using the
ID Placeholder and checks the required response Unique Identifier through the
typed response accessor. The controlled fixture executes Create → Archive →
Revoke → Destroy, matching the catalog, and checks operation order, unique item
IDs, response fields, and teardown. OASIS KMIP 2.0 §6.1.4 Tables 158–159 and
KMIP 2.1 §6.1.4 Tables 173–174 do not require Activate; the Unique Identifier
is optional and the client is only indicating that a KMS may archive the object.
Red `b2e3ced75` asserts the catalogued recipe, Green `5707246cb` implements it,
and Refactor `de029189b` clarifies the state handling. The route passed 12/12
cells and CTest passed 7/7. At the Archive integration snapshot, the full C
matrix had 24,673 pending cells, so the
FR-003/FR-005 gates remain open.

### C MAC and Sign response TTLV accessors

For KMIPKIT-0092-FR-003 and FR-005, the installed consumer invokes the generic
response-TTLV accessor for successful MAC and Sign responses. The corresponding
normative sources are OASIS KMIP v2.0 §§6.1.31 and 6.1.49 and v2.1 §§6.1.32
and 6.1.55. The fixture checks the complete response item's four outer fields
without reading MAC Data or Signature Data. Red `29a3e1867` exposed the four
missing decode cells; Green `3bbfc5180` invoked both accessors; Refactor
`01b12cbb7` named the shared shape contract. The focused route test passes for
both operations and versions, and CTest passes 7/7. The matrix at
`01b12cbb7` has 66,736 expected, 42,067 invoked, zero failed, and 24,669
pending cells; the global requirements remain incomplete.
