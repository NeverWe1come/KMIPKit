# Requirements Quality Checklist: KMIP 2.1 Typed Client Execution

**Purpose**: Review specification completeness and source mapping before implementation planning is approved.
**Created**: 2026-10-05
**Feature**: [`spec.md`](../spec.md)
**Review Ownership**: Reviewer-owned. An unchecked item requires review, correction, or an explicit documented finding.

## Scope and dependencies

- [ ] CHK001 Confirm the first operation is explicitly Discover Versions and does not imply live-server support or automatic preflight behavior.
- [ ] CHK002 Confirm `Client::execute(&mut self, batch: ClientBatch, limits: &CodecLimits) -> Result<ClientBatchResponse, ClientError>` and the closed Discover Versions-only request set match the accepted KMIPKIT-0005/0006 APIs before approval.
- [ ] CHK003 Confirm `Transport::exchange` is documented public API of `kmipkit-transport` under ADR-0014, absent from the top-level facade, and not injectable through a public `Client` constructor; confirm the unpublished fake belongs in `kmipkit-test-support` and the later TLS/HTTPS feature constructs `Client` from validated configuration.
- [ ] CHK004 Confirm OASIS §8.3/Table 396, §9.7/Table 406, and §6.1.16/Table 211 impose no operation-uniqueness or Discover Versions batch restriction; repeated Discover Versions items are fake-only test data and do not imply server interoperability.
- [ ] CHK005 Confirm all dependency, ADR-0012, and human-review gates are stated as blockers and cannot be bypassed by task ordering.

## Normative precision

- [ ] CHK006 Verify each §6.1.16 request/response and error statement against Tables 211–213 and catalog records; distinguish OASIS requirements from fixed KMIPKit 1.0 behavior.
- [ ] CHK007 Verify response Operation and Unique Batch Item ID echo requirements against §§8.3, 8.6, 9.21, Tables 396/399; label extra strictness as project policy.
- [ ] CHK008 Verify mixed Pending/completed acceptance against `KMIPKIT-REQ-SPEC-8-003-002`, §9.2, Table 432, and §11.3, including extension-range semantics, rejection of Pending without the required Asynchronous Correlation Value (`KMIPKIT-REQ-SPEC-9.19-002`), and capability-like-value redaction/zeroization under FR-017.
- [ ] CHK009 Verify outbound Batch Error Continuation presence/cardinality against §9.6; preserve `KMIPKIT-DISC-001`; verify `KMIPKIT-DISC-043` and delegated project policy `KMIPKIT-DEC-002` separately record the §9.6/Table 435 value-range discrepancy and assigned-only outbound behavior.
- [ ] CHK010 Verify Maximum Response Size and local response-byte limits are distinguished, the peer field is omitted for Discover Versions, and §9.12 large-response recommendation is assigned to later operation specifications.
- [ ] CHK011 Verify rejection of unrecognized critical extensions against §9.13; separately verify preservation of unknown non-critical extensions as KMIPKit policy (OASIS permits processing them as absent); confirm ADR-0013 assigns registry and typed extension adapters to KMIPKIT-0012 before API freeze.
- [ ] CHK012 Verify the exact-2.1 acceptance behavior and `KMIPKIT-DISC-022` scope-exception wording against ADR-0002 and §9.16.
- [ ] CHK013 Confirm derived conformance tests are not described as official OASIS test vectors when no linked test IDs/fixtures exist.

## Security, testability, and traceability

- [ ] CHK014 Confirm `max_response_bytes` is derived exactly from `CodecLimits.max_message_bytes()`, the 0007 fake obeys the exchange cap contract and the client independently checks before decoder entry; its wrapper-drop and partial-error tests inspect only initialized bytes in the current allocation; assign concrete pre-allocation and allocation-growth cleanup tests on success/errors to each later production adapter specification.
- [ ] CHK015 Confirm the owner-through-transport test exercises the candidate production callsite, partial writes, both success/error returns, request and Pending-correlation zeroization, delivery state, `Debug`/`Display`/error-source/log redaction, and no retry; separately confirm T010's direct low-level request-sentinel test covers nonlogging, nonretention after both outcomes, and initialized-range cleanup of temporary copies.
- [ ] CHK016 Confirm same-reference CodecLimits forwarding is testable without cloning or reconstructing limits.
- [ ] CHK017 Confirm each FR/SC has a measurable acceptance test and stable traceability ID.
- [ ] CHK018 Confirm C ABI, Java/Python, live transport, all other operation schemas, Poll/Cancel/Process, retries, and server-initiated behavior are explicit exclusions.
- [ ] CHK019 Confirm `kmipkit::Client::execute` has no raw-send or arbitrary-transport injection path; explicitly document that the low-level `kmipkit-transport` API accepts caller bytes and returns only zeroizing `TransportResponse`, its ADR-0012 Decision 4 exception, caller responsibility, initialized-current-allocation cleanup, and limits for spare capacity, previous allocations, and external copies; verify API examples are executable/doctested.
- [ ] CHK020 Confirm the optional request Time Stamp is represented as Date-Time, an explicit caller value is preserved exactly and absence stays absent, the value is absent from diagnostics, error formatting and exposed error-source chains and never logged, no countdown-derived outgoing value is exposed under the OD-004 scope disposition, and Server Correlation Value is excluded from the typed client-initiated request API.
- [ ] CHK021 Confirm this Draft makes the first secret-bearing operation specification's owner-through-transport lifecycle test a mandatory approval and enablement gate under OD-006; this Discover Versions-only slice does not authorize secret-bearing requests.
- [ ] CHK022 Confirm ADR-0014, FR-005, the data model, contract, plan, quickstart, tasks, approval record, and public API documentation agree on initialized-byte/current-allocation cleanup; confirm the fake tests do not claim cleanup of spare capacity, prior allocations, or TLS/transport-library copies; and confirm every future concrete adapter specification assigns success/error allocation-growth and request sentinel tests as applicable.

## Notes

- All checklist items remain unchecked until reviewed. The Draft status and unchecked markers are intentional; they are not approval evidence.
