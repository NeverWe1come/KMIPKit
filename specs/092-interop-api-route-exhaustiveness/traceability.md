# KMIPKIT-0092 Traceability

| Requirement | Implementation target | Verification |
|---|---|---|
| KMIPKIT-0092-FR-001 | `interop/shared/coverage/route-capabilities.json`, route capability schema, `route_models.py`, and `coverage_matrix.py` | Source-backed route inventory binds typed attribute markers to exact elements, maps Register/Revoke request and batch-item constructors for every language, covers the Batch Error Continuation Option builder across four languages (§§8.2, 9.6, 11.5), limits batch result readers to KMIP 2.0/2.1 response Batch Count and Batch Item fields (§§8.5–8.6, 9.5, 9.7), keeps §11.34 Object Type Enumeration targets separate from §4.36 Object Type Attribute routes, prevents attribute wrappers from claiming managed-object structures, scopes Secret Data constructor/parser routes to their own object tree and excludes Cryptographic Algorithm and Cryptographic Length as required by KMIP 2.0/2.1 §3.1 Table 11, scopes Key Wrapping Data setters to their exact §3.3 Tables 14–17 member and prevents Encryption Key Information and MAC/Signature Key Information Cryptographic Parameters routes from crossing sibling structures, limits Rust `KeyMaterial::byte_string` to the generic Key Material tag because KMIP 2.0/2.1 §11.25 distinguishes ByteString and Structure formats, excludes checked-`Tag` constructors and Item-view accessors only when reserved/unused allocations cannot yield an Item, retains RawTag construction and decoder-rejection rows, and binds async operations to exact public executors; installed artifact probes for all four languages |
| KMIPKIT-0092-FR-002 | `coverage_matrix.py`, `validate_route_evidence.py`, and language CI follow-ups | Negative tests reject missing target/version/route/action cells, duplicate or unplanned evidence, and incorrect scenario axes; expected NotSent cases require zero transport calls and omit impossible response decoding; async request cells use per-operation senders; actual full language CI is pending |
| KMIPKIT-0092-FR-003 | Per-language recipe and local TTLV suites; C `route_matrix.py` and `test_route_matrix.py` | C installed-ABI coverage parses complete Request Messages (root, structures, Protocol Version, Batch Count, Operation, Request Payload, Unique Batch Item ID, Maximum Response Size, optional Header fields, and batch options) and complete Response Messages (root, structures, optional Header and Batch Item fields including a valid Failure/Result Reason/Result Message shape) for KMIP 2.0/2.1. The latest local installed C artifact matrix records 40,804/66,736 invoked cells, zero failed, and 25,932 pending. Five catalogued Cancel Cancellation Result values, Mandatory asynchronous Create, omitted-indicator synchronous Create, Optional-indicator synchronous Create, and explicit Prohibited-indicator synchronous Create execute across applicable encoding/transport combinations and both versions; the Optional Pending branch and other async branches and response-accessor routes remain pending. Full applicability remains open across all languages, values, batches, async cases, and boundaries. |
| KMIPKIT-0092-FR-004 | `validate_report()`, `validate_parity_collection()`, route evidence schema, language peer fixtures | Constructed and sent request trees must match; tree hashes are compared across routes and languages; batch events retain ordered operation names and peer-observed response counts; full language events are pending |
| KMIPKIT-0092-FR-005 | Controlled KMIP peer contracts in each language validator | The C controlled peer checks Mandatory asynchronous Create as Pending with a server-issued correlation and required Response Payload, omitted-indicator Create as synchronous Success with no correlation, and Optional- and Prohibited-indicator Create as synchronous Success without an unnecessary Poll, across all six encoding/transport pairs and both versions; exercises all five catalogued Cancel results; and for KMIP 2.1 verifies Create(Pending) → Poll(Pending) → Cancel(Canceled) with the same correlation and exactly one terminal result. The Optional Pending branch and other supported network rows and lifecycle branches remain pending. |
| KMIPKIT-0092-FR-006 | Language artifact installers, route manifest, and evidence validator | Exact `release/1.0.0` commit and per-language installed artifact digest are checked for all four artifacts; comparable runtime reports are pending |
| KMIPKIT-0092-FR-007 | `validation-results.schema.json`, route evidence schema, aggregator, parity validator | Per-run output retains transport, encoding, API, construction route, execution route, and artifact axes; tests cover same-server/config comparisons |
| KMIPKIT-0092-FR-008 | Report writers and redaction helpers | Secret and raw-message negative fixtures |
| KMIPKIT-0092-FR-009 | Capability manifest and per-language validator route registries | Common matrix fails missing target routes unless the exact public API limitation is source-backed; Secret Data constructor and parser target sets are limited to their own object subtree and omit the two Key Block fields excluded by KMIP 2.0/2.1 §3.1 Table 11. Exact C CI run `43539660499` on head `5adcf0c` had 39,313 invoked cells, 27,423 `validator_recipe_pending`, and zero failed cells; both red steps were caused by pending route recipes. The latest local C matrix has 40,804/66,736 invoked cells, zero failed, and 25,932 pending, all `validator_recipe_pending`; the matching completeness validator correctly fails on missing routes. The gate remains open. |
| KMIPKIT-0092-FR-010 | Pinned OASIS inventory, catalog generator, and normative inventory tests | Source-backed inventory tests cover versioned operations, objects, attributes, TTLV types, enum values, flags, field branches, and defined boundary/error classes |

The common contract establishes planned obligations, schema behavior, and
artifact availability. It does not claim that any validator has executed the
full matrix. FR-003, FR-005, and the runtime portion of FR-006 remain open until
the four independent language branches emit evidence from actual public API
calls and controlled-peer exchanges.

### C Adjust Attribute slice

For KMIPKIT-0092-FR-003, FR-005, and FR-009, the installed C route test
`RouteMatrixTests.test_adjust_attribute_operation_runs_from_the_installed_c_consumer`
checks both
`kmipkit_client_batch_item_adjust_attribute` and
`kmipkit_client_batch_item_adjust_attribute_from_request`, followed by
`kmipkit_client_batch_item_response_adjust_attribute`. Its controlled peer
validates the operation fields, response Batch Item ID, and isolated object
state. It covers KMIP 2.0 §6.1.3/Table 155 and §11.1/Table 391, plus KMIP 2.1
§6.1.3/Table 170 and §11.1/Table 429, over all six encoding/transport pairs.
This is one operation's evidence only; the four-language and complete-matrix
requirements above remain open.

### C Batch Count mismatch slice

For KMIPKIT-0092-FR-003, FR-005, and FR-009, the installed C route test
`RouteMatrixTests.test_batch_count_mismatch_is_recomputed_before_transport`
executes the catalogued two-item Get / Get Attribute List case through the
public typed batch API. The controlled peer checks that the request's Batch
Count equals its two Batch Items before returning two successful results. It
covers OASIS KMIP 2.0 and 2.1 §§8.1–8.6, §§9.5–9.8 and §9.21, plus Get
§6.1.19 and Get Attribute List §6.1.21, over all six encoding/transport pairs.
This verifies the recomputed-count path without adding an API for callers to
set Batch Count manually. Both direct and request-based Get Attribute List
constructors are exercised, and the consumer validates the typed Get and Get
Attribute List response accessors. The local matrix reports all 142 cells for
this case invoked; the full C route matrix and rejection path for inconsistent
generic TTLV input remain open.

### C single-item batch options omitted slice

For KMIPKIT-0092-FR-003, FR-005, and FR-009, the installed C route test
`RouteMatrixTests.test_single_item_batch_omits_optional_batch_options` sends a
single typed Get without optional batch options. It checks the typed response,
response header TTLV, metadata, Unique Batch Item ID, and absent pending
correlation value through the installed ABI. The local matrix reports all 10
cells for this case invoked and no pending cells; complete language parity and
the full route matrix remain open.
