# Public Rust Contract: KMIP 2.1 Managed-Object State Transitions

## Typed request models

- `ActivateRequest`, `ArchiveRequest`, `DestroyRequest`, and `RecoverRequest` expose the optional Unique Identifier in their respective KMIP request tables.
- Each typed request has a closed `ClientRequest` variant and may be added to an ordered `ClientBatch`. Existing ID Placeholder and batch-option semantics remain unchanged.
- Constructors preserve an omitted identifier and do not generate one. Structurally invalid inputs fail before dispatch with delivery state `NotSent`.

## Client entry points and outcomes

- `Client::activate`, `Client::archive`, `Client::destroy`, and `Client::recover` provide typed single-operation conveniences following the existing operation method convention. Each constructs one batch item and invokes the shared `Client::execute` path once.
- `Client::execute` remains the sole multi-item exchange path; response correlation and batch order are handled by the common model.
- The typed success variants contain the operation-required Unique Identifier. Failure remains a failure result; it is not converted into a typed successful response.
- Pending remains operation-agnostic and preserves the operation identity, shared Result, and exact Asynchronous Correlation Value. The caller may explicitly invoke Poll after Recover as permitted by the shared API; Get after Recover is a separate explicit operation.
- No convenience method automatically retries, polls, or issues Get.

## Validation, errors, and compatibility

- Missing, duplicate, or wrong-typed required success fields produce a sanitized protocol error before a typed response is returned.
- Valid server failures preserve their operation-specific Result Reason and permitted Result Message under the shared result model.
- Transport failures preserve NotSent, PossiblySent, or ResponseStarted. The client never retries automatically.
- Generic TTLV extension and unknown-value handling, redaction, and zeroization remain governed by existing common contracts.
- This is additive pre-1.0 Rust surface. Generated C, JNI, and CFFI methods are added by the later API parity specifications; this spec does not modify generated binding code.
