//! Compile-check a Rust mirror of the policy-control examples in the English
//! and Spanish user guides. Assertions bind the mirror to both Markdown guides.
//! Examples follow OASIS KMIP v2.1 §§6.1.22, 6.1.23, 6.1.52, and 6.1.53; they
//! are project examples, not official conformance tests.

use std::error::Error;

use kmipkit_protocol::{
    GetConstraintsRequest, GetUsageAllocationRequest, SetConstraintsRequest, SetDefaultsRequest,
    UniqueIdentifier,
};
use kmipkit_ttlv::{Structure, Value, codec::CodecLimits};

use crate::execute_test_support::{test_item, test_structure};
use crate::{Client, ClientBatchItemResponse, ClientError, ClientRequest, UsageAllocationContext};

const ENGLISH_GUIDE: &str = include_str!("../../../../docs/user-guide/en/policy-controls.md");
const SPANISH_GUIDE: &str = include_str!("../../../../docs/user-guide/es/controles-de-uso.md");
const DEBUG_SENTINEL: &str = "P025-DEBUG-SECRET-SENTINEL";
const CONSTRAINT_TAG: u32 = 0x0042_0169;
const ATTRIBUTES_TAG: u32 = 0x0042_0025;
const ATTRIBUTE_TAG: u32 = 0x0042_0008;
const VENDOR_IDENTIFICATION_TAG: u32 = 0x0042_009D;
const ATTRIBUTE_NAME_TAG: u32 = 0x0042_000A;
const ATTRIBUTE_VALUE_TAG: u32 = 0x0042_000B;
const OBJECT_DEFAULTS_TAG: u32 = 0x0042_0153;
const OBJECT_TYPE_TAG: u32 = 0x0042_0057;

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

fn vendor_attribute_with_debug_sentinel() -> Structure {
    test_structure([test_item(
        ATTRIBUTE_TAG,
        Value::structure(test_structure([
            test_item(
                VENDOR_IDENTIFICATION_TAG,
                Value::text_string("KMIPKit test vendor".to_owned()),
            ),
            test_item(
                ATTRIBUTE_NAME_TAG,
                Value::text_string("Debug Redaction Sentinel".to_owned()),
            ),
            test_item(
                ATTRIBUTE_VALUE_TAG,
                Value::byte_string(DEBUG_SENTINEL.as_bytes().to_vec()),
            ),
        ])),
    )])
}

fn constraints_with_debug_sentinel() -> Structure {
    test_structure([test_item(
        CONSTRAINT_TAG,
        Value::structure(test_structure([test_item(
            ATTRIBUTES_TAG,
            Value::structure(vendor_attribute_with_debug_sentinel()),
        )])),
    )])
}

fn defaults_with_debug_sentinel() -> Structure {
    test_structure([test_item(
        OBJECT_DEFAULTS_TAG,
        Value::structure(test_structure([
            test_item(OBJECT_TYPE_TAG, Value::enumeration(2)),
            test_item(
                ATTRIBUTES_TAG,
                Value::structure(vendor_attribute_with_debug_sentinel()),
            ),
        ])),
    )])
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
fn policy_request_debug_uses_operation_names_without_request_payloads() {
    let constraints = constraints_with_debug_sentinel();
    let defaults = defaults_with_debug_sentinel();
    let requests = [
        (ClientRequest::get_constraints(), "GetConstraints", false),
        (
            ClientRequest::set_constraints(SetConstraintsRequest::new(constraints)),
            "SetConstraints(",
            true,
        ),
        (
            ClientRequest::set_defaults(
                SetDefaultsRequest::new().with_defaults_information(defaults),
            ),
            "SetDefaults(",
            true,
        ),
        (
            ClientRequest::get_usage_allocation(GetUsageAllocationRequest::new(1)),
            "GetUsageAllocation(",
            false,
        ),
    ];

    for (request, expected_prefix, has_sensitive_payload) in requests {
        let formatted = format!("{request:?}");
        assert!(
            formatted.starts_with(expected_prefix),
            "{formatted:?} starts with {expected_prefix:?}"
        );
        assert!(
            !formatted.contains(DEBUG_SENTINEL),
            "request Debug exposed a supplied payload: {formatted}"
        );
        if has_sensitive_payload {
            assert!(
                formatted.contains("[REDACTED]"),
                "request Debug must mark payload redaction: {formatted}"
            );
        }
    }
}

#[test]
fn documented_guard_examples_compile_against_the_client_api() {
    let _: AllocationExample = allocate_for_object;
    let _: GuardedEncryptExample = encrypt_with_allocation;
}
