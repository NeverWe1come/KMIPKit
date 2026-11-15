# Data Model: KMIP 2.1 Managed-Object State Transitions

## LifecycleOperation

One of Activate, Archive, Destroy, or Recover, identified by the exact KMIP Operation enumeration value and its catalog operation element. Each request is independent; a batch may contain more than one operation under the existing ordered batch model.

## LifecycleRequest

Each typed request contains the operation's optional Unique Identifier. Omission is preserved. A valid existing ID Placeholder remains available when the request is used in a batch whose preceding items establish that placeholder. The client does not generate an identifier, resolve a remote identifier locally, or infer object state.

| Request | KMIP table | Fields represented by this feature |
| --- | --- | --- |
| ActivateRequest | §6.1.1 Table 164 | Optional Unique Identifier |
| ArchiveRequest | §6.1.4 Table 173 | Optional Unique Identifier |
| DestroyRequest | §6.1.15 Table 208 | Optional Unique Identifier |
| RecoverRequest | §6.1.42 Table 288 | Optional Unique Identifier |

## LifecycleResponse

Each successful typed response contains the operation-required Unique Identifier. Response payloads use their operation-specific KMIP response table; missing, duplicated, wrong-typed, or otherwise malformed required fields are protocol errors. A non-success result does not produce a fabricated typed-success payload.

| Response | KMIP table | Required success field |
| --- | --- | --- |
| ActivateResponse | §6.1.1 Table 165 | Unique Identifier |
| ArchiveResponse | §6.1.4 Table 174 | Unique Identifier |
| DestroyResponse | §6.1.15 Table 209 | Unique Identifier |
| RecoverResponse | §6.1.42 Table 289 | Unique Identifier |

## LifecycleOperationResult

The result is correlated to its request using the shared Batch Item model. Success exposes the appropriate typed response. Failure retains the shared Result Status, operation-specific Result Reason, and permitted Result Message. Pending remains the shared operation-agnostic outcome, with the originating operation, exact KMIP result, and exact Asynchronous Correlation Value. Pending is accepted only when the request and common client contract permit it.

## State and ownership boundaries

- KMIPKit has no local managed-object lifecycle state machine. The remote KMIP server owns activation, archival, destruction, recovery, authorization, and policy decisions.
- Each explicit invocation performs at most one exchange; delivery evidence is NotSent, PossiblySent, or ResponseStarted under the existing transport contract.
- Response parsing uses configured TTLV limits before allocation and retains generic source data for values not modeled by this operation family.
- Errors and formatting do not include raw KMIP bodies or secret-bearing attribute values.
