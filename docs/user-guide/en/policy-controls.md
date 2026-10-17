# Policy and Usage Controls

KMIPKit implements the client-initiated Get Constraints, Get Usage Allocation, Set Constraints, and Set Defaults operations from OASIS KMIP v2.1. The request and response layouts follow §§6.1.22, 6.1.23, 6.1.52, and 6.1.53. These operations transmit and expose server policy; KMIPKit does not evaluate or enforce server-side constraints locally.

Get Usage Allocation is mapped across Rust, C, Java, and Python. Its request
preserves the optional Unique Identifier and exact signed 64-bit Usage Limits
Count; a successful result exposes the resolved identifier. The common batch
result preserves Pending and Failure, correlation, and delivery state without
automatic polling or retry. On success, the response also exposes its complete
ordered generic TTLV, including unknown fields. Rust lends a view; C returns a
caller-owned structure that must be released with
`kmipkit_ttlv_structure_release`; Java returns a `TtlvStructure` owned by the
closeable result; and Python exposes `response_ttlv` on the closeable response.
The C, Java, and Python typed accessors do not return a generic response for a
Failure or Pending result.

## Build policy requests

The protocol layer accepts the ordered TTLV `Structure` supplied by the caller. Fill Constraints and Defaults Information with values appropriate to the server and your policy. This example shows payload construction without choosing policy values:

```rust
use kmipkit_protocol::{
    GetConstraintsRequest, GetUsageAllocationRequest, SetConstraintsRequest,
    SetDefaultsRequest, UniqueIdentifier,
};
use kmipkit_ttlv::Structure;

fn build_policy_payloads(
    object_id: UniqueIdentifier,
    constraints: Structure,
    defaults: Structure,
) -> Result<(), Box<dyn std::error::Error>> {
    let _get_constraints = GetConstraintsRequest::new().to_ttlv_payload()?;

    // Usage Limits Count is a signed Long Integer (i64), per §7.40, Table 392.
    let _get_allocation = GetUsageAllocationRequest::new(25)
        .with_unique_identifier(object_id)
        .to_ttlv_payload()?;

    let _set_constraints = SetConstraintsRequest::new(constraints).to_ttlv_payload()?;

    // Omission means remove all Object Defaults (Table 328).
    let _remove_all_defaults = SetDefaultsRequest::new().to_ttlv_payload()?;

    // An explicitly supplied empty Structure is distinct from omission.
    let _set_empty_defaults = SetDefaultsRequest::new()
        .with_defaults_information(defaults)
        .to_ttlv_payload()?;
    Ok(())
}
```

With a configured `Client`, call `get_constraints`, `get_usage_allocation`, `set_constraints`, or `set_defaults`. Each convenience method performs one exchange and does not retry or poll automatically. Inspect the operation-specific response through `response.outcome().response()`; preserve `Pending` and its correlation value when using an asynchronous batch.

## Guard usage-limited protection

OASIS §6.1.23 places usage allocation obligations on the client. The caller supplies the exact positive allocation count and the Usage Limits Unit learned from trusted object metadata; the allocation response supplies the resolved object identifier. KMIPKit does not retrieve those object attributes or infer a protection cost.

```rust
use kmipkit_client::{
    Client, ClientBatchItemResponse, ClientError, ClientRequest, UsageAllocationContext,
};
use kmipkit_protocol::{EncryptRequest, GetUsageAllocationRequest, UniqueIdentifier};
use kmipkit_ttlv::codec::CodecLimits;

fn allocate_for_object(
    client: &mut Client,
    object_id: UniqueIdentifier,
    requested_units: i64,
    usage_limits_unit: u32,
    limits: &CodecLimits,
) -> Result<UsageAllocationContext, Box<dyn std::error::Error>> {
    let response = client.get_usage_allocation(
        GetUsageAllocationRequest::new(requested_units)
            .with_unique_identifier(object_id),
        limits,
    )?;
    let response_view = response.outcome().response();
    let allocation = response_view
        .get_usage_allocation()
        .ok_or("expected Get Usage Allocation response")?;
    Ok(UsageAllocationContext::from_successful_allocation(
        allocation,
        requested_units,
        usage_limits_unit,
    )?)
}

fn encrypt_with_allocation(
    client: &mut Client,
    allocation: &mut UsageAllocationContext,
    request: EncryptRequest,
    caller_supplied_cost: i64,
    limits: &CodecLimits,
) -> Result<ClientBatchItemResponse, ClientError> {
    // Construct `request` with explicit cryptographic parameters and the same
    // object identifier bound to `allocation`.
    client.execute_guarded_protection(
        allocation,
        ClientRequest::encrypt(request),
        caller_supplied_cost,
        limits,
    )
}
```

The guard accepts only typed Encrypt, MAC, and Sign requests with an explicit identifier matching the allocation. It rejects non-positive costs and costs above the locally available allowance before calling the transport. It releases a reservation only when delivery is known to be `NotSent`; an ambiguous failure, response, or `Pending` retains the reservation locally.

This accounting covers only requests sent through the supplied context in this process. It does not observe unguarded calls, Decrypt, Hash, verification operations, another client, another process, or server-side consumption. A local reservation is not evidence that the server used those units. Do not request an allocation outside the object's Activation Date and Protect Stop Date window; the caller must know and observe that window.

These are project-derived examples based on OASIS operation tables, not official OASIS conformance tests. See [Client execution](client-execution.md) for transport setup and delivery results.
