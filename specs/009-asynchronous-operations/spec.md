# Feature Specification: KMIP 2.1 Client Asynchronous Operations

**Feature Branch**: `feature/KMIPKIT-0009-asynchronous-operations-spec`
**Created**: 2026-10-06
**Status**: Draft. Direct human instruction for this implementation task explicitly authorizes autonomous implementation without further approval requests. This records implementation authorization only; it does not claim that this specification or reviewer-owned checklists were reviewed or approved. The typed Query Asynchronous Requests response mapping remains gated by open `KMIPKIT-DISC-039`; until that source conflict is resolved, its response is exposed losslessly through generic TTLV.
**Input**: Roadmap item `KMIPKIT-0009-asynchronous-operations`.

## Scope and normative sources

This feature gives a KMIP 2.1 client explicit, one-exchange follow-up for an operation reported as Pending; typed client request and response models for Poll, Cancel, and Process; and a typed Query Asynchronous Requests request with a lossless generic response. It preserves the server-issued Asynchronous Correlation Value byte-for-byte. A Poll that remains Pending returns that state to the caller and requires a new explicit caller action; KMIPKit does not wait, poll again, or retry automatically.

The normative source is the immutable pinned OASIS KMIP Specification v2.1 at `specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html`. The catalog is `specification/catalog/kmip-2.1.json`. All clauses below are cited by exact section and table. The catalog does not link operation-specific official Test Case IDs for these requirements; tests in this feature are derived tests and must not be described as official conformance vectors.

| Stable requirement / source | Treatment |
|---|---|
| `KMIPKIT-REQ-SPEC-9.1-001`; §9.1, Table 400 | Preserve the server-generated correlation value and use it unchanged in a subsequent Poll or Cancel for that original operation. |
| `KMIPKIT-REQ-SPEC-9.19-002`; §9.19, Table 424 | For an Operation Pending response, use the associated asynchronous correlation value in the subsequent Poll. |
| `KMIPKIT-REQ-SPEC-6.1.38-001-001`; §6.1.38, Table 276 | Every Poll request includes the original operation's required Asynchronous Correlation Value. |
| `KMIPKIT-REQ-SPEC-6.1.38-001-002`; §6.1.38 prose | Poll is not itself processed asynchronously. A Poll response may nevertheless report that the original operation remains Pending, subject to §6.1.38's explicit response rules and §8.6 Table 399's Pending correlation field. |
| `KMIPKIT-REQ-SPEC-6.1.5-001`; §6.1.5, Tables 176–178 | Every Cancel request includes the original operation's correlation value. A successful Cancel response contains that value and a Cancellation Result. The Cancel response cannot itself be asynchronous; Pending is rejected even when the original request allowed asynchronous outcomes. |
| `KMIPKIT-CLAUSE-SPEC-6.1.39-001`, `-002`, `-003`; §6.1.39, Tables 278–280; §8.6, Table 399 | Process carries the required correlation value. Every non-Failure Process response has a Response Payload under §8.6/Table 399, and Table 279 defines it as empty; Failure follows the general no-payload shape. Its server-side effect and possible impact on other batch items are documented; KMIPKit does not claim to enforce server behavior. The catalog currently has no requirement ID for Table 278's required client request field; this is an explicit catalog traceability gap and must be addressed through the catalog workflow before a conformance claim. |
| `KMIPKIT-CLAUSE-SPEC-8.6-003`; §8.6, Table 399 | A Pending response batch item contains an Asynchronous Correlation Value. This is server production behavior; the client validates and retains the received value. Poll's operation-specific no-payload rule in §6.1.38 applies when the original request is still pending. |
| §6.1.41, Tables 285–287; `KMIPKIT-DISC-039` | Query Asynchronous Requests accepts optional correlation-value and operation filters. The pinned source captions Table 286 “PKCS#11 Response Payload” while its placement and row describe an asynchronous-request response. Because the conflict is unresolved, KMIPKit does not assign a typed response schema or make an OASIS conformance claim for that mapping in this feature. The response remains available as generic TTLV. |
| §7.1, Table 352; §7.2, Table 353 | Source shapes for Asynchronous Correlation Values and Asynchronous Request are recorded for the Query response mapping decision; they do not override `KMIPKIT-DISC-039`. |

The exact `MUST`/`SHALL` applicability remains that of the pinned source and catalog. Server obligations (for example, returning a pending/completed Poll outcome, honoring Process, or selecting synchronous/asynchronous processing) are not recast as client duties. In particular, `KMIPKIT-REQ-SPEC-6.1.38-001-002` describes the Poll response mode: it does not prohibit a Poll response from reporting the original request's Pending status.

## Clarification record and gates

Decisions supported by the approved roadmap, existing protocol model, client execution contract, and pinned OASIS source:

- KMIPKIT-0006 owns the generic lossless message envelope and typed read-only views. KMIPKIT-0007 owns one synchronous, bounded transport exchange, response/request association, explicit delivery state, and the initial Pending outcome. KMIPKIT-0009 adds operation semantics without weakening those contracts.
- A follow-up is explicit and one-shot. Each invocation sends at most one request and receives at most one response. There is no automatic polling, retry, delay, backoff, background task, wait helper, failover, or cancel-on-drop behavior.
- Correlation values are opaque bytes. KMIPKit never parses, trims, normalizes, regenerates, decodes as text, or substitutes a Client Correlation Value or Unique Batch Item ID for them. Pending values are exposed only through the explicit borrowed accessor defined by KMIPKIT-0007; their KMIPKit-owned storage remains zeroizing and MUST NOT be copied into ordinary unzeroized storage. The exact value supplied by that accessor is emitted in Poll, Cancel, and Process payloads. Query correlation filters preserve caller-supplied bytes. Caller-owned filter input storage remains the caller's responsibility; any copy KMIPKit creates is redacted from diagnostics, kept in zeroizing storage, and not duplicated into ordinary unzeroized storage. A later Pending response supplies the value for any further explicit action; equality with the preceding request's bytes is not inferred.
- Poll and Cancel are not themselves asynchronous operations. Poll can return `Result Status = Pending` for the original operation, with no Poll response payload under §6.1.38 and a correlation value under §8.6 Table 399. A completed Poll carries the original operation's terminal status/reason and response payload semantics, not a Poll-specific result. A successful original-operation completion has its operation payload; a completed Failure exposes its Result Reason and has no response payload. Until each operation has a typed result model, any present payload is retained as generic TTLV. Cancel cannot return Pending; it returns a synchronous Cancellation Result or a synchronous operation failure.
- Process is a separate operation. It asks the server to change processing so the next Poll does not return Pending. When Batch Order Option is true or absent, server processing may affect other batch items; KMIPKit exposes the request and result without promising that effect. The source does not say that Process itself cannot be handled asynchronously, so its response follows the general response model and caller-selected asynchronous indicator.
- The Query Asynchronous Requests response is retained as generic TTLV while `KMIPKIT-DISC-039` remains open. This is a conservative, lossless project disposition, not a claim that the erroneous caption has been authoritatively corrected. A future typed mapping requires a reviewed source decision and corresponding catalog/traceability update.
- Existing public API and ADR-0014 delivery, size-limit, no-retry, and request-lifecycle contracts remain in force. The client continues accepting only typed operations through its normal typed request path; arbitrary caller wire bytes are not introduced.

| ID | Status | Question or disposition | Gate |
|---|---|---|---|
| OD-001 | Open — source conflict | `KMIPKIT-DISC-039`: Does Table 286's “PKCS#11 Response Payload” caption contain a copy/paste error, or is the response table misplaced? | Typed Query response mapping and any conformance claim stay out of scope until a reviewed decision resolves the discrepancy. The generic TTLV response path remains in scope. |
| OD-002 | Open — catalog traceability | Table 278 requires a Process Asynchronous Correlation Value, but the catalog has no requirement ID for the client request field. | Track a catalog input correction through the approved catalog workflow. This feature assigns stable project requirement IDs and tests, but does not hand-edit generated catalog files or claim catalog traceability is closed. |

These gates do not block typed Poll, Cancel, and Process payload work or the generic Query response path. They do block closing normative catalog traceability or claiming complete conformance for the affected areas.

## User Scenarios & Testing

### User Story 1 — Continue or inspect one pending operation (Priority: P1)

A caller receives a Pending outcome, keeps its server-issued correlation bytes, and chooses whether to submit one Poll or Cancel request. The caller can inspect an incomplete Poll result without being put into an automatic loop.

**Why this priority**: Without a safe explicit continuation path, the client cannot expose the result of an operation that the server reports as pending.

**Independent Test**: With a deterministic fake transport, produce an initial Pending outcome, access the value through its borrowed accessor, submit one Poll or Cancel, and assert that the exact correlation bytes appear in the request, KMIPKit creates no ordinary unzeroized duplicate, and only one exchange occurs.

**Acceptance Scenarios**:

1. **Given** an operation response with Result Status Pending and a correlation value, **when** the client exposes its pending outcome, **then** the caller can access the original bytes only through the explicit borrowed accessor, without text conversion, normalization, or an ordinary unzeroized duplicate.
2. **Given** a pending outcome, **when** the caller submits Poll, **then** exactly one Poll exchange uses those exact bytes and returns either Pending or the original operation's terminal status/reason and payload semantics: a successful completion carries the original operation's generic payload, while Failure exposes Result Reason without a payload.
3. **Given** Poll returns Pending, **when** execution completes, **then** no subsequent Poll is sent until the caller explicitly requests another one.
4. **Given** a pending outcome, **when** the caller submits Cancel, **then** exactly one Cancel exchange uses those exact bytes and the caller receives the echoed correlation value and raw/known Cancellation Result.
5. **Given** a Cancel response has Result Status Pending, **when** the client validates it, **then** it rejects the response as a protocol error even if the original operation permitted asynchronous results.
6. **Given** Poll reports that the original operation completed with Failure, **when** the client returns the terminal outcome, **then** it exposes the failure status and Result Reason with no response payload and does not misclassify the failure as Pending.
7. **Given** a malformed, oversized, or invalid response, **when** a follow-up exchange fails, **then** the client returns a redacted error with the correct delivery state and does not retry.

### User Story 2 — Request server processing for a pending operation (Priority: P1)

A caller explicitly submits Process for a pending operation when it wants to request the server's defined processing-mode change, and receives the result of that Process operation separately from the original operation's outcome.

**Why this priority**: Process is a distinct KMIP operation with side effects that must not be conflated with polling or cancellation.

**Independent Test**: Verify that a Process request contains the preserved correlation bytes, that the response is correlated to the Process request, that its payload follows Table 279, and that one invocation causes one exchange only.

**Acceptance Scenarios**:

1. **Given** a pending outcome, **when** the caller submits Process, **then** the request uses its exact correlation value and the client does not represent Process as Poll or Cancel.
2. **Given** Batch Order Option is absent or true, **when** the caller submits Process, **then** documentation explains that server processing may affect other items in that original batch; the client does not claim to control or verify that server-side effect.
3. **Given** Process returns a result, **when** the client exposes it, **then** it is associated with the Process request and its delivery state is independent of the original pending operation.
4. **Given** Process itself returns Pending and the request permitted asynchronous results, **when** execution completes, **then** the new pending outcome is exposed without automatically polling it.

### User Story 3 — Query outstanding asynchronous requests (Priority: P2)

A caller can construct Query Asynchronous Requests with zero or more correlation-value filters and zero or more operation filters, then inspect its response without losing unknown TTLV content.

**Why this priority**: Query provides a separate server-supported way to discover outstanding asynchronous results, while the source's response caption conflict prevents an unreviewed typed interpretation.

**Independent Test**: Round-trip query requests with omitted, empty, and repeated filter structures; return an arbitrary structurally valid response payload and verify byte-preserving generic access.

**Acceptance Scenarios**:

1. **Given** no filters, **when** Query Asynchronous Requests is encoded, **then** both optional filter structures may be absent.
2. **Given** correlation-value or operation filters, **when** the request is encoded, **then** order, repetition, and exact supplied values are preserved according to Tables 285 and 352.
3. **Given** a server response to this operation, **when** it is decoded while `KMIPKIT-DISC-039` remains open, **then** the response is exposed as generic TTLV and is not represented as an official typed Table 286 interpretation.

### Edge Cases

- Empty correlation byte strings are not rejected unless an exact normative source clause or approved policy requires that rule; no token format or maximum independent of `CodecLimits` is invented.
- Arbitrary binary correlation bytes, embedded NUL, high-bit bytes, and maximum configured TTLV payloads.
- Poll Pending has no operation payload but has the Pending correlation required by §8.6 Table 399; a terminal Poll exposes the original operation's status/reason, with generic payload on success and no payload on Failure under §8.6/Table 399.
- A Poll response can have Pending status for the original operation even though Poll itself is not asynchronous. It must not trigger hidden follow-up work.
- Cancel response correlation does not match the request; Cancellation Result contains a known or unknown Enumeration; server error result.
- Process with Batch Order Option absent, false, or true; Process itself returns Success, Failure, or permitted Pending; every non-Failure result carries the empty Response Payload required by §8.6/Table 399 and §6.1.39/Table 279, while Failure has no payload.
- Query filters absent, present-but-empty, repeated, mixed, and containing unknown operation values; opaque response contains unknown tags and repeated structures.
- Duplicate, missing, unexpected, or mismatched Unique Batch Item IDs on the follow-up response are rejected under KMIPKIT-0007 response association rules.
- Transport failures before sending, after possible partial sending, and after response bytes begin; never retry.

## Requirements

### Functional Requirements

- **KMIPKIT-0009-FR-001**: The client MUST represent typed client-initiated Poll, Cancel, Process, and Query Asynchronous Requests inputs using the exact payload shapes and required/optional fields in OASIS v2.1 §§6.1.5, 6.1.38, 6.1.39, and 6.1.41, Tables 176–178, 276–280, and 285–287. Query's response interpretation is subject to FR-011.
- **KMIPKIT-0009-FR-002**: When an initial or subsequent response is Pending, the client MUST retain the server-provided Asynchronous Correlation Value as exact opaque bytes and expose it only through KMIPKIT-0007's explicit borrowed accessor. KMIPKit-owned storage MUST remain zeroizing and MUST NOT be copied into ordinary unzeroized storage. The value MUST remain distinct from Client Correlation Value and Unique Batch Item ID. Sources: `KMIPKIT-0007-FR-017`; requirements `KMIPKIT-REQ-SPEC-9.1-001` (§9.1, Table 400), `KMIPKIT-REQ-SPEC-9.19-002` (§9.19, Table 424), and `KMIPKIT-CLAUSE-SPEC-8.6-003` (§8.6, Table 399).
- **KMIPKIT-0009-FR-003**: Every Poll request MUST contain the pending operation's exact correlation bytes. Poll's response MUST be handled according to §6.1.38: Pending for the original operation has no operation payload; completion exposes the original operation's terminal status/reason and payload semantics. A completed failure exposes its Result Reason and has no payload under the general Response Batch Item rules. A present operation payload remains generic when its typed response model is unavailable. Poll MUST NOT be treated as a new asynchronous operation. Sources: `KMIPKIT-REQ-SPEC-6.1.38-001-001`, `KMIPKIT-REQ-SPEC-6.1.38-001-002`, and `KMIPKIT-CLAUSE-SPEC-8.6-003`, §§6.1.38 and 8.6, Tables 276 and 399.
- **KMIPKIT-0009-FR-004**: Every Cancel request MUST contain the pending operation's exact correlation bytes. A successful Cancel response MUST expose the echoed correlation value and Cancellation Result from Table 177. Known Cancellation Result values MUST have typed views; unknown enumeration values MUST remain representable. A Cancel response MUST NOT be Pending or otherwise asynchronous; the client MUST reject such a response even if the original operation permitted Pending. Source: `KMIPKIT-REQ-SPEC-6.1.5-001`, §6.1.5, Tables 176–178; enum definitions §11.7.
- **KMIPKIT-0009-FR-005**: Every Process request MUST contain the pending operation's exact correlation bytes. Every non-Failure Process response MUST include the empty Response Payload Structure defined by §6.1.39, Table 279, as required by §8.6, Table 399; a Failure response has no payload. The client MUST expose Process independently and MUST NOT claim that it has made a later Poll complete or that it has controlled any other batch item. Sources: `KMIPKIT-CLAUSE-SPEC-6.1.39-001` through `-003`, §6.1.39, Tables 278–280. A stable project requirement ID is assigned here because the catalog lacks one for the required request field; catalog repair remains open under OD-002.
- **KMIPKIT-0009-FR-006**: Query Asynchronous Requests MUST represent both optional request filters from Table 285, including zero or more correlation values and zero or more operation values. Operation and correlation values remain lossless, including unknown values. Source: §6.1.41, Table 285; structures in §7.1, Table 352.
- **KMIPKIT-0009-FR-007**: Until `KMIPKIT-DISC-039` is resolved, the Query Asynchronous Requests response MUST be available as generic TTLV and MUST NOT be exposed as a typed mapping claimed to be defined by Table 286. The response mapping, literal caption, structural placement, and open discrepancy MUST be documented together. Source: §6.1.41, Table 286 and `specification/catalog/review-evidence.md`.
- **KMIPKIT-0009-FR-008**: Each explicit follow-up invocation MUST perform at most one transport exchange and return its outcome to the caller. KMIPKit MUST NOT automatically poll, retry, wait, back off, schedule background work, fail over, or cancel on drop. Transport failure MUST preserve KMIPKIT-0007/ADR-0014 delivery state and redaction behavior.
- **KMIPKIT-0009-FR-009**: All Asynchronous Correlation Values MUST be redacted from Debug, Display, errors, logs, and diagnostic output, including caller-supplied Query filters and server-provided Pending values. KMIPKit-owned allocations MUST be zeroized under the accepted ownership contract and MUST NOT be copied into ordinary unzeroized storage. Caller-owned Query filter input storage remains the caller's responsibility; the client MUST NOT log, format, or retain it beyond the request lifetime. The raw KMIP message body and response payload MUST never be added to an error.
- **KMIPKIT-0009-FR-010**: All follow-up results MUST use KMIPKIT-0007's batch response association and validation rules. A Poll completion's original-operation response payload MUST remain generic and lossless until a typed model for that original operation exists; it MUST NOT be misrepresented as a Discover Versions response.
- **KMIPKIT-0009-FR-011**: This feature MUST NOT change upstream OASIS sources or generated normative catalog outputs; MUST NOT resolve `KMIPKIT-DISC-039` without a reviewed source decision; MUST NOT claim profile support; and MUST NOT add server-initiated operations, automatic async orchestration, a server, new transport, or language bindings.
- **KMIPKIT-0009-FR-012**: Every applicable normative requirement and project policy MUST map to its exact clause/table, this feature's stable requirement ID, implementation location, and executable verification. Where the upstream catalog lacks a client requirement ID or official case, the gap MUST remain explicit and no complete-traceability or official-conformance claim may be made.

### Requirement Traceability

| Stable feature requirement | Normative/project source | Implementation and executable evidence | Disposition |
|---|---|---|---|
| FR-001 | §§6.1.5, 6.1.38, 6.1.39, 6.1.41; Tables 176–178, 276–280, 285–287 | Models: `crates/kmipkit-protocol/src/{cancel,poll,process,query_async_requests}.rs`; tests: `crates/kmipkit-protocol/tests/unit/{cancel,poll,process,query_async_requests}_tests.rs` cover fields, order, values, and operation tags | Implemented for represented inputs; disputed Query response interpretation remains excluded by FR-007 |
| FR-002 | `KMIPKIT-0007-FR-017`; `KMIPKIT-REQ-SPEC-9.1-001`, `KMIPKIT-REQ-SPEC-9.19-002`, `KMIPKIT-CLAUSE-SPEC-8.6-003`; §§8.6, 9.1, 9.19; Tables 399, 400, 424 | Zeroizing `SecretBytes`: `crates/kmipkit-protocol/src/asynchronous.rs`; borrowed views: `crates/kmipkit-protocol/src/message/batch.rs`, `crates/kmipkit-client/src/execute.rs`; tests: `crates/kmipkit-protocol/tests/unit/poll_tests.rs::poll_payload_round_trips_arbitrary_binary_correlation_values`, `crates/kmipkit-protocol/tests/message_conversion.rs::asynchronous_correlation_bytes_survive_generated_pending_round_trips`, `crates/kmipkit-client/tests/unit/poll_execution_tests.rs::pending_poll_returns_without_repeating_and_zeroizes_request_and_response_copies`, `crates/kmipkit-ttlv/tests/value_zeroization.rs::live_zeroize_clears_each_payload_variant_and_nested_structure` | Implemented under KMIPKIT-0007's ownership boundary |
| FR-003 | `KMIPKIT-REQ-SPEC-6.1.38-001-001`, `-001-002`; `KMIPKIT-CLAUSE-SPEC-8.6-003`; §§6.1.38, 8.6, Tables 276, 399 | Implementation: `crates/kmipkit-protocol/src/poll.rs`, `crates/kmipkit-client/src/execute.rs`; tests: `crates/kmipkit-protocol/tests/unit/poll_tests.rs`, `crates/kmipkit-client/tests/unit/poll_execution_tests.rs` cover Pending/no-payload, generic completion, terminal Failure reason, exact correlation, one exchange, and no automatic Poll | Implemented |
| FR-004 | `KMIPKIT-REQ-SPEC-6.1.5-001`; §6.1.5, Tables 176–178; §11.7, Tables 437–438 | Implementation: `crates/kmipkit-protocol/src/cancel.rs`, `crates/kmipkit-client/src/execute.rs`; `crates/kmipkit-protocol/tests/unit/cancel_tests.rs` covers exact bytes, all five assigned Cancellation Result values, and unknown preservation; `crates/kmipkit-client/tests/unit/cancel_execution_tests.rs` covers echo, mismatch, Pending rejection, redaction, and one exchange | Implemented |
| FR-005 | `KMIPKIT-CLAUSE-SPEC-6.1.39-001` through `-003`; §6.1.39, Tables 278–280; §8.6, Table 399 | Implementation: `crates/kmipkit-protocol/src/process.rs`, `crates/kmipkit-client/src/execute.rs`; `process_tests.rs::process_pending_rejects_a_nonempty_table_279_payload` and `process_execution_tests.rs::process_pending_with_nonempty_payload_is_rejected_after_one_exchange` verify operation-boundary rejection while generic Pending shape remains valid and one exchange occurs; other Process tests cover correlation, empty Success, Failure, and association | Implemented for represented behavior; catalog field ID gap open under OD-002 |
| FR-006 | §6.1.41, Table 285; §7.1, Table 352 | Implementation: `crates/kmipkit-protocol/src/query_async_requests.rs`; tests: `crates/kmipkit-protocol/tests/unit/query_async_requests_tests.rs` cover absent, present-empty, repeated, ordered, unknown, and arbitrary-byte filters | Implemented |
| FR-007 | §6.1.41, Table 286; `KMIPKIT-DISC-039` | Generic response owner and callback view: `crates/kmipkit-protocol/src/query_async_requests.rs`, `crates/kmipkit-client/src/execute.rs`; tests: `crates/kmipkit-protocol/tests/unit/query_async_response_tests.rs`, `crates/kmipkit-client/tests/unit/query_async_execution_tests.rs`; no typed Table 286 schema exists | Generic access implemented; typed mapping and discrepancy closure blocked by OD-001 |
| FR-008 | Constitution IV; ADR-0014; KMIPKIT-0007 transport contract | `Client::exchange_operation` in `crates/kmipkit-client/src/execute.rs` is the single private transport path; operation tests in `crates/kmipkit-client/tests/unit/{poll,cancel,process,query_async}_execution_tests.rs` assert one exchange and delivery state; `crates/kmipkit-client/tests/unit/execute_boundary_tests.rs::production_source_inventory_is_complete_and_execute_owns_the_only_writer_permit_pair` guards the shared writer/permit/exchange path | Implemented for explicit calls |
| FR-009 | `KMIPKIT-0007-FR-017`; AGENTS.md §8; constitution IV; ADR-0014 | Existing `ZeroizationObserver` verifies the encoded request owner is cleared on Cancel/Query success and exchange-error paths in `cancel_execution_tests.rs::{cancel_encoded_request_copy_is_zeroized_after_success,cancel_encoded_request_copy_is_zeroized_after_exchange_error}` and `query_async_execution_tests.rs::{query_filter_encoded_request_copy_is_zeroized_after_success,query_filter_encoded_request_copy_is_zeroized_after_exchange_error}`. Generic TTLV `Value` cleanup remains covered by `value_zeroization.rs`; `execute_boundary_tests.rs::query_request_is_dropped_before_exchange_and_not_retained_by_the_client` checks Query model lifetime; a separate caller-owned buffer remains unchanged in the Query success test. These observers claim only the owner they inspect. | Implemented under documented ownership boundary; caller-owned Query input remains caller responsibility |
| FR-010 | KMIPKIT-0006/0007 accepted models; generic TTLV contract | `validate_async_response` in `crates/kmipkit-client/src/execute.rs` uses shared `associate_batch_items`; association errors: `crates/kmipkit-client/tests/unit/execute_private_error_tests.rs`; unexpected response ID: `crates/kmipkit-client/tests/unit/process_execution_tests.rs::process_rejects_a_response_batch_id_that_the_request_did_not_supply`; Poll payload semantics: `crates/kmipkit-client/tests/unit/poll_execution_tests.rs::completed_poll_keeps_original_payload_generic_and_failure_has_no_payload` | Implemented for one-item explicit follow-up requests |
| FR-011 | AGENTS.md §§2, 4, 9; roadmap; pinned-source immutability rule | No upstream OASIS source, normative catalog input, or generated file is intentionally changed; verify after commits with `git diff --exit-code b5e5579..HEAD -- specification/oasis/kmip-2.1/upstream specification/catalog/kmip-2.1.json crates/kmipkit-ttlv/src/generated/tag_allocations.rs crates/kmipkit-protocol/src/result_values_generated.rs` | In scope; no profile or official-conformance claim |
| FR-012 | AGENTS.md §5; constitution I–II | This matrix links stable feature IDs to exact clauses, code paths, and executable tests. Derived tests are not official vectors; the Process catalog field ID and typed Query interpretation remain explicitly open | Internal implementation mapping recorded; release-wide catalog traceability remains open under OD-001/OD-002 |

## Key Entities

- **Pending Operation**: A caller-visible outcome holding the most recent server-provided correlation bytes and original request association. It does not schedule or execute work.
- **Asynchronous Correlation Value**: Opaque server-issued bytes, distinct from the client correlation string and batch item identifier.
- **Poll Request / Outcome**: An explicit request for the original operation's current result. Pending carries no original operation payload; successful terminal completion carries the original operation payload, while terminal Failure exposes its status/Result Reason without a payload.
- **Cancel Request / Outcome**: An explicit cancellation attempt and server-reported Cancellation Result, including the echoed correlation value.
- **Process Request / Outcome**: An explicit request to change server processing mode for a previously submitted operation.
- **Query Asynchronous Requests**: A request with optional correlation-value and operation filters; response remains generic TTLV while `KMIPKIT-DISC-039` is open.
- **Follow-up Exchange**: One caller-selected request and one synchronous transport exchange with delivery-aware errors.

## Success Criteria

- **KMIPKIT-0009-SC-001**: Tests prove exact byte-for-byte preservation for correlation values of arbitrary binary content through pending retention and Poll, Cancel, and Process request encoding; no normalization or text interpretation occurs.
- **KMIPKIT-0009-SC-002**: Table-driven tests cover required and optional fields, valid status/payload combinations, malformed TTLV, unknown enum values, and batch response association for Poll, Cancel, Process, and Query request models.
- **KMIPKIT-0009-SC-003**: A Poll returning Pending or completed causes exactly one exchange; a second Poll is sent only after a distinct explicit caller invocation; every failure case performs zero automatic retries and reports the established delivery state.
- **KMIPKIT-0009-SC-004**: Pending correlation values are available only through the borrowed accessor and no ordinary unzeroized duplicate is created. No sentinel correlation value appears in Debug, Display, errors, or logs; KMIPKit-owned correlation buffers are zeroized on drop within the accepted memory ownership contract, while caller-owned Query filter input storage remains the caller's responsibility.
- **KMIPKIT-0009-SC-005**: 100% of in-scope requirement IDs and project rules link to exact clauses/tables, planned implementation, and executable tests; gaps in catalog IDs or unavailable official Test Case IDs are explicitly recorded.
- **KMIPKIT-0009-SC-006**: Query Asynchronous Requests responses and any successful Poll completion payload survive generic TTLV round trips without field or byte loss; terminal Poll Failure exposes its status/Result Reason without a payload. No typed Query response or official conformance claim is made while DISC-039 remains open.

## Assumptions and exclusions

- KMIP 2.1 and TTLV are the only protocol version and encoding in scope for the 1.0 line.
- The initial pending response is produced only when the request's Asynchronous Indicator permits it under KMIPKIT-0007. This feature does not choose server timing or require a server to use asynchronous execution.
- Server policy may reject any operation. A represented operation is not a guarantee that a server supports or accepts it.
- Generic TTLV is the compatibility path for results whose operation-specific typed model has not yet landed.
- Derived tests are not official vectors; no requirement-specific official Test Case IDs are linked by the pinned catalog.
- Excluded: automatic polling/retry/wait/backoff, server-initiated operations (1.1), transport creation, TLS/HTTPS behavior, C/Java/Python bindings, JSON/XML, profile claims, credential selection, and server-side asynchronous processing.

## Implementation gates

The human instruction authorizing this implementation task permits autonomous implementation without further approval prompts. It does not claim formal approval of this draft specification or its reviewer-owned checklists; their owners retain review responsibility, and their unchecked items remain open. The accepted KMIPKIT-0006/0007 contracts, pinned source, `KMIPKIT-DISC-039`, and the Process catalog traceability gap were checked before implementation. Implementation requires strict Red, Green, Refactor commits and the project-required tests, coverage, source immutability, and security checks. This authorization closes none of the open catalog/source gates.
