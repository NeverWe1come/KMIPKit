# Operation API Contract

This contract defines the typed client surface for the four in-scope operations in `spec.md`. All operation calls use the existing typed batch, bounded TTLV codec, result, and delivery-state contracts. Calls perform one explicit exchange. Errors and server results are sanitized; they never expose raw KMIP bodies.

## Typed request/response surface

| Rust-level model/call | Request TTLV contract | Success response TTLV contract | Normative source |
| --- | --- | --- | --- |
| `GetConstraintsRequest` / `Client::get_constraints` | Empty operation payload | Required Constraints Structure | Specification §6.1.22, Tables 229–231; §7.7 |
| `GetUsageAllocationRequest` / `Client::get_usage_allocation` | Optional Unique Identifier; required Usage Limits Count as TTLV Long Integer (`i64`) | Required Unique Identifier | Specification §6.1.23, Tables 232–234; §7.40, Table 392 |
| `SetConstraintsRequest` / `Client::set_constraints` | Required Constraints Structure | Empty operation payload | Specification §6.1.52, Tables 325–327; §7.7 |
| `SetDefaultsRequest` / `Client::set_defaults` | Optional Defaults Information Structure | Empty operation payload | Specification §6.1.53, Tables 328–330; §7.11 |

The Operation field and operation payload remain distinct in the KMIP request message. Field order, repeated values where permitted, optional omission, and structurally valid unknown elements are preserved. Enumeration values are open raw values, not closed enums that reject future values.

## Result and delivery behavior

- Return the existing typed response/item view with Result Status, Result Reason, permitted Result Message, extensions, async metadata, and request/response identity.
- Preserve `Operation Pending` and exact correlation bytes when the caller has explicitly selected an asynchronous-capable batch.
- Do not poll, retry, allocate a new request identifier, or repeat an exchange automatically.
- Return `NotSent` for local payload, shape, or guard validation failure before request transmission.
- Preserve `PossiblySent` or `ResponseStarted` on ambiguous transport/protocol failures.
- Preserve complete operation failures from the exact error tables. Unknown values remain available.
- No automatic object-state, credential, defaults, or endpoint-role mutation is maintained locally after an operation.

## Error tables

Each operation preserves its specification Table 231, 234, 327, or 330 error forms and common result fields. The implementation must cite exact tables in tests and must not convert an Operation Failed result into a transport/protocol failure.

## Set Defaults absence

OASIS KMIP v2.1 §6.1.53, Table 328 explicitly states that omission of Defaults Information requests removal of all Object Defaults from the server. The encoder must preserve the absent field rather than serialize an empty Defaults Information structure. KMIPKIT-DISC-020 concerns Table 330's truncated error introduction, not this request semantic.

## Scope limits

Set Endpoint Role (§6.1.54) and its server-to-client counterpart (§6.2.5) are deferred from 1.0. OASIS §6.1.54 requires a post-success role transition, which cannot be completed within the approved client-only boundary. A 1.1 specification must cover both directions and channel behavior. This contract also excludes administrative policy discovery, remote object-state lookup, profile claiming, and local cryptographic operations. Generic structurally valid TTLV access remains governed by the existing API and is not promoted into an operation-specific typed method here.
