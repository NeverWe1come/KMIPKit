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
- The selected Java report validates 33,808/33,808 cells with zero failures. The generated ledger records 71,641 planned, 33,808 executed, and 37,833 pending cells. The selected scope is complete, but the Java full matrix is not; other languages and the full controlled-peer matrix remain open under FR-002, FR-003, FR-005, and FR-006.

## Java managed-object Object Type routes — 2026-11-18

- **FR-003:** For all nine catalogued managed-object cases, the Java controlled consumer now invokes the installed `TtlvItem.create`, `TtlvItem.view`, and `TtlvCodec.decode` routes for the Object Type Enumeration in KMIP 2.0 and 2.1 (§11.34). Each fixture checks the catalogued wire value and `Object Type` tag. This closes 54 local cells and makes no Register/Create transport claim.
- The selected Java report validates 33,978/33,978 cells with zero failures; the generated ledger records 71,641 planned, 33,978 executed, and 37,663 pending cells. Full Java matrix closure remains required under FR-002/FR-003/FR-009.

## Java RSA public Key Material model routes — 2026-11-18

- **FR-003:** The installed Java consumer now builds and reads the RSA public Key Material structure with both typed constructors, `KeyMaterial.fromFields` and `KeyMaterial.fromItem`, for the catalogued Modulus and Public Exponent fields in KMIP 2.0 and 2.1 (§3.8). A deterministic synthetic fixture is checked through the typed readers; it is not production key material and does not claim cryptographic validity.
- The selected report validates 33,978/33,978 cells with zero failures; the generated ledger records 71,641 planned, 33,978 executed, and 37,663 pending cells. The full Java matrix and the other language matrices remain open under FR-002/FR-003/FR-005/FR-006.

## Java transparent Key Material model routes — 2026-11-18

- **FR-003:** The installed Java consumer now exercises `KeyMaterial.fromFields` and `KeyMaterial.fromItem` for all 29 catalogued Big Integer members in transparent DSA, RSA, and DH private/public plus EC private formats (KMIP 2.0/2.1 §§3.5–3.11). Each fixture includes the format's required fields and applicable optional fields and verifies the target member through both typed readers. Values are deterministic, synthetic, and not tested as cryptographic keys.
- This adds 108 cells to the previous Java selected scope. The report validates 33,978/33,978 cells with zero failures and leaves 37,663 of 71,641 pending. FR-002/FR-003/FR-009 remain open for full Java matrix closure and the remaining languages.

## Java typed Key Value and Key Block routes — 2026-11-18

- **FR-003:** The Java controlled consumer now invokes `KeyValue.fromFields`,
  `KeyValue.fromItem`, `KeyBlock.fromFields`, and `KeyBlock.fromItem` for all
  29 catalogued Big Integer members in the transparent DSA, RSA, DH, and EC
  Key Material formats, for KMIP 2.0 and 2.1 (§§3.5–3.11). Each typed result is
  read back through nested typed accessors and checked against the target
  synthetic field.
- This adds 232 cells. The selected report validates 34,210/34,210 cells with
  zero failures; 37,431 of 71,641 remain pending. Full Java matrix closure
  remains open under FR-002/FR-003/FR-009.

## Java EC public Key Material model routes — 2026-11-18

- **FR-003:** The installed Java consumer now constructs EC public Key Material
  with the catalogued `Recommended Curve` and `Q String` fields and exercises
  `KeyMaterial`, `KeyValue`, and `KeyBlock` through both `fromFields` and
  `fromItem` in KMIP 2.0 and 2.1 (§§3.11–3.12). The deterministic point bytes
  are synthetic fixture data and do not claim cryptographic key validity.
- This adds 36 cells. The selected report validates 34,246/34,246 cells with
  zero failures; 37,395 of 71,641 remain pending. Full Java matrix closure
  remains open under FR-002/FR-003/FR-009.

## Java Transparent Symmetric Key model routes — 2026-11-18

- **FR-003:** The installed Java consumer now constructs the §3.4 `Key` Byte
  String through `KeyMaterial.fromFields` and `KeyMaterial.fromItem`, and also
  exercises `KeyValue` and `KeyBlock` through both typed constructors for KMIP
  2.0 and 2.1 (§§3.4, 11.25). The four key bytes are deterministic synthetic
  fixture data, not production key material.
- This adds 12 cells. The selected report validates 34,258/34,258 cells with
  zero failures; 37,383 of 71,641 remain pending. Full Java matrix closure
  remains open under FR-002/FR-003/FR-009.

## Java typed Key Material structure routes — 2026-11-18

- **FR-003:** The installed Java consumer now creates and reads all nine
  catalogued transparent Key Material Structures through `KeyMaterial`,
  `KeyValue`, and `KeyBlock` using both `fromFields` and `fromItem`, for KMIP
  2.0 and 2.1 (§§3.4–3.12). Fixtures use the format's source-defined fields;
  their deterministic synthetic values are not cryptographic key tests.
- This adds 108 cells. The selected report validates 34,366/34,366 cells with
  zero failures; 37,275 of 71,641 remain pending. Full Java matrix closure
  remains open under FR-002/FR-003/FR-009.

## Java ManagedObject Key Material routes — 2026-11-18

- **FR-003:** The installed Java consumer now invokes `ManagedObject.fromFields`
  and `ManagedObject.fromItem` for each catalogued transparent Key Material
  field and structure. The test wraps each format in the matching Symmetric
  Key, Private Key, or Public Key object structure, then reads the nested Key
  Block and verifies its format and target field (KMIP 2.0/2.1 §§2.5, 2.6, 2.9,
  3.4–3.12). Values are deterministic synthetic fixtures and do not claim
  cryptographic validity.
- This adds 168 cells. The selected report validates 34,534/34,534 cells with
  zero failures and leaves 37,107 of 71,641 pending. Full Java matrix closure
  remains open under FR-002/FR-003/FR-009.

## Java typed ManagedObject key factories — 2026-11-18

- **FR-003:** The installed Java consumer now constructs every transparent Key
  Material fixture through the matching `ManagedObject.symmetricKey`,
  `ManagedObject.privateKey`, or `ManagedObject.publicKey` factory, then reads
  back and verifies its nested Key Block. The fixture covers KMIP 2.0/2.1
  object structures and Key Material formats (§§2.5–2.9, 3.1–3.12); fixture
  values are synthetic and do not claim cryptographic validity.
- This adds 84 cells. The selected report validates 34,618/34,618 cells with
  zero failures and leaves 37,023 of 71,641 pending. Full Java matrix closure
  remains open under FR-002/FR-003/FR-009.

## Java Certificate managed-object routes — 2026-11-18

- **FR-003:** The installed Java consumer exercises
  `ManagedObject.certificate` for the Certificate Object Structure, Certificate
  Type, Certificate Value, and Certificate tag cases in KMIP 2.0 and 2.1.
  `ManagedObject.fromFields` and `ManagedObject.fromItem` run for the three
  structure/member cases where the generated matrix declares them applicable
  (KMIP 2.0/2.1 §2.1).
- This adds 20 cells. The selected report validates 34,638/34,638 cells with
  zero failures and leaves 37,003 of 71,641 pending. Full Java matrix closure
  remains open under FR-002/FR-003/FR-009.

## Java Certificate Request and Opaque Object routes — 2026-11-18

- **FR-003:** The installed Java consumer exercises
  `ManagedObject.certificateRequest` and `ManagedObject.opaqueObject`, as well
  as `fromFields` and `fromItem` for each catalogued structure/member target
  where the capability matrix declares those routes applicable. The tests
  verify the object type and typed enum/value fields for KMIP 2.0 and 2.1
  (§§2.1–2.3).
- This adds 40 cells. The selected report validates 34,678/34,678 cells with
  zero failures and leaves 36,963 of 71,641 pending. Full Java matrix closure
  remains open under FR-002/FR-003/FR-009.

## Java key-bearing managed-object root routes — 2026-11-18

- **FR-003:** The installed Java consumer now executes the Symmetric Key,
  Private Key, and Public Key root cases through the matching named factory,
  `ManagedObject.fromFields`, and `ManagedObject.fromItem` in KMIP 2.0 and 2.1.
  Each uses a deterministic Raw Key Block fixture and verifies the parsed
  object type and Key Block presence (KMIP §§2.5, 2.6, 2.9, 3.1, 3.2, 11.25).
- The selected report validates 34,696/34,696 cells with zero failures; the
  full Java ledger records 71,641 planned, 34,696 executed, and 36,945 pending
  cells. Full-matrix closure remains required under FR-002/FR-003/FR-009.
- **FR-007:** Generated ledger case IDs are sorted, with a regression test
  confirming output stability when the declared input order varies. The
  regenerated reference and comparison now pass with the 34,696-cell Java
  report.

## Java Split Key managed-object root routes — 2026-11-18

- **FR-003:** The installed Java consumer now exercises the Split Key root
  through `ManagedObject.splitKey`, `ManagedObject.fromFields`, and
  `ManagedObject.fromItem` in KMIP 2.0 and 2.1. The fixture uses Table 9 field
  order, the assigned XOR method, valid part/threshold counts, and a Raw Key
  Block, then checks the typed accessors and object type (KMIP §§2.8, 3.1, 11.34).
- This adds six local model cells. The selected report validates 34,702/34,702
  cells with zero failures; the Java ledger records 71,641 planned, 34,702
  executed, and 36,939 pending cells. Full matrix closure remains open under
  FR-002/FR-003/FR-009.

## Java Split Key managed-object member routes — 2026-11-18

- **FR-003:** The installed Java consumer executes the named Split Key factory
  for all six §2.8 Table 9 members and the Split Key tag; `fromFields` and
  `fromItem` execute for each applicable structure member in KMIP 2.0/2.1.
  Fixtures preserve Table 9 order and include Prime Field Size only with
  Polynomial Sharing Prime Field (KMIP §§2.8, 3.1, 11.34).
- The 38 new route events cover six member cases through three factories and
  the tag through its named factory in each protocol version. The selected
  report validates 34,740/34,740 cells with zero failures; the full Java
  ledger records 71,641 planned, 34,740 executed, and 36,901 pending cells.
  Full matrix closure remains open under FR-002/FR-003/FR-009.

### Java Get Attributes request payload-field route evidence — 2026-11-18

- **FR-001/FR-003/FR-005/FR-009:** `ClientBatchItem.getAttributes` and
  `AttributeRequest.getAttributesToBatchItem` now execute the catalogued
  Unique Identifier present/omitted and Attribute Reference absent/single/
  repeated branches over every approved encoding/transport pair. Omitted IDs
  resolve through an earlier Create batch item. Controlled mutual-TLS peers
  inspect the request, and the client validates the typed response (OASIS KMIP
  2.0/2.1 §6.1.20, §11.58, §§8.3, 8.6).
- Red `daf05f7df` requires evidence for every payload branch and network pair;
  Green `3c41ea623` exercises the public routes and controlled exchanges.
- The selected report validates 40,197/40,197 cells with zero failures. The
  ledger records 71,641 planned, 40,197 executed, and 31,444 pending cells.
  The Java suite passes 92/92, the shared suite passes 196/196, ledger
  comparison passes, and the capability audit accepts 1,507 routes. Full
  matrix closure remains open under FR-002/FR-003/FR-009.

## Java PGP Key and Secret Data managed-object routes — 2026-11-18

- **FR-003:** The installed Java consumer exercises the PGP Key and Secret
  Data root structures, required members, and assigned object tags through the
  matching named factories; `fromFields` and `fromItem` run where applicable.
  The deterministic fixtures include PGP Key Version or Secret Data Type plus
  a Raw Key Block (KMIP §§2.4, 2.7).
- This adds 40 local route cells. The selected report validates 34,780/34,780
  cells with zero failures; the full Java ledger records 71,641 planned,
  34,780 executed, and 36,861 pending cells. Full matrix closure remains open
  under FR-002/FR-003/FR-009.

## Java generic Key Block member routes — 2026-11-18

- **FR-003:** Catalogued `Object Data Structure` Key Block members for PGP,
  Private, Public, Secret Data, and Split Key now execute generic Item creation,
  view, and decode through the installed Java package under KMIP 2.0/2.1
  (KMIP §§2.4–2.8, 3.1). `TtlvValue.structure` is recorded only for targets
  declared by the route manifest.
- Selected evidence validates 34,816/34,816 cells with zero failures; 36,825
  of 71,641 remain pending. Full matrix closure remains open under
  FR-002/FR-003/FR-009.

## Java generic Operation routes for KMIP 2.0 version boundaries — 2026-11-18

- **FR-003/FR-009:** The six KMIP 2.0 rejection cases for operations introduced
  in 2.1 execute generic Operation Item construction, view, and decode using
  the source-defined 2.1 enumeration value in a local 2.0-context fixture.
  This exercises TTLV representation only and does not send an unsupported
  request.
- Selected evidence validates 34,834/34,834 cells with zero failures; 36,807
  of 71,641 remain pending. Full matrix closure remains open under
  FR-002/FR-003/FR-009.

## Java generic Batch Item routes for batch workflows — 2026-11-18

- **FR-003/FR-009:** Generic Item construction, view, and decode now execute
  for all catalogued batch-workflow Request Batch Item and Response Batch Item
  targets in KMIP 2.0 and 2.1 (OASIS §§8.3, 8.6). The fixture validates the
  generic TTLV representation; it does not claim that it executes workflow
  behavior.
- Red `fb43d7432` failed on missing case/target/version/route cells; Green
  `be6a84fd4` adds the fixture calls and report events; Refactor `e39b24fc1`
  asserts the exact evidence-key set.
- The full Java suite passes 92/92; shared evidence suites pass 109/109. The
  selected report validates 34,984/34,984 cells with zero failures. The ledger
  records 71,641 planned, 34,984 executed, and 36,657 pending cells. Full
  matrix closure remains open under FR-002/FR-003/FR-009.

## Java Get Attributes typed request routes — 2026-11-18

- **FR-003/FR-009:** Both installed typed Get Attributes builder routes now
  execute for all catalogued Get Attributes cases in KMIP 2.0/2.1 controlled
  TTLV exchanges. The fixture checks the corresponding typed response under
  OASIS KMIP 2.0/2.1 §6.1.20 and §§8.3, 8.6. It covers operation-level route
  execution, not each attribute's independent semantic policy.
- Red `51dd2f2a9` failed on missing route evidence; Green `8b5d8a7f6` added the
  controlled requests and response checks; Refactor `23e349af8` extracted the
  common assertions.
- Selected evidence validates 35,873/35,873 cells with zero failures. The
  full Java suite passes 92/92 and shared route tests pass 109/109. The ledger
  matches its reference and the capability audit accepts 1,507 public routes.
  It records 71,641 planned, 35,873 executed, and 35,768 pending cells. Full
  matrix closure remains open under FR-002/FR-003/FR-009.

## Java Get Attributes encoding and transport routes — 2026-11-18

- **FR-001/FR-003/FR-005:** Both typed Get Attributes construction routes
  execute each catalogued case/version over TTLV, JSON, and XML using raw TLS
  and HTTPS. Controlled mTLS peers inspect requests and return matching
  encoded responses; assertions cover protocol version, Attribute References,
  batch identifiers, and the typed result (OASIS KMIP 2.0/2.1 §6.1.20 and
  §§8.3, 8.6).
- Red `eca46473a` catches route combinations missing from report metadata;
  Green `d6486c936` records exactly the combinations present in route events.
- The selected report validates 39,743/39,743 cells with zero failures. The
  ledger records 71,641 planned, 39,743 executed, and 31,898 pending cells.
  Full matrix closure remains open under FR-002/FR-003/FR-009.

## Java Get Attribute List encoding and transport routes — 2026-11-18

- **FR-001/FR-003/FR-005:** Both installed typed Get Attribute List builders
  execute every catalogued case/version over TTLV, JSON, and XML on raw TLS
  and HTTPS. Controlled mTLS peers inspect requests and return checked
  responses; the test validates the Unique Identifier and ordered Attribute
  References (OASIS KMIP 2.0/2.1 §6.1.21 and §§8.3, 8.6). Local evidence also
  invokes the typed request payload encoder and typed response reader for the
  catalogued operation and fields.
- Red `5631cd0fb` detects missing per-case route evidence; Green `b8992a5c2`
  exercises both public constructors and all six network combinations.
- The selected report validates 40,101/40,101 cells with zero failures. The
  ledger records 71,641 planned, 40,101 executed, and 31,540 pending cells.
  Full matrix closure remains open under FR-002/FR-003/FR-009.

### Java asynchronous Cancellation Result generic TTLV routes — 2026-11-18

- **FR-003/FR-009:** The five asynchronous Cancel result scenarios now use
  their catalogued Cancellation Result enum target and value with the
  Cancellation Result tag, then execute generic Item construction, view, and
  decode for KMIP 2.0 and 2.1. This is local generic TTLV evidence, not
  asynchronous network-lifecycle evidence. References: OASIS KMIP 2.0 §6.1.5
  Table 162, §11.7, §11.56; KMIP 2.1 §6.1.5 Table 177, §11.7, §11.56.
- Red `af7765d94` requires every result scenario/version to provide the three
  generic TTLV route events and fails on the missing `TtlvItem.create` event.
  Green `bb7b4d7b2` builds and round-trips each source-catalogued value.
- The selected report validates 40,227/40,227 cells with zero failures. The
  full ledger records 71,641 planned, 40,227 executed, and 31,414 pending
  cells. Java tests pass 92/92, shared tests 196/196, ledger comparison passes,
  and the capability audit accepts 1,507 routes. Full matrix closure remains
  open under FR-002/FR-003/FR-009.

### Java Asynchronous Indicator generic TTLV routes — 2026-11-18

- **FR-003/FR-009:** The four catalogued asynchronous-indicator scenarios
  resolve their enum target/value and exercise generic Item create, view, and
  decode in KMIP 2.0 and 2.1. The omitted parameter is checked as Prohibited;
  explicit Prohibited, Optional, and Mandatory are also covered. This does not
  claim asynchronous network-lifecycle coverage. References: OASIS KMIP
  2.0/2.1 §8.2, §§11.3 and 11.56.
- Red `a484aa932` requires all three local routes per case/version; Green
  `4f78ac602` performs catalog-driven construction and round trips.
- The selected report validates 40,251/40,251 cells with zero failures. The
  ledger records 71,641 planned, 40,251 executed, and 31,390 pending cells.
  Java tests pass 92/92, shared tests 196/196, ledger comparison passes, and
  the capability audit accepts 1,507 routes. The complete matrix remains open.

### Java Asynchronous Cancel request and response routes — 2026-11-18

- **FR-001/FR-003/FR-005/FR-009:** The five catalogued Cancellation Result
  cases execute Java `Client.executeCancel` for both protocol versions, all
  three encodings, and raw TLS/HTTPS against controlled mutual-TLS peers. The
  peer validates operation, version, and correlation; Java validates the
  decoded operation, success status, echo, and result. This covers the Cancel
  exchange, not a subsequent Poll lifecycle. References: OASIS KMIP 2.0/2.1
  §6.1.5 Tables 161, 162, 176, and 177, §§8.2, 8.3, 8.6, 11.7, and 11.56.
- Red `c5ec2ae4d` requires all per-case/version/encoding/transport targets and
  actions; Green `edb09424d` runs the 60 controlled exchanges and records their
  event evidence.
- The selected report validates 40,671/40,671 cells with zero failures. The
  ledger records 71,641 planned, 40,671 executed, and 30,970 pending cells.
  Java tests pass 92/92 and shared tests 196/196; ledger comparison passes and
  the capability audit accepts 1,507 routes. Full matrix closure remains open.

### Java asynchronous Cancel remaining route cells — 2026-11-18

- **FR-001/FR-003/FR-005/FR-009:** `Client.executeCancel` now has controlled
  request/response evidence for every catalog-applicable Cancel case, payload
  field, and asynchronous workflow case in KMIP 2.0/2.1 over TTLV, JSON, and
  XML on raw TLS and HTTPS. Assertions cover operation, version, correlation,
  result, and response shape (OASIS KMIP 2.0 §6.1.5 Tables 161/162 and KMIP
  2.1 §6.1.5 Tables 176/177; §§8.2, 8.3, 8.6, 11.7, 11.56). Race and
  recursion cases cover the Cancel exchange only, not a full Poll lifecycle.
- Red `58d8e2b90` requires the remaining route cells and fails on the missing
  ordinary `test-op-cancel` exchange; Green `d00cf8272` executes 120 controlled
  exchanges and adds 228 route cells.
- The Java suite passes 92/92; selected evidence validates 40,899/40,899
  cells with zero failures. The full ledger records 71,641 planned, 40,899
  executed, and 30,742 pending cells; ledger comparison and the 1,507-route
  capability audit pass. The Java matrix remains incomplete under FR-002,
  FR-003, and FR-009.

Java FR-003 also includes explicit local TTLV assertions for Usage Limits
Total, Count, and Unit in KMIP 2.0 and 2.1, including required field order,
tags, Long Integer/Enumeration types, and values (KMIP 2.0 §7.34 Table 354;
KMIP 2.1 §7.40 Table 392). This is local route evidence and does not claim
vendor KMS interoperability.

## Java typed Alternative Name routes — 2026-11-18

- **FR-003/FR-009:** The catalogued Alternative Name Structure now exercises
  `TypedAttribute.create`, `TypedAttribute.fromItem`, `AttributeSet.tryNew`,
  and `GetAttributesRequest.toTtlvPayload` in KMIP 2.0 and 2.1. The fixture
  checks source-defined member tags/types/values and the Attribute Reference
  request payload. References: OASIS KMIP 2.0/2.1 §4.2, §6.1.20 Tables
  208/223, §§8.3, 8.6.
  This is local typed-model and encoding evidence, not a server exchange or
  attribute-policy validation.
- Red `d1e35d5e8`, `322554a5a`, and `a5b006191` establish missing route,
  target-attribution, and request-hash failures. Green `3535eac0a`,
  `3fbfb09f3`, and `13bd4226b` implement and correct those paths.
- The Gradle suite passes 95/95, shared route/schema/capability tests pass
  109/109, and the capability audit accepts 1,507 routes. The selected report
  validates 42,204/42,204 cells with zero failures. The ledger records 71,641
  planned, 42,204 executed, and 29,437 pending cells; ledger comparison passes.
  Full matrix closure remains open under FR-002/FR-003/FR-009.

## Java Get Attributes local TTLV payload encoder matrix — 2026-11-18

- **FR-003/FR-009:** `GetAttributesRequest.toTtlvPayload` now has local encoder
  evidence for all 133 catalogued route cells across KMIP 2.0/2.1 operation
  cases and the four request-payload targets. Tests check the source-shaped
  payload, attribute references, and normalized request-tree hashes. Sources:
  OASIS KMIP 2.0 §6.1.20 Table 208; KMIP 2.1 §6.1.20 Table 223; §§8.3, 8.6.
- Red `dae729939` requires the missing encoder events; Green `73a07201e`
  produces the per-case and payload-target evidence; Refactor `c0e998a4f`
  centralizes event emission.
- The full Java suite passes 95/95, shared evidence/schema/capability tests
  109/109, traceability tests 33/33, and the capability audit accepts 1,507
  routes. Selected evidence validates 42,333/42,333 cells with zero failures;
  the ledger records 29,308 full-matrix cells still pending. Ledger comparison
  passes. FR-002/FR-003/FR-009 matrix closure remains open and the PR stack
  remains WIP.

## Java typed Boolean Attribute routes — 2026-11-18

- **FR-003/FR-009:** Nine catalogued Boolean Attributes now exercise the
  installed `TypedAttribute.create`, `TypedAttribute.fromItem`, and
  `AttributeSet.tryNew` routes in addition to generic TTLV construction,
  view, and decode. The fixtures check tag identity, Boolean value, and the
  resulting one-item Attribute Set. Sixteen catalog-applicable case/version
  pairs add 48 local route cells. Sources: OASIS KMIP 2.0/2.1 §§4.3, 4.23,
  4.24, 4.28, 4.33, 4.45, 4.48, 4.52, 4.55, and 11.56. This is local
  construction evidence, not server acceptance or attribute-policy evidence.
- Red `5f619e9ac` requires all seven generic and typed routes; Green
  `dfa28b4e5` executes them; Refactor `4d4abf496` isolates the typed Boolean
  path. The Gradle suite passes 95/95, shared route/schema/capability tests
  109/109, requirement traceability tests 2/2, feature traceability tests
  31/31, and the route capability audit accepts 1,507 routes. Selected
  evidence validates 42,381/42,381 cells with zero failures. The ledger
  records 71,641 planned, 42,381 executed, and 29,260 pending cells; ledger
  comparison passes. Full matrix closure remains open under FR-002/FR-003/
  FR-009 and the Java stack stays WIP.

## Java typed Date-Time Attribute routes — 2026-11-18

- **FR-003/FR-009:** All 12 selected Date-Time Attribute cases now exercise
  `TypedAttribute.create`, `TypedAttribute.fromItem`, and
  `AttributeSet.tryNew` alongside generic TTLV construction and decoding.
  Fixtures verify the assigned standard attribute, Date-Time value, and
  one-item Attribute Set. The cases span 23 applicable version/case pairs
  and close 63 previously pending cells. Sources: OASIS KMIP 2.0/2.1 §§4.1,
  4.5, 4.10, 4.11, 4.18, 4.20, 4.25, 4.29, 4.38, 4.40, 4.41, 4.49, and
  11.56. These checks establish local construction only, not server acceptance.
- Red `ad2717b75` requires all seven routes; Green `cfdc4b51d` emits them;
  Refactor `22ff6a4e5` shares typed route construction with Boolean Attributes.
  The Gradle suite passes 95/95, shared route/schema/capability tests 109/109,
  requirement traceability 2/2, feature traceability 31/31, and the route
  capability audit accepts 1,507 routes. Selected evidence validates
  42,444/42,444 cells with zero failures. The ledger records 71,641 planned,
  42,444 executed, and 29,197 pending cells; ledger comparison passes. Full
  matrix closure remains open under FR-002/FR-003/FR-009 and the Java stack
  stays WIP.

## Java typed Enumeration Attribute routes — 2026-11-18

- **FR-003/FR-009:** Nine catalogued Enumeration Attribute cases now exercise
  `TypedAttribute.create`, `TypedAttribute.fromItem`, and
  `AttributeSet.tryNew` alongside generic TTLV construction and decoding.
  Fixtures use catalog-selected representative enum values and verify the
  standard attribute, value, and one-item Attribute Set. This closes 54
  previously pending route cells. Sources: OASIS KMIP 2.0/2.1 §§4.7, 4.13,
  4.22, 4.26, 4.34, 4.36, 4.37, 4.42, 4.57, and 11.56. These are local
  construction checks, not server acceptance.
- Red `4be80061d` requires all seven routes; Green `8cdd1b967` emits them;
  Refactor `44a755f7c` shares applicable case/version counting across typed
  attribute tests. The Gradle suite passes 95/95; shared route/schema/
  capability tests pass 109/109; requirement/feature traceability passes 2/2
  and 31/31; the route capability audit accepts 1,507 routes. Selected
  evidence validates 42,498/42,498 cells with zero failures. The ledger
  records 71,641 planned, 42,498 executed, and 29,143 pending cells; ledger
  comparison passes. Matrix closure remains open under FR-002/FR-003/FR-009;
  the Java stack stays WIP.
