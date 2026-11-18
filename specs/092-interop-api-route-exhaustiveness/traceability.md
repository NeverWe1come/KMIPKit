# KMIPKIT-0092 Traceability

| Requirement | Implementation target | Verification |
|---|---|---|
| KMIPKIT-0092-FR-001 | `interop/shared/coverage/route-capabilities.json`, route capability schema, `route_models.py`, and `coverage_matrix.py` | Source-backed route inventory binds typed attribute markers to exact elements, maps Register/Revoke request and batch-item constructors for every language, covers the Batch Error Continuation Option builder across four languages (§§8.2, 9.6, 11.5), limits batch result readers to KMIP 2.0/2.1 response Batch Count and Batch Item fields (§§8.5–8.6, 9.5, 9.7), keeps §11.34 Object Type Enumeration targets separate from §4.36 Object Type Attribute routes, prevents attribute wrappers from claiming managed-object structures, scopes Secret Data constructor/parser routes to their own object tree and excludes Cryptographic Algorithm and Cryptographic Length as required by KMIP 2.0/2.1 §3.1 Table 11, scopes Key Wrapping Data setters to their exact §3.3 Tables 14–17 member and prevents Encryption Key Information and MAC/Signature Key Information Cryptographic Parameters routes from crossing sibling structures, limits Rust `KeyMaterial::byte_string` to the generic Key Material tag because KMIP 2.0/2.1 §11.25 distinguishes ByteString and Structure formats, excludes checked-`Tag` constructors and Item-view accessors only when reserved/unused allocations cannot yield an Item, retains RawTag construction and decoder-rejection rows, and binds async operations to exact public executors; installed artifact probes for all four languages |
| KMIPKIT-0092-FR-002 | `coverage_matrix.py`, `validate_route_evidence.py`, and language CI follow-ups | Negative tests reject missing target/version/route/action cells, duplicate or unplanned evidence, and incorrect scenario axes; expected NotSent cases require zero transport calls and omit impossible response decoding; async request cells use per-operation senders; actual full language CI is pending |
| KMIPKIT-0092-FR-003 | Per-language recipe and local TTLV suites | Java: Java invokes the pinned installed TTLV constructors, item creation/views, decoders, and typed attribute wrappers for the previously catalogued enum and bitmask cases; complete local route matrices are verified for all 19 Block Cipher Mode values across KMIP 2.0/2.1 (§§4.16, 11.6), all three Rotate Name Type values across KMIP 2.1 (§11.51), all 71 Recommended Curve values through four generic routes (§11.44 in 2.0, §11.45 in 2.1), all 78 Result Reason values through four generic routes (§11.45 in 2.0, §11.46 in 2.1, with three 2.1-assigned values also round-tripped as unknown in 2.0), all 60 Operation values through four generic routes (§11.36 in 2.0/2.1, with six 2.1-assigned values also round-tripped as unknown in 2.0), additionally, every Operation obligation in the selected Java cases exercises the §8.2 Operation field through generic Item create/view/decode routes, including object and payload cases and ordered batch steps with workflow identity; this adds 2,883 cells beyond the 60 Operation-value cases, and claims local TTLV behavior only (§11.36), all 24 Key Format Type values through four generic and three typed attribute routes plus the controlled Raw response accessor (§§8.6, 9.7, 11.25), all 20 NIST Key Type values through four generic and three typed attribute routes (§§4.34, 11.32 in KMIP 2.0/2.1, with tag allocation §11.54 in 2.0 and §11.56 in 2.1), all 25 Key Role Type values through four generic routes and the Cryptographic Parameters builder (§§4.16, 11.26 in KMIP 2.0/2.1), all 18 Hashing Algorithm values through four generic routes and the Cryptographic Parameters builder (§§4.16, 11.21 in KMIP 2.0/2.1), all 15 Link Type values through four generic routes (§11.28 in KMIP 2.0/2.1), all 11 Padding Method values through four generic routes and the Cryptographic Parameters builder (§§4.16, 11.37 in KMIP 2.0/2.1), all 20 Digital Signature Algorithm values through four generic, three typed attribute, and Cryptographic Parameters builder routes (§§4.16, 11.16), Opaque Data Type values through four generic and three typed attribute routes (§11.35), Processing Stage and Ticket Type values through all generic routes (§§11.40, 11.55/11.57), Mask Generator values through all generic routes and the Cryptographic Parameters builder (§11.30), nine additional enum families through all four generic TTLV routes (Encoding Option §11.18, Endpoint Role §11.19, Key Value Location Type §11.27, Key Wrap Type §11.29, Name Type §11.31, Object Group Member §11.33, Put Function §§11.42/11.43, Secret Data Type §§11.50/11.52, Usage Limits Unit §§11.58/11.60), all 57 Cryptographic Algorithm values through four generic routes, three typed attribute routes, and the Cryptographic Parameters builder (§§4.16, 11.12), and all 43 Profile Name targets through four generic routes (§11.40 in 2.0; §11.41 in 2.1); the individual reserved-range target also completes four generic routes in both versions using the catalogued lower boundary. A catalog-derived assertion now requires local generic fixtures for every eligible aggregate enum case with a same-named standalone tag (58 case IDs); this latest increment adds 1,904 cells. Local recipes additionally complete all applicable generic routes for 264 Tag cases (including 20 reserved tags) and 189 referenced PKCS#11 values (§§11.38-11.39, §11.54 (2.0), §11.56 (2.1)); both PKCS#11 aggregate cases now independently record their 1,512 value/route/version cells. Activation Date and Query have controlled-peer recipes in KMIP 2.0/2.1. All 44 per-value bitmask cases across Cryptographic Usage Mask, Protection Storage Mask, and Storage Status Mask now complete their applicable local generic and typed routes (592 value/route/version cells; §§12.1–12.3). The KMIP 2.1 Object Group structure member now has all three applicable local generic Item routes, with optional and repeated Text String values checked under §§4.35 Tables 99–100 and 7.23 Table 375. Four KMIP 2.1 members of Constraint and Constraints now exercise the installed generic Structure constructor, Item create/view, and decoder routes; tests cover the optional §7.6 fields and the repeatable §7.7 Constraint member (Tables 357–358). The two KMIP 2.1 Asynchronous Correlation Value members exercise all four applicable generic ByteString routes; §7.1 also invokes `QueryAsyncRequestsRequest.withCorrelationValues` and checks the optional filter, repeated values, order, and tags (§§7.1–7.2, Tables 352–353; §11.56 Table 487). The three required members of the KMIP 2.1 Asynchronous Request Structure exercise generic Enumeration or DateTimeExtended constructors, Item creation/view, and decoder routes; the outer Structure also completes six generic Structure/Item/decoder cells through TtlvStructure.create, TtlvStructure.withItem, TtlvValue.structure, TtlvItem.create, TtlvItem.view, and TtlvCodec.decode (§7.2 Table 353); the KMIP 2.1 Object Groups and Object Types Structures also complete those six generic routes each with their repeatable Text String and Enumeration members (§§7.23–7.24, Tables 375–376); Data Length, Final Indicator, Interop Identifier, Log Message, MAC Data, Authenticated Encryption Additional Data, Authenticated Encryption Tag, and Init Indicator each complete six local generic routes in KMIP 2.0/2.1 using exact Integer, Boolean, Text String, or Byte String values; the optional Authenticated Encryption fields are checked both present and absent (KMIP 2.0 §§7.1–7.2, 7.6, 7.10, 7.12–7.13, 7.15–7.16; KMIP 2.1 §§7.3–7.4, 7.10, 7.14, 7.16–7.17, 7.19–7.20; tag allocations §§11.54/11.56); Correlation Value completes six generic routes in both protocol versions and invokes the typed `ClientBatchItemResult.pendingCorrelationValue` accessor against a controlled pending Query response (KMIP 2.0 §7.4, KMIP 2.1 §7.8; tag allocations §§11.54/11.56); Extension Information completes six generic Structure/Item routes and the typed `ExtensionInformation.toTtlv` route in both versions, checking all seven fields and exact values (KMIP 2.0 §7.9 Table 329, KMIP 2.1 §7.13 Table 365; tag allocations §§11.54/11.56); Profile Information and Profile Version each complete six generic routes in KMIP 2.0/2.1, and the five Profile Information members complete constructor/Item/view/decode routes in both versions (64 cells; KMIP 2.0 §§7.25–7.26 Tables 345–346; KMIP 2.1 §§7.31–7.32 Tables 383–384), while the installed controlled Query consumer invokes and asserts `QueryResponse.profileInformation()`; Server Information adds 90 KMIP 2.0/2.1 cells across the outer Structure and all nine Text String members plus the Vendor-Specific Any member (KMIP 2.0 §7.31, KMIP 2.1 §7.37 Table 389), and the controlled mTLS Query invokes and validates `QueryResponse.serverInformations()`; a mutual-TLS Query Asynchronous Requests exchange invokes `AsyncOperationOutcome.responsePayloadTtlv` and verifies the typed response Structure, field tags, order, and values (§7.2 Table 353; §11.56 Table 487). The current ledger records 26,921 executed and 44,728 pending cells of 71,649; the Java selected scope completes Ephemeral, all 12 catalogued Boolean message-field and structure-member cases through applicable generic routes, typed Request/Response Message parsing for Criticality Indicator, Attestation Capable Indicator and Batch Order Option in a valid two-item request, and the Cryptographic Parameters builder, typed attribute, and AttributeSet routes for Random IV (§§4.16, 9.3, 9.8, 9.13); the Item Type aggregate and all 11 individual value cases complete their four generic TTLV Enumeration routes in both versions after `TtlvItemType.fromCode` verifies each catalog code (§11.23); it also includes all 20 cells for each of the six versioned Add/Modify/Set Attribute New Attribute payload cases. The remaining 40,716 cells across 1,108 case IDs all have `route_evidence_not_emitted`; they remain planned by the catalog/capability matrix and are unresolved required coverage under FR-002/FR-009, not justified exclusions or API limitations; the `test-enum-tag` aggregate now covers all 3,674 generic TTLV cells, while 50 individual Tag cases retain 652 pending typed-route cells; remaining values, fields, batches, async cases and boundaries remain pending. The client-initiated KMIP 2.0 Discover Versions typed exchange is also pending because the installed response converter checks against its default KMIP 2.1 offer; this is a KMIPKit correctness issue, not a server compatibility result. Java CI currently validates its declared selected scope and compares the full generated ledger; it does not require `complete_matrix=true`, so a green selected-scope step is not feature completion. Commands and Red/Green/Refactor evidence are in `java-route-expansion-evidence.md`; Rust: Rust directly executes `KeyBlock::with_key_value`, `KeyBlock::with_key_compression_type`, `KeyBlock::with_key_wrapping_data`, `KeyValue::try_new`, `KeyValue::try_from_item`, all three `KeyMaterial` constructors/parsers, and the Key Wrapping Data, Encryption Key Information, and MAC/Signature Key Information constructors and raw/typed Cryptographic Parameters setters for every applicable manifest target with local TTLV roundtrips; the full Rust matrix and remaining languages remain pending |
| KMIPKIT-0092-FR-004 | `validate_report()`, `validate_parity_collection()`, route evidence schema, language peer fixtures | Constructed and sent request trees must match; tree hashes are compared across routes and languages; batch events retain ordered operation names and peer-observed response counts; full language events are pending |
| KMIPKIT-0092-FR-005 | Controlled KMIP peer contracts in each language validator | Pending until every supported network row is exercised and the peer checks request/response semantics and cleanup |
| KMIPKIT-0092-FR-006 | Language artifact installers, route manifest, and evidence validator | Exact `release/1.0.0` commit and per-language installed artifact digest are checked for all four artifacts; comparable runtime reports are pending |
| KMIPKIT-0092-FR-007 | `validation-results.schema.json`, route evidence schema, aggregator, parity validator | Per-run output retains transport, encoding, API, construction route, execution route, and artifact axes; tests cover same-server/config comparisons |
| KMIPKIT-0092-FR-008 | Report writers and redaction helpers | Secret and raw-message negative fixtures |
| KMIPKIT-0092-FR-009 | Capability manifest and per-language validator route registries | Common matrix fails missing target routes unless the exact public API limitation is source-backed; Secret Data constructor and parser target sets are limited to their own object subtree and omit the two Key Block fields excluded by KMIP 2.0/2.1 §3.1 Table 11; KMIP 2.0 Discover Versions remains a pending correctness route despite the public request factory existing; per-language recipe gates are pending |
| KMIPKIT-0092-FR-010 | Pinned OASIS inventory, catalog generator, and normative inventory tests | Source-backed inventory tests cover versioned operations, objects, attributes, TTLV types, enum values, flags, field branches, and defined boundary/error classes |

The common contract establishes planned obligations, schema behavior, and
artifact availability. It does not claim that any validator has executed the
full matrix. FR-003, FR-005, and the runtime portion of FR-006 remain open until
the four independent language branches emit evidence from actual public API
calls and controlled-peer exchanges.

The common serializer/response-reader route assertion loads the API manifest
from the exact `library_commit` recorded in `route-capabilities.json`, so
functions added to a later release cannot be required from an older installed
artifact. This is part of the FR-006 artifact provenance check.

## Java selected-scope update — 2026-11-17

- **FR-001/FR-009:** The Java async response-payload accessor is now mapped to
  the observed KMIP 2.1 Asynchronous Request structure (§7.2 Table 353). The
  Operation member inside that structure is not mapped to Message Field 9.15.
  The shared route model also recognizes restricted Text String source
  encodings when selecting the corresponding generic constructors.
- **FR-003/FR-005:** Controlled mTLS evidence invokes the Java response-payload
  accessor for Query Asynchronous Requests in KMIP 2.1 and Cancel in KMIP 2.0
  and 2.1. The client batch builder's Batch Error Continuation Option is
  verified on a two-item request in both protocol versions, and the returned
  Unique Batch Item ID accessor is exercised against controlled responses.
  Scalar request/response message fields and typed message parsers are also
  covered by valid 2.0/2.1 fixtures.
- The selected Java report validates 27,135/27,135 invoked cells with zero
  failures. The generated ledger has 71,641 planned, 27,135 executed, and
  44,506 pending cells. This is selected-scope evidence only; FR-002, FR-003,
  FR-005, and the runtime portion of FR-006 remain open until the full matrix
  and four-language peer runs are complete.
- **FR-003/FR-005:** Tag `0x42000E` now has Java per-case `RawTag.fromRaw`,
  `RawTag.checked`, and `ClientBatch.addErrorContinuationValue` evidence in
  KMIP 2.0/2.1. The typed setter is tied to the observed serialized header of
  the controlled two-item Query request (OASIS KMIP 2.0 §11.54; KMIP 2.1
  §11.56).
- The updated selected report validates 27,147/27,147 invoked cells with zero
  failures. The ledger records 71,641 planned, 27,147 executed, and 44,494
  pending cells. This remains selected-scope evidence; full-matrix closure is
  still required.
- **FR-003/FR-005:** The Java KMIP 2.1 Ping case now invokes the installed
  `ClientBatchItem.ping()` factory and `ClientBatchItemResult.pingResponse()`
  against a controlled mTLS peer. The captured request confirms Operation
  Ping, and the response is successful. Its generic TTLV Operation Enumeration
  is also constructed, viewed, and decoded under §6.1.36.
- The latest selected Java report validates 27,154/27,154 cells with zero
  failures. Its generated ledger records 71,641 planned, 27,154 executed, and
  44,487 pending cells. This remains partial selected-scope evidence; full
  Java matrix closure and four-language peer runs are still required.
- **FR-003/FR-005:** The Java local recipe now covers the Set Endpoint Role
  request field and its Operation Enumeration with generic TTLV item creation,
  view, and decode, under KMIP 2.1 §6.1.54 Table 331, §11.19, and §11.36.
- The latest selected report validates 27,160/27,160 cells with zero failures.
  Its generated ledger records 71,641 planned, 27,160 executed, and 44,481
  pending cells. Full-matrix closure remains required.
- **FR-003/FR-005:** KMIP 2.0 Set Endpoint Role now has Java generic TTLV
  creation, Item view, and decoder evidence for both required Endpoint Role
  request and response fields, plus the Operation Enumeration, under OASIS
  KMIP 2.0 §6.1.48 Tables 299–300, §11.19, and §11.36.
- The selected report validates 27,172/27,172 Java cells with zero failures.
  Its generated ledger records 71,641 planned, 27,172 executed, and 44,469
  pending cells across 1,231 pending case IDs. Full-matrix closure remains
  required.


## Java selected-scope update — 2026-11-18

- **FR-003:** The Java fixture now covers every Operation obligation in the selected catalog cases through generic Item creation, view, and decoding. It preserves workflow step, operation, and item identity for batch targets, and reuses Operation evidence emitted by other local recipes. Local enum and bitmask fixture operations remain descriptive context unless they are separately catalogued as Operation targets.
- The selected Java report validates 33,886/33,886 cells with zero failures. The generated ledger records 71,641 planned, 33,886 executed, and 37,755 pending cells. The Usage Limits structure/member routes remain included in #216, and the eight Right Usage Limits member cells remain included in #217. The selected scope is complete, but the Java full matrix is not; other languages and the full controlled-peer matrix remain open under FR-002, FR-003, FR-005, and FR-006.

## Java managed-object Object Type routes — 2026-11-18

- **FR-003:** For all nine catalogued managed-object cases, the Java controlled consumer invokes the installed `TtlvItem.create`, `TtlvItem.view`, and `TtlvCodec.decode` routes for the Object Type Enumeration in KMIP 2.0 and 2.1 (§11.34). Each fixture checks the catalogued wire value and `Object Type` tag. This closes 54 local cells and makes no Register/Create transport claim.
- The selected Java report validates 33,994/33,994 cells with zero failures; the generated ledger records 71,641 planned, 33,994 executed, and 37,647 pending cells. Full Java matrix closure remains required under FR-002/FR-003/FR-009.

## Java RSA public Key Material model routes — 2026-11-18

- **FR-003:** The installed Java consumer now builds and reads the RSA public Key Material structure with both typed constructors, `KeyMaterial.fromFields` and `KeyMaterial.fromItem`, for the catalogued Modulus and Public Exponent fields in KMIP 2.0 and 2.1 (§3.8). A deterministic synthetic fixture is checked through the typed readers; it is not production key material and does not claim cryptographic validity.
- On the #187 branch, the initial RSA public-key slice adds eight cells to 33,862, and the transparent-format expansion adds 108 more, producing 33,978 selected cells. After syncing through #216, the report validates 33,994/33,994 cells and records 37,647 pending of 71,641, including Usage Limits coverage. Full Java matrix closure remains required under FR-002/FR-003/FR-009.

## Java transparent Key Material model routes — 2026-11-18

- **FR-003:** The installed Java consumer exercises `KeyMaterial.fromFields` and `KeyMaterial.fromItem` for all 29 catalogued Big Integer members in transparent DSA, RSA, and DH private/public plus EC private formats (KMIP 2.0/2.1 §§3.5–3.11). Each fixture includes the format's required fields and applicable optional fields and verifies the target member through both typed readers. Values are synthetic and not tested as cryptographic keys.
- The selected report validates 33,994/33,994 cells with zero failures and records 37,647 pending of 71,641. FR-002/FR-003/FR-009 remain open for full Java matrix closure and the remaining languages.

## Java typed Key Value and Key Block routes — 2026-11-18

- **FR-003:** The Java controlled consumer now invokes `KeyValue.fromFields`,
  `KeyValue.fromItem`, `KeyBlock.fromFields`, and `KeyBlock.fromItem` for all
  29 catalogued Big Integer members in the transparent DSA, RSA, DH, and EC
  Key Material formats, for KMIP 2.0 and 2.1 (§§3.5–3.11). Each typed result is
  read back through nested typed accessors and checked against the target
  synthetic field.
- This adds 232 cells to the #187 Java selected scope. The combined #216 report
  validates 34,226/34,226 cells with zero failures and leaves 37,415 of 71,641
  pending. Full Java matrix closure remains open under FR-002/FR-003/FR-009.

## Java Right Usage Limits stack verification — 2026-11-18

- **FR-003:** The #217 controlled consumer retains all eight generic Structure,
  Item creation, Item view, and decode cells for the optional Right Usage
  Limits member in KMIP 2.0 and 2.1 (§7.28 and §7.34). The evidence uses the
  exact catalog target and confirms all four routes in both versions.
- The full Java suite passes 84 tests. The #217 selected report validates
  34,226/34,226 cells with zero failures and leaves 37,415 of 71,641 pending;
  full Java matrix closure remains open under FR-002/FR-003/FR-009.
