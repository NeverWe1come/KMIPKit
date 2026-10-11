# Usage Allocation Guard Contract

## Purpose and source

This explicit caller-supplied local guard exposes the client obligations recorded in OASIS Specification v2.1 §6.1.23 and the approved interpretation in `spec.md`. It is not a server-side allocation manager and does not claim to observe actual remote consumption.

## Creating a context

A context can be created only from one complete successful Get Usage Allocation exchange and these caller-known inputs:

1. The exact positive Usage Limits Count Integer sent in that request.
2. The open Usage Limits Unit Enumeration known from the target object's Usage Limits attribute (§4.59/Table 148 and §11.60/Table 491).
3. The required Unique Identifier returned by the server (Table 233).

The context is bound to this resolved identifier and unit and is not cloneable. It cannot be created from a Failure, Pending result, malformed response, or response without a resolved identifier. If the request omitted Unique Identifier, use the concrete response identifier, not ID Placeholder. The caller supplies the requested count and unit because the response omits them.

This context makes no claim that the server granted the allocation because of count alone: it must be created only after a complete successful result. It does not fetch or confirm Usage Limits metadata, activation/protect-stop dates, or remote allowance.

## Guarded operations and caller cost

Only typed Encrypt, MAC, and Sign calls are guarded. Decrypt, MAC Verify, Signature Verify, and Hash are excluded. One guarded call accepts one typed operation request; it does not accept an arbitrary batch, generic TTLV Structure, or raw bytes. Multipart operations are charged per explicit request call, and the caller supplies the cost for every part.

Before transport, require an explicit concrete Unique Identifier equal to the context identifier. Reject missing, ID Placeholder, and mismatched IDs. The caller supplies a strictly positive cost in the exact context unit. Reject cost greater than the locally available amount before any request byte is sent. Costs use checked arithmetic and are represented within the positive range of KMIP Integer. Do not infer cost from data length or operation type, round units, or convert between Byte/Object/vendor units. Preserve unknown unit values.

## Reservation transitions

1. Validate object identifier, cost, and available allowance; on failure, return a sanitized `NotSent` error and do not call transport.
2. Reserve cost in local context before beginning the exchange.
3. If final execution evidence is `NotSent`, release the reservation.
4. If request bytes may have been sent (`PossiblySent`), response bytes began (`ResponseStarted`), or a complete Success, Failure, or Pending response is returned, retain the reservation as locally unavailable.

Retained amount is a conservative local reservation. It is not labeled consumed or reconciled with the server. A timeout or ambiguous I/O error after possible transmission does not release it. A subsequent allocation creates another context; no automatic retry, reallocation, reset, or reconciliation occurs.

## Responsibility boundary

The caller is responsible for routing every protection use it knows is for a Usage-Limited object through its selected context and for providing an accurate unit cost. This guard counts only calls through that context in this local process. Ordinary unguarded typed calls, generic TTLV, another process/client, external applications, and server-side activity are not observed or counted. Therefore the guard is not a global or server-wide quota guarantee.

## Required verification

- A guard is created after successful allocation using returned object ID plus explicit requested count/unit; failed, Pending, malformed, and unresolved-ID results cannot create it.
- Exact remaining cost is accepted; a greater cost is rejected before transport.
- Zero/negative cost, overflow, absent ID, placeholder ID, wrong ID, and unsupported operation variant are rejected before transport.
- `NotSent` releases a reservation; `PossiblySent`, `ResponseStarted`, Success, Failure, and Pending retain it.
- Decrypt, MAC Verify, Signature Verify, and Hash do not consume this guard.
- Guard context is not cloneable; different contexts remain independent and do not assert remote allocation totals.
- Unguarded/raw/external activity does not affect this instance, and docs never describe local reservation as server-consumed usage.
