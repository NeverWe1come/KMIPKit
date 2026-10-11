# Operation API Contract

This contract defines the typed client surface for the five operations in `spec.md`. All operation calls use the existing typed batch, bounded TTLV codec, result, and delivery-state contracts. Calls perform one explicit exchange. Errors and server results are sanitized; they never expose raw KMIP bodies.

## Typed request/response surface

| Rust-level model/call | Request TTLV contract | Success response TTLV contract | Normative source |
| --- | --- | --- | --- |
| `GetConstraintsRequest` / `Client::get_constraints` | Empty operation payload | Required Constraints Structure | Specification §6.1.22, Tables 229–231; §7.7 |
| `GetUsageAllocationRequest` / `Client::get_usage_allocation` | Optional Unique Identifier; required Usage Limits Count Integer | Required Unique Identifier | Specification §6.1.23, Tables 232–234 |
| `SetConstraintsRequest` / `Client::set_constraints` | Required Constraints Structure | Empty operation payload | Specification §6.1.52, Tables 325–327; §7.7 |
| `SetDefaultsRequest` / `Client::set_defaults` | Optional Defaults Information Structure | Empty operation payload | Specification §6.1.53, Tables 328–330; §7.11 |
| `SetEndpointRoleRequest` / `Client::set_endpoint_role` | Required Endpoint Role Enumeration | Required Endpoint Role Enumeration | Specification §6.1.54, Tables 331–333; §11.19 |

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

Each operation preserves its specification Table 231, 234, 327, 330, or 333 error forms and common result fields. The implementation must cite exact tables in tests and must not convert an Operation Failed result into a transport/protocol failure.

## Set Defaults absence

Omission of Defaults Information is semantically meaningful: under the approved interpretation in `spec.md`, it requests removal of all Object Defaults from the server. The encoder must preserve absent field rather than serialize an empty Defaults Information structure.

## Scope limits

This contract does not implement the server-to-client Set Endpoint Role operation in §6.2.5, administrative policy discovery, remote object-state lookup, profile claiming, or any local cryptographic operation. Generic structurally valid TTLV access remains governed by the existing API and is not promoted into an operation-specific typed method here.
