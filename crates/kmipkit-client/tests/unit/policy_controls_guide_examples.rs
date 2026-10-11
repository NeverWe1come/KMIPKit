//! Compile-check a Rust mirror of the policy-control examples in the English
//! and Spanish user guides. Assertions bind the mirror to both Markdown guides.
//! Examples follow OASIS KMIP v2.1 §§6.1.22, 6.1.23, 6.1.52, and 6.1.53; they
//! are project examples, not official conformance tests.

use std::error::Error;

use kmipkit_protocol::{
    GetConstraintsRequest, GetUsageAllocationRequest, SetConstraintsRequest, SetDefaultsRequest,
    UniqueIdentifier,
};
use kmipkit_ttlv::{Structure, codec::CodecLimits};

use crate::{Client, ClientBatchItemResponse, ClientError, ClientRequest, UsageAllocationContext};

const ENGLISH_GUIDE: &str = include_str!("../../../../docs/user-guide/en/policy-controls.md");
const SPANISH_GUIDE: &str = include_str!("../../../../docs/user-guide/es/controles-de-uso.md");

type AllocationExample = fn(
    &mut Client,
    UniqueIdentifier,
    i64,
    u32,
    &CodecLimits,
) -> Result<UsageAllocationContext, Box<dyn Error>>;
type GuardedEncryptExample = fn(
    &mut Client,
    &mut UsageAllocationContext,
    kmipkit_protocol::EncryptRequest,
    i64,
    &CodecLimits,
) -> Result<ClientBatchItemResponse, ClientError>;

fn build_policy_payloads(
    object_id: UniqueIdentifier,
    constraints: Structure,
    defaults: Structure,
) -> Result<(), Box<dyn Error>> {
    let _get_constraints = GetConstraintsRequest::new().to_ttlv_payload()?;
    let _get_allocation = GetUsageAllocationRequest::new(25)
        .with_unique_identifier(object_id)
        .to_ttlv_payload()?;
    let _set_constraints = SetConstraintsRequest::new(constraints).to_ttlv_payload()?;
    let _remove_all_defaults = SetDefaultsRequest::new().to_ttlv_payload()?;
    let _set_empty_defaults = SetDefaultsRequest::new()
        .with_defaults_information(defaults)
        .to_ttlv_payload()?;
    Ok(())
}

fn allocate_for_object(
    client: &mut Client,
    object_id: UniqueIdentifier,
    requested_units: i64,
    usage_limits_unit: u32,
    limits: &CodecLimits,
) -> Result<UsageAllocationContext, Box<dyn Error>> {
    let response = client.get_usage_allocation(
        GetUsageAllocationRequest::new(requested_units).with_unique_identifier(object_id),
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
    request: kmipkit_protocol::EncryptRequest,
    caller_supplied_cost: i64,
    limits: &CodecLimits,
) -> Result<ClientBatchItemResponse, ClientError> {
    client.execute_guarded_protection(
        allocation,
        ClientRequest::encrypt(request),
        caller_supplied_cost,
        limits,
    )
}

#[test]
fn documented_policy_payloads_preserve_defaults_omission_semantics() {
    for guide in [ENGLISH_GUIDE, SPANISH_GUIDE] {
        assert!(guide.contains("GetUsageAllocationRequest::new(25)"));
        assert!(guide.contains("with_defaults_information(defaults)"));
        assert!(guide.contains("execute_guarded_protection"));
        assert!(guide.contains("local"));
    }

    build_policy_payloads(
        UniqueIdentifier::TextString("policy-object".to_owned()),
        Structure::new(),
        Structure::new(),
    )
    .expect("the guide's caller-supplied policy Structures are representable");

    let omitted = SetDefaultsRequest::new()
        .to_ttlv_payload()
        .expect("omitting Defaults Information is representable");
    let explicit_empty = SetDefaultsRequest::new()
        .with_defaults_information(Structure::new())
        .to_ttlv_payload()
        .expect("an explicitly empty Defaults Information Structure is representable");

    assert!(omitted.view().children().is_empty());
    assert_eq!(explicit_empty.view().children().len(), 1);
}

#[test]
fn documented_guard_examples_compile_against_the_client_api() {
    let _: AllocationExample = allocate_for_object;
    let _: GuardedEncryptExample = encrypt_with_allocation;
}
