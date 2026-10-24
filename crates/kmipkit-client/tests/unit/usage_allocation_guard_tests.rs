//! Project-derived checks for the explicit local guard in OASIS KMIP
//! Specification v2.1 §6.1.23 and approved interpretation KMIPKIT-025-FR-009.
//! These are not official OASIS test cases.

use kmipkit_protocol::{
    DecryptRequest, EncryptRequest, GetUsageAllocationRequest, HashRequest, MacRequest,
    MacVerifyRequest, OperationData, SecretBytes, SignRequest, SignatureVerifyRequest,
    UniqueIdentifier,
};
use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::RequestDeliveryState;
use kmipkit_ttlv::codec::CodecLimits;
use serde_json::Value as JsonValue;

use crate::asynchronous_execution_test_support::client_for;
use crate::execute::encode_message_for_test;
use crate::execute_test_support::{
    asynchronous_response_bytes, asynchronous_response_with_batch_id_bytes, test_item,
    test_structure,
};
use crate::{
    ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientBatchResponse, ClientErrorCategory,
    ClientRequest, UsageAllocationContext, UsageAllocationContextError,
};

const GET_USAGE_ALLOCATION: u32 = 0x0000_0011;
const UNIQUE_IDENTIFIER_TAG: u32 = 0x0042_0094;
const RESPONSE_HEADER: u32 = 0x0042_007A;
const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
const TIME_STAMP: u32 = 0x0042_0092;
const BATCH_COUNT: u32 = 0x0042_000D;
const BATCH_ITEM: u32 = 0x0042_000F;
const OPERATION: u32 = 0x0042_005C;
const RESULT_STATUS: u32 = 0x0042_007F;
const RESULT_REASON: u32 = 0x0042_007E;
const RESPONSE_PAYLOAD: u32 = 0x0042_007C;
const UNIQUE_BATCH_ITEM_ID: u32 = 0x0042_0093;
const ASYNCHRONOUS_CORRELATION_VALUE: u32 = 0x0042_0006;
const OBJECT_ID: &str = "usage-object-23";
const UNKNOWN_USAGE_UNIT: u32 = 0x8000_0054;
const ALLOCATION: i64 = 12;

fn response_bytes(
    operation: u32,
    status: u32,
    reason: Option<u32>,
    correlation: Option<&[u8]>,
    identifier: Option<&str>,
) -> Vec<u8> {
    response_bytes_with_batch_id(operation, status, reason, correlation, identifier, None)
}

fn response_bytes_with_batch_id(
    operation: u32,
    status: u32,
    reason: Option<u32>,
    correlation: Option<&[u8]>,
    identifier: Option<&str>,
    batch_item_id: Option<&[u8]>,
) -> Vec<u8> {
    response_bytes_with_identifier_value(
        operation,
        status,
        reason,
        correlation,
        identifier.map(|value| kmipkit_ttlv::Value::text_string(value.to_owned())),
        batch_item_id,
    )
}

fn response_bytes_with_identifier_value(
    operation: u32,
    status: u32,
    reason: Option<u32>,
    correlation: Option<&[u8]>,
    identifier: Option<kmipkit_ttlv::Value>,
    batch_item_id: Option<&[u8]>,
) -> Vec<u8> {
    let version = test_structure([
        test_item(PROTOCOL_VERSION_MAJOR, kmipkit_ttlv::Value::integer(2)),
        test_item(PROTOCOL_VERSION_MINOR, kmipkit_ttlv::Value::integer(1)),
    ]);
    let header = test_structure([
        test_item(PROTOCOL_VERSION, kmipkit_ttlv::Value::structure(version)),
        test_item(TIME_STAMP, kmipkit_ttlv::Value::date_time(1)),
        test_item(BATCH_COUNT, kmipkit_ttlv::Value::integer(1)),
    ]);
    let mut fields = vec![test_item(
        OPERATION,
        kmipkit_ttlv::Value::enumeration(operation),
    )];
    if let Some(batch_item_id) = batch_item_id {
        fields.push(test_item(
            UNIQUE_BATCH_ITEM_ID,
            kmipkit_ttlv::Value::byte_string(batch_item_id.to_vec()),
        ));
    }
    fields.push(test_item(
        RESULT_STATUS,
        kmipkit_ttlv::Value::enumeration(status),
    ));
    if let Some(reason) = reason {
        fields.push(test_item(
            RESULT_REASON,
            kmipkit_ttlv::Value::enumeration(reason),
        ));
    }
    if let Some(correlation) = correlation {
        fields.push(test_item(
            ASYNCHRONOUS_CORRELATION_VALUE,
            kmipkit_ttlv::Value::byte_string(correlation.to_vec()),
        ));
    }
    if let Some(identifier) = identifier {
        let payload = test_structure([test_item(UNIQUE_IDENTIFIER_TAG, identifier)]);
        fields.push(test_item(
            RESPONSE_PAYLOAD,
            kmipkit_ttlv::Value::structure(payload),
        ));
    }
    encode_message_for_test(
        test_structure([
            test_item(RESPONSE_HEADER, kmipkit_ttlv::Value::structure(header)),
            test_item(
                BATCH_ITEM,
                kmipkit_ttlv::Value::structure(test_structure(fields)),
            ),
        ]),
        &CodecLimits::defaults(),
    )
    .expect("project response fixture is valid TTLV")
}

fn allocate_context() -> UsageAllocationContext {
    allocate_context_with_count(ALLOCATION)
}

fn allocate_context_with_count(count: i64) -> UsageAllocationContext {
    allocate_context_for(count, UNKNOWN_USAGE_UNIT, OBJECT_ID)
}

fn allocate_context_for(
    count: i64,
    usage_limits_unit: u32,
    identifier: &str,
) -> UsageAllocationContext {
    allocate_context_for_with_exchange_count(count, usage_limits_unit, identifier).0
}

fn allocate_context_for_with_exchange_count(
    count: i64,
    usage_limits_unit: u32,
    identifier: &str,
) -> (UsageAllocationContext, usize) {
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response: response_bytes(GET_USAGE_ALLOCATION, 0, None, None, Some(identifier)),
        request_write_chunks: Vec::new(),
    });
    let response = client
        .get_usage_allocation(
            GetUsageAllocationRequest::new(count),
            &CodecLimits::defaults(),
        )
        .expect("the allocation exchange succeeds");
    let response_view = response.outcome().response();
    let typed_response = response_view
        .get_usage_allocation()
        .expect("the exchange returns its typed allocation result");
    let context = UsageAllocationContext::from_successful_allocation(
        typed_response,
        count,
        usage_limits_unit,
    )
    .expect("the caller supplies the exact requested count and known unit");
    (context, fake.borrow().exchange_count())
}

fn execute_allocation(
    status: u32,
    correlation: Option<&[u8]>,
    identifier: Option<&str>,
) -> Result<ClientBatchResponse, crate::ClientError> {
    execute_allocation_response(response_bytes(
        GET_USAGE_ALLOCATION,
        status,
        (status == 1).then_some(0x05),
        correlation,
        identifier,
    ))
}

fn execute_allocation_response(
    response: Vec<u8>,
) -> Result<ClientBatchResponse, crate::ClientError> {
    let (mut client, _, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    client.execute(
        ClientBatch::new(ClientBatchItem::new(ClientRequest::get_usage_allocation(
            GetUsageAllocationRequest::new(ALLOCATION),
        ))),
        &CodecLimits::defaults(),
    )
}

fn protection_request(operation: u32, identifier: Option<UniqueIdentifier>) -> ClientRequest {
    let data = || OperationData::ByteString(SecretBytes::new(vec![0x31, 0x32]));
    match operation {
        0x1F => ClientRequest::encrypt(EncryptRequest::new(identifier, Some(data()))),
        0x23 => {
            let request = MacRequest::new().with_data(data());
            ClientRequest::mac(match identifier {
                Some(identifier) => request.with_unique_identifier(identifier),
                None => request,
            })
        }
        0x21 => {
            let request = SignRequest::new().with_data(data());
            ClientRequest::sign(match identifier {
                Some(identifier) => request.with_unique_identifier(identifier),
                None => request,
            })
        }
        _ => unreachable!("only typed Encrypt, MAC, and Sign are test inputs"),
    }
}

fn unprotected_request(operation: u32) -> ClientRequest {
    let identifier = UniqueIdentifier::TextString(OBJECT_ID.to_owned());
    let data = || OperationData::ByteString(SecretBytes::new(vec![0x31, 0x32]));
    match operation {
        0x20 => ClientRequest::decrypt(DecryptRequest::new(Some(identifier), Some(data()))),
        0x27 => ClientRequest::hash(
            HashRequest::new(test_structure([test_item(
                0x0042_0038,
                kmipkit_ttlv::Value::enumeration(6),
            )]))
            .with_data(data()),
        ),
        0x24 => ClientRequest::mac_verify(
            MacVerifyRequest::new()
                .with_unique_identifier(identifier)
                .with_data(data())
                .with_mac_data(SecretBytes::new(vec![0xA1])),
        ),
        0x22 => ClientRequest::signature_verify(
            SignatureVerifyRequest::new()
                .with_unique_identifier(identifier)
                .with_data(data())
                .with_signature_data(SecretBytes::new(vec![0xA1])),
        ),
        _ => unreachable!("only decrypt, verification, and hash are test inputs"),
    }
}

#[test]
fn successful_allocation_binds_returned_identifier_and_open_caller_unit() {
    let context = allocate_context();

    assert_eq!(
        context.unique_identifier(),
        &UniqueIdentifier::TextString(OBJECT_ID.to_owned())
    );
    assert_eq!(context.usage_limits_unit(), UNKNOWN_USAGE_UNIT);
    assert_eq!(context.allocated_units(), ALLOCATION);
    assert_eq!(context.available_units(), ALLOCATION);
    assert_eq!(context.reserved_units(), 0);
}

// This parity scenario checks allocation, accepted use, and local rejection as one contract.
#[allow(clippy::too_many_lines)]
#[test]
fn fixture_usage_allocation_guard_matches_shared_accounting_contract() {
    // KMIP 2.1 §6.1.23 is the normative protocol source. The local reservation
    // behavior and accounting scope are project policy traced by this fixture.
    let fixture: JsonValue = serde_json::from_str(include_str!(
        "../../../../tests/fixtures/api/shared-capability-parity.json"
    ))
    .expect("shared API parity fixture is valid JSON");
    let case = fixture["cases"]
        .as_array()
        .expect("shared fixture cases are an array")
        .iter()
        .find(|case| case["id"] == "usage-allocation.guard-accounting.v1")
        .expect("shared Usage Allocation fixture case exists");
    let input = &case["input"];
    let identifier = input["allocationIdentifier"]
        .as_str()
        .expect("allocation identifier is text");
    let allocation_count = input["allocationCount"]
        .as_i64()
        .expect("allocation count is signed");
    let usage_limits_unit = u32::try_from(
        input["usageLimitsUnit"]
            .as_u64()
            .expect("usage limits unit is unsigned"),
    )
    .expect("usage limits unit fits the KMIP Enumeration");
    let operation = match input["operation"]
        .as_str()
        .expect("protection operation is text")
    {
        "Encrypt" => 0x1F,
        other => panic!("unsupported protection operation {other}"),
    };
    let successful_cost = input["successfulCost"]
        .as_i64()
        .expect("successful cost is signed");
    let over_budget_cost = input["overBudgetCost"]
        .as_i64()
        .expect("over-budget cost is signed");
    let expected = &case["expectedOutput"];
    assert_eq!(case["errorCategory"], "invalid_input");
    assert_eq!(expected["localOnlyAccounting"], true);

    let (mut context, allocation_exchanges) =
        allocate_context_for_with_exchange_count(allocation_count, usage_limits_unit, identifier);
    assert_eq!(allocation_exchanges, 1);
    let initial = &expected["initial"];
    assert_eq!(
        context.allocated_units(),
        initial["allocated"].as_i64().unwrap()
    );
    assert_eq!(
        context.reserved_units(),
        initial["reserved"].as_i64().unwrap()
    );
    assert_eq!(
        context.available_units(),
        initial["available"].as_i64().unwrap()
    );
    assert_eq!(context.usage_limits_unit(), usage_limits_unit);
    assert_eq!(
        context.unique_identifier(),
        &UniqueIdentifier::TextString(identifier.to_owned())
    );

    let (mut protection_client, protection_transport, _) = client_for(ExchangeScript::Success {
        response: response_bytes(operation, 0, None, None, Some(identifier)),
        request_write_chunks: Vec::new(),
    });
    let protection_response = protection_client
        .execute_guarded_protection(
            &mut context,
            protection_request(
                operation,
                Some(UniqueIdentifier::TextString(identifier.to_owned())),
            ),
            successful_cost,
            &CodecLimits::defaults(),
        )
        .expect("the fixture protection operation succeeds");
    assert_eq!(
        protection_response
            .outcome()
            .response()
            .result()
            .status()
            .raw(),
        0
    );
    assert_eq!(protection_transport.borrow().exchange_count(), 1);

    let after_success = &expected["afterSuccessfulProtection"];
    assert_eq!(
        context.allocated_units(),
        after_success["allocated"].as_i64().unwrap()
    );
    assert_eq!(
        context.reserved_units(),
        after_success["reserved"].as_i64().unwrap()
    );
    assert_eq!(
        context.available_units(),
        after_success["available"].as_i64().unwrap()
    );

    let state_before_rejection = (
        context.allocated_units(),
        context.reserved_units(),
        context.available_units(),
    );
    let (mut rejected_client, rejected_transport, _) = client_for(ExchangeScript::FailBeforeWrite);
    let rejection = rejected_client
        .execute_guarded_protection(
            &mut context,
            protection_request(
                operation,
                Some(UniqueIdentifier::TextString(identifier.to_owned())),
            ),
            over_budget_cost,
            &CodecLimits::defaults(),
        )
        .expect_err("the over-budget fixture attempt is rejected before transport");
    assert_eq!(rejection.category(), ClientErrorCategory::Validation);
    assert_eq!(
        rejection.delivery_state(),
        Some(RequestDeliveryState::NotSent)
    );
    assert_eq!(rejected_transport.borrow().exchange_count(), 0);
    assert_eq!(
        (
            context.allocated_units(),
            context.reserved_units(),
            context.available_units(),
        ),
        state_before_rejection,
        "rejecting an over-budget operation leaves local accounting unchanged"
    );
    assert_eq!(expected["overBudgetAttempt"]["accepted"], false);
    assert_eq!(expected["overBudgetAttempt"]["stateUnchanged"], true);
    assert_eq!(
        allocation_exchanges
            + protection_transport.borrow().exchange_count()
            + rejected_transport.borrow().exchange_count(),
        usize::try_from(
            expected["transportExchangeCount"]
                .as_u64()
                .expect("fixture transport exchange count is unsigned")
        )
        .expect("transport exchange count fits usize")
    );
}

#[test]
fn usage_allocation_cannot_bind_the_id_placeholder_as_an_object_identifier() {
    let allocation = execute_allocation_response(response_bytes_with_identifier_value(
        GET_USAGE_ALLOCATION,
        0,
        None,
        None,
        Some(kmipkit_ttlv::Value::enumeration(1)),
        None,
    ))
    .expect("the KMIP response carries a successful allocation result");
    let item = allocation.get(0).expect("one allocation result");
    let response_view = item.outcome().response();
    let response = response_view
        .get_usage_allocation()
        .expect("the successful allocation response has its typed payload");

    assert!(
        UsageAllocationContext::from_successful_allocation(
            response,
            ALLOCATION,
            UNKNOWN_USAGE_UNIT,
        )
        .is_err()
    );
}

#[test]
fn usage_context_debug_redacts_the_object_identifier() {
    let context = allocate_context();
    let debug = format!("{context:?}");

    assert!(!debug.contains(OBJECT_ID));
    assert!(debug.contains("has_unique_identifier: true"));
}

#[test]
fn unsuccessful_pending_malformed_and_nonpositive_allocations_cannot_create_contexts() {
    let failure =
        execute_allocation(1, None, None).expect("Failure remains a typed allocation response");
    let failure_item = failure.get(0).expect("one failure response");
    let failure_view = failure_item.outcome().response();
    let failure_response = failure_view
        .get_usage_allocation()
        .expect("Failure retains its typed operation response");
    assert!(
        UsageAllocationContext::from_successful_allocation(
            failure_response,
            ALLOCATION,
            UNKNOWN_USAGE_UNIT,
        )
        .is_err()
    );

    let (mut pending_client, _, _) = client_for(ExchangeScript::Success {
        response: asynchronous_response_bytes(
            GET_USAGE_ALLOCATION,
            2,
            None,
            Some(&[0xA5, 0x5A]),
            Some(test_structure([])),
        ),
        request_write_chunks: Vec::new(),
    });
    let pending = pending_client
        .execute(
            ClientBatch::new(ClientBatchItem::new(ClientRequest::get_usage_allocation(
                GetUsageAllocationRequest::new(ALLOCATION),
            )))
            .with_asynchronous_indicator(1),
            &CodecLimits::defaults(),
        )
        .expect("explicitly asynchronous allocation retains Pending");
    let pending = pending.get(0).expect("one pending response").outcome();
    assert!(matches!(pending, ClientBatchOutcome::Pending(_)));
    assert!(pending.response().get_usage_allocation().is_some());

    assert!(execute_allocation(0, None, None).is_err());

    let success = execute_allocation(0, None, Some(OBJECT_ID))
        .expect("successful allocation has a resolved identifier");
    let success_item = success.get(0).expect("one success response");
    let success_view = success_item.outcome().response();
    let response = success_view
        .get_usage_allocation()
        .expect("successful allocation retains its typed response");
    for invalid_count in [0, -1] {
        assert!(
            UsageAllocationContext::from_successful_allocation(
                response,
                invalid_count,
                UNKNOWN_USAGE_UNIT,
            )
            .is_err()
        );
    }
}

#[test]
fn usage_allocation_context_errors_have_payload_free_display_messages() {
    let cases = [
        (
            UsageAllocationContextError::AllocationNotSuccessful,
            "usage allocation response is not successful",
        ),
        (
            UsageAllocationContextError::InvalidAllocationCount,
            "usage allocation count must be positive",
        ),
        (
            UsageAllocationContextError::MissingResolvedIdentifier,
            "usage allocation response has no resolved identifier",
        ),
        (
            UsageAllocationContextError::InvalidUsageCost,
            "usage cost must be positive",
        ),
        (
            UsageAllocationContextError::CostExceedsAvailable,
            "usage cost exceeds locally available allocation",
        ),
        (
            UsageAllocationContextError::ReservationOverflow,
            "usage reservation exceeds local integer bounds",
        ),
        (
            UsageAllocationContextError::UnsupportedOperation,
            "operation is outside the guarded protection scope",
        ),
        (
            UsageAllocationContextError::MissingUniqueIdentifier,
            "guarded protection requires an explicit object identifier",
        ),
        (
            UsageAllocationContextError::MismatchedUniqueIdentifier,
            "guarded protection object does not match the allocation",
        ),
    ];

    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
        assert!(!error.to_string().contains(OBJECT_ID));
    }
}

#[test]
fn invalid_cost_and_identifier_are_rejected_before_transport() {
    let invalid_requests = [
        (
            protection_request(0x1F, Some(UniqueIdentifier::TextString(OBJECT_ID.into()))),
            0,
        ),
        (
            protection_request(0x23, Some(UniqueIdentifier::TextString(OBJECT_ID.into()))),
            -1,
        ),
        (
            protection_request(0x21, Some(UniqueIdentifier::TextString("other".into()))),
            1,
        ),
        // An omitted Unique Identifier would select the batch-scoped ID
        // Placeholder under KMIP §4.2.1; the guard requires a concrete ID.
        (protection_request(0x1F, None), 1),
        (
            protection_request(0x23, Some(UniqueIdentifier::Enumeration(0x0000_0001))),
            1,
        ),
    ];

    for (request, cost) in invalid_requests {
        let mut context = allocate_context();
        let (mut client, fake, _) = client_for(ExchangeScript::FailBeforeWrite);
        let error = client
            .execute_guarded_protection(&mut context, request, cost, &CodecLimits::defaults())
            .expect_err("invalid guard input fails without sending");
        assert_eq!(error.category(), ClientErrorCategory::Validation);
        assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
        assert_eq!(fake.borrow().exchange_count(), 0);
        assert_eq!(context.available_units(), ALLOCATION);
        assert_eq!(context.reserved_units(), 0);
    }
}

#[test]
fn each_typed_protection_operation_reserves_only_the_supplied_cost() {
    for (operation, units) in [(0x1F, 3), (0x23, 4), (0x21, 5)] {
        let mut context = allocate_context();
        let (mut client, fake, _) = client_for(ExchangeScript::Success {
            response: response_bytes(operation, 1, Some(0x05), None, None),
            request_write_chunks: Vec::new(),
        });
        let response = client
            .execute_guarded_protection(
                &mut context,
                protection_request(
                    operation,
                    Some(UniqueIdentifier::TextString(OBJECT_ID.to_owned())),
                ),
                units,
                &CodecLimits::defaults(),
            )
            .expect("a typed Failure is still a completed exchange");

        assert_eq!(fake.borrow().exchange_count(), 1);
        assert_eq!(response.outcome().response().result().status().raw(), 1);
        assert_eq!(context.reserved_units(), units);
        assert_eq!(context.available_units(), ALLOCATION - units);
    }
}

#[test]
fn over_budget_is_not_sent_and_not_sent_failure_releases_a_valid_reservation() {
    let mut context = allocate_context();
    let (mut no_send_client, no_send, _) = client_for(ExchangeScript::FailBeforeWrite);
    let not_sent = no_send_client
        .execute_guarded_protection(
            &mut context,
            protection_request(
                0x1F,
                Some(UniqueIdentifier::TextString(OBJECT_ID.to_owned())),
            ),
            ALLOCATION + 1,
            &CodecLimits::defaults(),
        )
        .expect_err("over-budget protection is rejected locally");
    assert_eq!(
        not_sent.delivery_state(),
        Some(RequestDeliveryState::NotSent)
    );
    assert_eq!(no_send.borrow().exchange_count(), 0);
    assert_eq!(context.reserved_units(), 0);

    let (mut failed_client, failed, _) = client_for(ExchangeScript::FailBeforeWrite);
    let sent_error = failed_client
        .execute_guarded_protection(
            &mut context,
            protection_request(
                0x1F,
                Some(UniqueIdentifier::TextString(OBJECT_ID.to_owned())),
            ),
            3,
            &CodecLimits::defaults(),
        )
        .expect_err("transport can fail before writing");
    assert_eq!(
        sent_error.delivery_state(),
        Some(RequestDeliveryState::NotSent)
    );
    assert_eq!(failed.borrow().exchange_count(), 1);
    assert_eq!(
        context.reserved_units(),
        0,
        "NotSent returns the reservation"
    );
    assert_eq!(context.available_units(), ALLOCATION);
}

#[test]
fn possibly_sent_and_response_started_errors_keep_reservations() {
    for script in [
        ExchangeScript::FailAfterPartialWrite { written_bytes: 1 },
        ExchangeScript::FailAfterPartialRead {
            written_bytes: usize::MAX,
            response_bytes: vec![0x42, 0x00],
        },
    ] {
        let mut context = allocate_context();
        let (mut client, _, _) = client_for(script);
        let error = client
            .execute_guarded_protection(
                &mut context,
                protection_request(
                    0x21,
                    Some(UniqueIdentifier::TextString(OBJECT_ID.to_owned())),
                ),
                2,
                &CodecLimits::defaults(),
            )
            .expect_err("ambiguous delivery cannot restore local allowance");
        assert!(matches!(
            error.delivery_state(),
            Some(RequestDeliveryState::PossiblySent | RequestDeliveryState::ResponseStarted)
        ));
        assert_eq!(context.reserved_units(), 2);
        assert_eq!(context.available_units(), ALLOCATION - 2);
    }
}

#[test]
fn exact_budget_is_accepted_and_reserved_as_locally_unavailable() {
    let mut context = allocate_context();
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response: response_bytes(0x1F, 1, Some(0x05), None, None),
        request_write_chunks: Vec::new(),
    });
    let response = client
        .execute_guarded_protection(
            &mut context,
            protection_request(
                0x1F,
                Some(UniqueIdentifier::TextString(OBJECT_ID.to_owned())),
            ),
            ALLOCATION,
            &CodecLimits::defaults(),
        )
        .expect("a positive cost exactly equal to the remaining allowance is accepted");
    assert_eq!(fake.borrow().exchange_count(), 1);
    assert_eq!(response.outcome().response().result().status().raw(), 1);
    assert_eq!(context.reserved_units(), ALLOCATION);
    assert_eq!(context.available_units(), 0);
}

#[test]
fn successful_guarded_protection_retains_local_reservation() {
    let mut context = allocate_context();
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response: response_bytes(0x1F, 0, None, None, Some(OBJECT_ID)),
        request_write_chunks: Vec::new(),
    });
    let response = client
        .execute_guarded_protection(
            &mut context,
            protection_request(
                0x1F,
                Some(UniqueIdentifier::TextString(OBJECT_ID.to_owned())),
            ),
            3,
            &CodecLimits::defaults(),
        )
        .expect("a complete Success response retains the local reservation");

    assert_eq!(fake.borrow().exchange_count(), 1);
    assert_eq!(response.outcome().response().result().status().raw(), 0);
    assert_eq!(context.reserved_units(), 3);
    assert_eq!(context.available_units(), ALLOCATION - 3);
}

#[test]
fn full_long_integer_allocation_can_be_reserved_without_overflow() {
    let mut context = allocate_context_with_count(i64::MAX);
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response: response_bytes(0x1F, 1, Some(0x05), None, None),
        request_write_chunks: Vec::new(),
    });
    let response = client
        .execute_guarded_protection(
            &mut context,
            protection_request(
                0x1F,
                Some(UniqueIdentifier::TextString(OBJECT_ID.to_owned())),
            ),
            i64::MAX,
            &CodecLimits::defaults(),
        )
        .expect("the exact positive Long Integer allocation is accepted");

    assert_eq!(fake.borrow().exchange_count(), 1);
    assert_eq!(response.outcome().response().result().status().raw(), 1);
    assert_eq!(context.reserved_units(), i64::MAX);
    assert_eq!(context.available_units(), 0);

    let (mut client, fake, _) = client_for(ExchangeScript::FailBeforeWrite);
    let error = client
        .execute_guarded_protection(
            &mut context,
            protection_request(
                0x1F,
                Some(UniqueIdentifier::TextString(OBJECT_ID.to_owned())),
            ),
            1,
            &CodecLimits::defaults(),
        )
        .expect_err("a reservation beyond i64::MAX fails before transport");
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(fake.borrow().exchange_count(), 0);
    assert_eq!(context.reserved_units(), i64::MAX);
}

#[test]
fn explicitly_asynchronous_guarded_protection_retains_pending_reservation() {
    let mut context = allocate_context();
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response: asynchronous_response_bytes(
            0x1F,
            2,
            None,
            Some(&[0xA5, 0x5A]),
            Some(test_structure([])),
        ),
        request_write_chunks: Vec::new(),
    });
    let outcome = client
        .execute_guarded_protection_with_asynchronous_indicator(
            &mut context,
            protection_request(
                0x1F,
                Some(UniqueIdentifier::TextString(OBJECT_ID.to_owned())),
            ),
            3,
            1,
            &CodecLimits::defaults(),
        )
        .expect("caller explicitly permits asynchronous completion");

    let ClientBatchOutcome::Pending(pending) = outcome.outcome() else {
        panic!("the explicitly permitted Pending result is retained");
    };
    assert_eq!(pending.asynchronous_correlation_value(), &[0xA5, 0x5A]);
    assert_eq!(fake.borrow().exchange_count(), 1);
    assert_eq!(context.reserved_units(), 3);
    assert_eq!(context.available_units(), ALLOCATION - 3);
}

#[test]
fn decrypt_verify_hash_and_ordinary_calls_do_not_consume_guard_allowance() {
    let mut context = allocate_context();
    for operation in [0x20, 0x24, 0x22, 0x27] {
        let mut request = unprotected_request(operation);
        let (mut guarded_client, guarded_fake, _) = client_for(ExchangeScript::FailBeforeWrite);
        let error = guarded_client
            .execute_guarded_protection(&mut context, request, 1, &CodecLimits::defaults())
            .expect_err("non-protection operations are outside the guard");
        assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
        assert_eq!(guarded_fake.borrow().exchange_count(), 0);
        assert_eq!(context.reserved_units(), 0);

        request = unprotected_request(operation);
        let (mut ordinary_client, ordinary_fake, _) = client_for(ExchangeScript::Success {
            response: response_bytes(operation, 1, Some(0x05), None, None),
            request_write_chunks: Vec::new(),
        });
        ordinary_client
            .execute(
                ClientBatch::new(ClientBatchItem::new(request)),
                &CodecLimits::defaults(),
            )
            .expect("ordinary typed processing remains available outside the guard");
        assert_eq!(ordinary_fake.borrow().exchange_count(), 1);
        assert_eq!(context.reserved_units(), 0);
        assert_eq!(context.available_units(), ALLOCATION);
    }

    // OASIS §6.1.23 obligations apply to protection uses, but this explicitly
    // caller-owned context accounts only for calls routed through its guard.
    for operation in [0x1F, 0x23, 0x21] {
        let (mut ordinary_client, ordinary_fake, _) = client_for(ExchangeScript::Success {
            response: response_bytes(operation, 1, Some(0x05), None, None),
            request_write_chunks: Vec::new(),
        });
        ordinary_client
            .execute(
                ClientBatch::new(ClientBatchItem::new(protection_request(
                    operation,
                    Some(UniqueIdentifier::TextString(OBJECT_ID.to_owned())),
                ))),
                &CodecLimits::defaults(),
            )
            .expect("ordinary protection remains outside this explicit local context");
        assert_eq!(ordinary_fake.borrow().exchange_count(), 1);
        assert_eq!(context.reserved_units(), 0);
        assert_eq!(context.available_units(), ALLOCATION);
    }
}

#[test]
fn guarded_batch_item_returns_standard_batch_and_preserves_request_metadata() {
    let mut context = allocate_context();
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response: response_bytes_with_batch_id(
            0x1F,
            1,
            Some(0x05),
            None,
            None,
            Some(&[0xA5, 0x5A]),
        ),
        request_write_chunks: Vec::new(),
    });
    let batch_id = vec![0xA5, 0x5A];
    let item = ClientBatchItem::new(protection_request(
        0x1F,
        Some(UniqueIdentifier::TextString(OBJECT_ID.to_owned())),
    ))
    .with_unique_batch_item_id(batch_id.clone());

    let response = client
        .execute_guarded_protection_item(&mut context, item, 3, &CodecLimits::defaults())
        .expect("the guarded item uses the standard one-item batch result");

    assert_eq!(response.len(), 1);
    assert_eq!(
        response
            .get(0)
            .map(|item| item.outcome().response().result().status().raw()),
        Some(1)
    );
    assert_eq!(
        response.get(0).and_then(|item| item.unique_batch_item_id()),
        Some(batch_id.as_slice())
    );
    assert_eq!(context.reserved_units(), 3);
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn guarded_batch_item_pending_preserves_metadata_and_retains_reservation() {
    let mut context = allocate_context();
    let correlation = [0xA5, 0x5A, 0xC3];
    let batch_id = vec![0xB1, 0xB2];
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response: asynchronous_response_with_batch_id_bytes(
            0x1F,
            2,
            None,
            Some(&correlation),
            Some(&batch_id),
            Some(test_structure([])),
        ),
        request_write_chunks: Vec::new(),
    });
    let item = ClientBatchItem::new(protection_request(
        0x1F,
        Some(UniqueIdentifier::TextString(OBJECT_ID.to_owned())),
    ))
    .with_unique_batch_item_id(batch_id.clone());

    let response = client
        .execute_guarded_protection_item_with_asynchronous_indicator(
            &mut context,
            item,
            3,
            1,
            &CodecLimits::defaults(),
        )
        .expect("the explicit asynchronous indicator permits Pending");
    let item_response = response.get(0).expect("one Pending response");
    let ClientBatchOutcome::Pending(pending) = item_response.outcome() else {
        panic!("the operation returns its standard Pending outcome");
    };

    assert_eq!(pending.asynchronous_correlation_value(), correlation);
    assert_eq!(
        item_response.unique_batch_item_id(),
        Some(batch_id.as_slice())
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
    assert_eq!(context.reserved_units(), 3);
    assert_eq!(context.available_units(), ALLOCATION - 3);
}

#[test]
fn guarded_batch_item_rejects_invalid_scope_and_cost_before_transport() {
    let valid_identifier = || UniqueIdentifier::TextString(OBJECT_ID.to_owned());
    let invalid_cases = [
        (unprotected_request(0x20), 1),
        (protection_request(0x1F, None), 1),
        (
            protection_request(0x1F, Some(UniqueIdentifier::Enumeration(1))),
            1,
        ),
        (
            protection_request(
                0x23,
                Some(UniqueIdentifier::TextString("different-object".to_owned())),
            ),
            1,
        ),
        (protection_request(0x21, Some(valid_identifier())), 0),
        (
            protection_request(0x1F, Some(valid_identifier())),
            ALLOCATION + 1,
        ),
    ];

    for (request, cost) in invalid_cases {
        let mut context = allocate_context();
        let (mut client, fake, _) = client_for(ExchangeScript::FailBeforeWrite);
        let item = ClientBatchItem::new(request);
        let error = client
            .execute_guarded_protection_item(&mut context, item, cost, &CodecLimits::defaults())
            .expect_err("invalid item scope or cost fails before transport");

        assert_eq!(error.category(), ClientErrorCategory::Validation);
        assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
        assert_eq!(fake.borrow().exchange_count(), 0);
        assert_eq!(context.reserved_units(), 0);
        assert_eq!(context.available_units(), ALLOCATION);
    }
}

#[test]
fn guarded_batch_item_transport_failures_settle_reservations_by_delivery_state() {
    let cases = [
        (
            ExchangeScript::FailBeforeWrite,
            RequestDeliveryState::NotSent,
            0,
        ),
        (
            ExchangeScript::FailAfterPartialWrite { written_bytes: 1 },
            RequestDeliveryState::PossiblySent,
            2,
        ),
        (
            ExchangeScript::FailAfterPartialRead {
                written_bytes: usize::MAX,
                response_bytes: vec![0x42, 0x00],
            },
            RequestDeliveryState::ResponseStarted,
            2,
        ),
    ];

    for (script, expected_state, expected_reservation) in cases {
        let mut context = allocate_context();
        let (mut client, fake, _) = client_for(script);
        let item = ClientBatchItem::new(protection_request(
            0x21,
            Some(UniqueIdentifier::TextString(OBJECT_ID.to_owned())),
        ));
        let error = client
            .execute_guarded_protection_item(&mut context, item, 2, &CodecLimits::defaults())
            .expect_err("the scripted transport failure is surfaced");

        assert_eq!(error.delivery_state(), Some(expected_state));
        assert_eq!(fake.borrow().exchange_count(), 1);
        assert_eq!(context.reserved_units(), expected_reservation);
        assert_eq!(context.available_units(), ALLOCATION - expected_reservation);
    }
}
