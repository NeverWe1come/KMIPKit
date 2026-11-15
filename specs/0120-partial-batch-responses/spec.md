# Specification: Partial responses for stopped and undone KMIP batches

**Feature Branch**: `feature/KMIPKIT-0120-partial-batch-responses`
**Status**: Approved for implementation by the maintainer's autonomous interoperability directive
**Owning identifier**: KMIPKIT-0120

## Purpose

Allow the typed client to represent every response item a KMIP server actually
returns when processing stops after a batch failure. Do not synthesize results
for request items that the server did not process. Keep `Continue` batches
strict: they must still return one response for each requested item.

## Normative references

- OASIS KMIP Specification Version 2.0, OS, §11.5, Table 396 (Batch Error
  Continuation Option descriptions); §9.19, Table 386 (Result Status).
- OASIS KMIP Specification Version 2.1, OS, §11.5, Table 434 (Batch Error
  Continuation Option descriptions); §9.19, Table 424 (Result Status).
- OASIS KMIP Specification Version 2.1, OS, §9.21, Table 426 (Unique Batch
  Item ID).

## Requirements

| ID | Requirement | Verification |
|---|---|---|
| KMIPKIT-0120-FR-001 | The client SHALL accept a response with fewer batch items than were requested only when the effective Batch Error Continuation Option is `Stop` or `Undo`, at least one returned response has `Operation Failed`, and all returned items correlate uniquely to request items. | Typed client tests over controlled transport. |
| KMIPKIT-0120-FR-002 | The client SHALL reject empty, oversized, duplicate, unknown, or operation-mismatched partial response sets and SHALL retain strict full-count validation for `Continue`. | Association and protocol validation tests. |
| KMIPKIT-0120-FR-003 | Returned items SHALL remain available in request order with their original Unique Batch Item IDs. Missing responses SHALL NOT be represented as `Operation Undone`, `Operation Failed`, or another fabricated outcome. | API result ordering and ID correlation tests. |
| KMIPKIT-0120-FR-004 | `ClientBatchResponse::len` and iteration SHALL describe responses actually returned by the server. Documentation SHALL explain that Stop/Undo can return fewer items than the request batch and that IDs correlate actual responses. | Public API documentation checks. |
| KMIPKIT-0120-FR-005 | C, Java and Python wrappers SHALL continue to expose the Rust client result consistently; no vendor extensions or live-server assumptions may be introduced. | Binding and installed-package CI. |

## Scope

This change applies only to typed request batches using KMIP 2.0 or 2.1 and the
standard `Stop`, `Undo`, and `Continue` options. It does not add a result value
for a request item that has no response. A caller can inspect the returned
item count and correlate returned outcomes with request items through their
Unique Batch Item IDs.

The offline controlled-peer test is evidence for the KMIPKit client behavior;
it is not evidence of interoperability with a vendor KMS.

## Acceptance

The client crate's focused and complete test suites, binding tests, API
contract checks, and CI SHALL pass. A consumer built against the updated
release artifact SHALL successfully decode a standards-conforming partial
Stop response and report the omitted request item as absent.
