# KMIPKIT-0051 Normative Traceability

Normative identifiers in this feature use the prefix `KMIPKIT-0051-`. OASIS references below resolve to the immutable pinned KMIP 2.1 source. Each final rule row must be completed with its implementation path and executable test before the draft PR is considered complete.

| Requirement ID | Exact source | Validation obligation | Implementation / test |
|---|---|---|---|
| KMIPKIT-0051-FR-001 | KMIP v2.1 §6.1 operation-specific Request Payload tables; §8.3 Table 396 | Provide explicit schema-aware validation for typed client requests. | Pending implementation |
| KMIPKIT-0051-FR-002 | KMIP v2.1 §10.1.2 | Validate only the schema explicitly identified for a generic Structure; do not infer from tags. | Pending implementation |
| KMIPKIT-0051-FR-003 | KMIP v2.1 §6.1 operation-specific Request Payload tables; §10.1.2; §11.56 Table 487 and each field's defining table | Check field type/order/requiredness/cardinality and local conditions per structure description. | Pending field-level rules in the 56-row operation matrix |
| KMIPKIT-0051-FR-004 | KMIP v2.1 §6.1 payload tables; §8.2 Table 395; §8.3 Table 396; §9.11 Table 403 | Apply repeatability/conditionality per exact field table. | Pending implementation |
| KMIPKIT-0051-FR-005 | KMIP v2.1 §1.2; project API policy | Stable, value-free diagnostics; API-only codes are not represented as OASIS requirements. | Pending implementation |
| KMIPKIT-0051-FR-006 | KMIP v2.1 §8.1 Table 394; §8.2 Table 395; §8.3 Table 396; relevant §6.1 payload table per operation | Run validation before transport for every supported client operation. | Pending implementation |
| KMIPKIT-0051-FR-007 | Project delivery contract; existing `RequestDeliveryState` semantics | Pre-send validation failure returns `NotSent` with no exchange/write. | Pending implementation |
| KMIPKIT-0051-FR-008 | KMIP v2.1 §10.1.2–§10.1.3; project compatibility contract | Existing successful request encoding and one-exchange semantics remain unchanged. | Pending implementation |
| KMIPKIT-0051-FR-009 | KMIP v2.1 §§6.1, 8.2, 9.4; server-controlled policy boundary | Do not turn unknown server state/capability/policy into local rejection. | Pending implementation |
| KMIPKIT-0051-FR-010 | KMIP v2.1 §9.13 Table 418; KMIPKit lossless TTLV API contract | Preserve generic/extension data; unknown alone is not an invalidity claim. | Pending implementation |
| KMIPKIT-0051-FR-011 | Project secret-handling policy; KMIPKIT-0007-FR-013 | Validation must not clone, serialize, log, or report secret values solely to validate. | Pending implementation |
| KMIPKIT-0051-FR-012 | KMIP v2.1 §6.1 and §8.3; §6.1.54 Table 331; §6.2 | Inventory all 56 supported client-initiated values; separately record client-initiated Set Endpoint Role as deferred to 1.1 and exclude §6.2 server-initiated operations. | Pending 56-row operation matrix |
| KMIPKIT-0051-FR-013 | OASIS KMIP v2.1; project traceability policy | Link each normative field rule to source, code, and tests; identify API-only decisions separately. | `KMIPKIT-0051-R###` rows below; API decisions use `KMIPKIT-0051-P###` |
| KMIPKIT-0051-FR-014 | Project API compatibility policy | Add validation APIs without removing or changing existing request APIs. | Pending implementation |
| KMIPKIT-0051-FR-015 | Pinned KMIP v2.1 §11.56 Table 487 and each affected operation/response table | Correct any audited serializer/reader field tag that disagrees with its exact OASIS assignment, with tests that do not reuse the production constant as their oracle. | T020–T022; R119 and R166 in `gap-matrix.md`; implementation/test links added after Green |

### Success-criterion implementation traceability

| Criterion ID | Buildable outcome | Implementation/test evidence |
|---|---|---|
| KMIPKIT-0051-SC-001 | Inventory all 56 supported operation values and explicitly record the 1.1/client/server-direction exclusions. | T001–T003; `gap-matrix.md`; exhaustive operation-identity tests in `typed_request_validation_tests.rs`. |
| KMIPKIT-0051-SC-002 | Test known-schema type, order, requiredness, cardinality, and locally decidable conditions; do not imply server policy validation. | T007–T025; `validation_schema_tests.rs`, typed operation validation tests, and field rule rows below. |
| KMIPKIT-0051-SC-003 | Invalid pre-send requests return `NotSent` with zero transport exchanges/writes. | T026–T028; fake/scripted transport tests in the client test paths. |
| KMIPKIT-0051-SC-004 | Existing valid request TTLV bytes remain identical. | T028; before/after byte fixtures in `request_validation_tests.rs`. |
| KMIPKIT-0051-SC-005 | Reports, errors, and Debug output disclose no secret values or raw request bytes. | T029–T031; secret-canary and raw-body tests in `validation_redaction_tests.rs`. |
| KMIPKIT-0051-SC-006 | Every normative rule traces to exact OASIS source, code, and executable test. | T002 and T032; each `KMIPKIT-0051-R###` row below. |

### Source interpretation notes

- §10.1.2 encodes fields in the order specified for each Structure; it does not authorize sorting by tag number.
- §8.3 explicitly makes payload structure depend on Operation. Each request payload rule therefore cites its own §6.1 table; no global payload rule is substituted.
- `NotSent`, a Rust `validate()` method, and a pre-send client gate are KMIPKit behavior, not direct OASIS requirements.
- The rules for Cryptographic Parameters remain those already reviewed in KMIPKIT-0050 against §4.16 Table 59.

## Stable rule identifier scheme

- `KMIPKIT-0051-R###` identifies one normative schema or validation rule. The sequence is assigned once in field order in `gap-matrix.md` and is never reused. Each rule row must cite the exact pinned OASIS section/table, production validation path, and executable test.
- `KMIPKIT-0051-P###` identifies a KMIPKit API or transport policy decision that OASIS does not prescribe, including the public `validate()` surface, the unsupported-schema diagnostic, and pre-send `NotSent` behavior. These rows must not be described as OASIS requirements.
- Functional requirement rows (`KMIPKIT-0051-FR-###`) and success criteria remain separately mapped to the relevant rule and project-decision IDs.

| Rule ID | Exact OASIS source | Validation obligation | Implementation / test |
|---|---|---|---|
| `KMIPKIT-0051-R001`–`KMIPKIT-0051-R189` | See each exact source in `gap-matrix.md` and `conditions-audit.md` | Validate the field’s schema and only locally decidable OASIS conditions; operation-specific exclusions and server-dependent conditions are recorded alongside each source rule. | T002–T003 source/model map complete; T032 adds validator and focused test links per rule before closure |

| Project decision ID | Decision | Evidence |
|---|---|---|
| `KMIPKIT-0051-P001` | Public borrowed `validate()` API and deterministic value-free report. | Pending API tests and rustdoc |
| `KMIPKIT-0051-P002` | Explicit unsupported-schema requests produce a sanitized error-severity issue; no schema inference. | Pending schema validation tests |
| `KMIPKIT-0051-P003` | `Client::execute` blocks locally invalid requests before encoding/transport and reports `NotSent`. | Pending fake-transport tests |
| `KMIPKIT-0051-P004` | Where the Request Payload table omits a TTLV type and OASIS provides no direct operation-specific type: encode Request Count as Integer (the operation describes an integer count and the typed field is `i32`); standalone Usage Limits Count as Long Integer (consistent with shared §7.40 Table 392 and typed `i64`); and IV/Counter/Nonce request values as Byte String (consistent with the explicit `SecretBytes` API and §3.3 Table 15). Do not add positive-only numeric bounds from this decision. | R016, R039, R046, R062, R081, R107 in `gap-matrix.md`; validate encoding through schema and wire tests. |
