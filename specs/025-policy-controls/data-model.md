# Data Model: KMIP 2.1 Policy and Usage Controls

This document describes the proposed Rust models and wire boundaries. Exact requirements come from `spec.md`; operation payload layouts cite the pinned OASIS Specification v2.1.

## Shared operation result

Every response is paired with the existing typed KMIP operation result and batch metadata. Preserve Result Status, Result Reason, permitted Result Message, async correlation, unknown values/extensions, and response delivery evidence. Do not create operation-specific result types that drop shared fields. Pending is a distinct result; no Poll or retry is implied.

## Operation payload models

| Model | Request fields | Response fields | Wire source |
| --- | --- | --- | --- |
| `GetConstraintsRequest` | No operation-specific members | `constraints: Structure` (required) | §6.1.22, Tables 229–230; Constraints §7.7 |
| `GetUsageAllocationRequest` | `unique_identifier: Option<UniqueIdentifier>`; `usage_limits_count: i64` (required, TTLV Long Integer) | `unique_identifier: UniqueIdentifier` (required) | §6.1.23, Tables 232–233; §4.59, Table 148 |
| `SetConstraintsRequest` | `constraints: Structure` (required) | Empty operation-specific payload | §6.1.52, Tables 325–326; Constraints §7.7 |
| `SetDefaultsRequest` | `defaults_information: Option<Structure>` | Empty operation-specific payload | §6.1.53, Tables 328–329; Defaults Information §7.11 |
| `SetEndpointRoleRequest` | `endpoint_role: OpenEnumeration` (required) | `endpoint_role: OpenEnumeration` (required) | §6.1.54, Tables 331–332; §11.19 |

Structure members retain TTLV order and unknown nested elements. `OpenEnumeration` means raw numeric values are retained, including future/vendor values. Optional absence remains distinct from an empty Structure. `Usage Limits Count` uses the KMIP Long Integer wire type (`i64`) specified by §4.59, Table 148; the operation codec must not encode it as Integer or unsigned Integer.

### Error forms

Preserve the source Result Status and each Result Reason alternative listed in Tables 231, 234, 327, 330, and 333. Duplicate source alternatives remain documented in `spec.md`; the implementation should not deduplicate or reject unknown future reasons. A malformed or incomplete network response is handled through the common sanitized protocol-response error and delivery state.

## Usage allocation context and guard

`UsageAllocationContext` is a caller-owned, non-cloneable local object containing:

| Field | Meaning | Source/constraint |
| --- | --- | --- |
| `unique_identifier` | Concrete object identifier to which this allocation applies | Required `Unique Identifier` in a successful response to §6.1.23 Table 232; includes server-resolved identifier when request used ID Placeholder. |
| `usage_limits_unit` | Open raw Enumeration supplied by caller from known Usage Limits object metadata | §4.59/Table 148 and §11.60/Table 491. The operation response does not carry it. Preserve unknown/vendor values. |
| `allocated_units` | Exact count placed in the successful Get Usage Allocation request | Required Table 232 Usage Limits Count. The response does not echo it. The client must not substitute a fetched or inferred value. |
| `locally_reserved_units` | Sum of costs retained after guarded requests may have been delivered | Local-only accounting; it is not remote-consumption evidence. |
| `available_units` | `allocated_units - locally_reserved_units`, checked with overflow-safe integer arithmetic | Guard check only; cannot be compared to the server's current remote count. |

A context can be created only after a complete successful Get Usage Allocation response and only if the caller provides the exact requested count and Usage Limits Unit. Failure, Pending, malformed response, absent resolved identifier, or invalid caller input cannot produce a context. A new successful allocation creates a new independent context; the client does not merge or reconcile contexts.

For one guarded operation, the caller provides a typed Encrypt, MAC, or Sign request and a strictly positive integer cost in the same unit. Require an explicit Unique Identifier equal to the context identifier. Missing ID, ID Placeholder, mismatch, zero/negative cost, or cost greater than `available_units` fails before writing request bytes with `NotSent`. The caller supplies cost for Byte, Object, and extension units; cost is never derived from input data length or an operation label.

Before exchange, add cost to local reservation. If execution returns `NotSent`, release the reservation. For `PossiblySent`, `ResponseStarted`, a complete Success/Failure/Pending result, or any other outcome without proof of zero transmission, retain it. This local conservative reservation prevents reusing possibly allocated units, but cannot determine remote consumption or actual protection. There is no decrement based on a response reason or success status.

The guard covers only calls explicitly routed through its guarded typed execution path in the current process. It does not inspect or count ordinary unguarded execution, generic TTLV, other client/process instances, server-side use, or use outside KMIPKit. Callers who know an object has Usage Limits are responsible for routing its protection requests through the selected context. No global counter or server-wide guarantee is represented.

## Invariants

- Required fields follow the exact table requiredness above and are validated before transmission.
- No response member is synthesized when absent; no unknown member is discarded solely because it is not modeled.
- Client calls send one explicit operation exchange and never retry or Poll automatically.
- A usage guard cannot be cloned or reused for another object or another unit.
- Guard accounting never claims that a remote server consumed the locally reserved amount.
- Secret/redaction and generic raw-body handling continue to follow the existing shared contracts.
