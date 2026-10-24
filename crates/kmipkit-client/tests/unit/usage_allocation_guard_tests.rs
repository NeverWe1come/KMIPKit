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

use crate::asynchronous_execution_test_support::client_for;
use crate::execute::encode_message_for_test;
use crate::execute_test_support::{asynchronous_response_bytes, test_item, test_structure};
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
        let payload = test_structure([test_item(
            UNIQUE_IDENTIFIER_TAG,
            kmipkit_ttlv::Value::text_string(identifier.to_owned()),
        )]);
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
    let (mut client, _, _) = client_for(ExchangeScript::Success {
        response: response_bytes(GET_USAGE_ALLOCATION, 0, None, None, Some(OBJECT_ID)),
        request_write_chunks: Vec::new(),
    });
    let response = client
        .get_usage_allocation(allocation_request(count), &CodecLimits::defaults())
        .expect("the allocation exchange succeeds");
    let response_view = response.outcome().response();
    let typed_response = response_view
        .get_usage_allocation()
        .expect("the exchange returns its typed allocation result");
    UsageAllocationContext::from_successful_allocation(typed_response, count, UNKNOWN_USAGE_UNIT)
        .expect("the caller supplies the exact requested count and known unit")
}

fn allocation_request(count: i64) -> GetUsageAllocationRequest {
    GetUsageAllocationRequest::new(count)
        .with_unique_identifier(UniqueIdentifier::TextString(OBJECT_ID.to_owned()))
}

fn execute_allocation(
    status: u32,
    correlation: Option<&[u8]>,
    identifier: Option<&str>,
) -> Result<ClientBatchResponse, crate::ClientError> {
    let (mut client, _, _) = client_for(ExchangeScript::Success {
        response: response_bytes(
            GET_USAGE_ALLOCATION,
            status,
            (status == 1).then_some(0x05),
            correlation,
            identifier,
        ),
        request_write_chunks: Vec::new(),
    });
    client.execute(
        ClientBatch::new(ClientBatchItem::new(ClientRequest::get_usage_allocation(
            allocation_request(ALLOCATION),
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
                allocation_request(ALLOCATION),
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
