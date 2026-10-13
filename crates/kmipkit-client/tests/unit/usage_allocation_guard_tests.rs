//! Project-derived guard tests: OASIS KMIP v2.1 §§4.59, 6.1.23, 11.60,
//! Tables 148, 232–233 and 491; approved local caller responsibility contract.

use crate::asynchronous_execution_test_support::client_for;
use crate::execute_test_support::{asynchronous_response_bytes, test_item, test_structure};
use crate::{ClientBatchOutcome, GuardedProtectionRequest, UsageAllocationContext};
use kmipkit_protocol::{
    EncryptRequest, GetUsageAllocationRequest, MacRequest, OperationData, SignRequest,
    UniqueIdentifier,
};
use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::RequestDeliveryState;
use kmipkit_ttlv::Value;
use kmipkit_ttlv::codec::CodecLimits;

static_assertions::assert_not_impl_any!(UsageAllocationContext: Clone, Copy);

fn context(count: i32, unit: u32) -> UsageAllocationContext {
    let response = asynchronous_response_bytes(
        0x11,
        0,
        None,
        None,
        Some(test_structure([test_item(
            0x420094,
            Value::text_string("allocated-object".into()),
        )])),
    );
    let (mut client, _, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![],
    });
    let result = client
        .get_usage_allocation(
            GetUsageAllocationRequest::new(None, count),
            &CodecLimits::defaults(),
        )
        .expect("allocation");
    let ClientBatchOutcome::GetUsageAllocation(response) = result.outcome() else {
        panic!("allocation result")
    };
    UsageAllocationContext::from_allocation(response, count, unit)
        .expect("complete successful allocation")
}

fn protection(identifier: Option<UniqueIdentifier>, variant: u8) -> GuardedProtectionRequest {
    match variant {
        0 => GuardedProtectionRequest::Encrypt(EncryptRequest::new(
            identifier,
            Some(OperationData::Integer(1)),
        )),
        1 => {
            let request = MacRequest::new().with_data(OperationData::Integer(1));
            GuardedProtectionRequest::Mac(if let Some(id) = identifier {
                request.with_unique_identifier(id)
            } else {
                request
            })
        }
        _ => {
            let request = SignRequest::new().with_data(OperationData::Integer(1));
            GuardedProtectionRequest::Sign(if let Some(id) = identifier {
                request.with_unique_identifier(id)
            } else {
                request
            })
        }
    }
}

#[test]
fn usage_allocation_guard_binds_resolved_id_exact_count_and_open_unit() {
    let context = context(i32::MAX, 0x80000003);
    assert_eq!(
        context.unique_identifier(),
        &UniqueIdentifier::TextString("allocated-object".into())
    );
    assert_eq!(context.usage_limits_unit(), 0x80000003);
    assert_eq!(context.allocated_units(), i32::MAX);
    assert_eq!(context.available_units(), i32::MAX);
    assert_eq!(context.locally_reserved_units(), 0);
    assert!(!format!("{context:?}").contains("allocated-object"));
}

#[test]
fn usage_allocation_guard_rejects_invalid_cost_and_id_without_transport() {
    for variant in [0, 1, 2] {
        for (identifier, cost) in [
            (
                Some(UniqueIdentifier::TextString("allocated-object".into())),
                0,
            ),
            (
                Some(UniqueIdentifier::TextString("allocated-object".into())),
                -1,
            ),
            (
                Some(UniqueIdentifier::TextString("allocated-object".into())),
                8,
            ),
            (Some(UniqueIdentifier::TextString("other-object".into())), 1),
            (Some(UniqueIdentifier::Enumeration(1)), 1),
            (Some(UniqueIdentifier::Integer(1)), 1),
            (None, 1),
        ] {
            let mut context = context(7, 1);
            let (mut client, transport, _) = client_for(ExchangeScript::FailBeforeWrite);
            let error = client
                .execute_guarded(
                    &mut context,
                    protection(identifier, variant),
                    cost,
                    None,
                    &CodecLimits::defaults(),
                )
                .expect_err("guard validation");
            assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
            assert_eq!(context.available_units(), 7);
            assert_eq!(transport.borrow().exchange_count(), 0);
            assert!(!format!("{error:?}").contains("other-object"));
        }
    }
}

#[test]
fn usage_allocation_guard_releases_only_proven_not_sent_reservations() {
    for variant in [0, 1, 2] {
        for (script, delivery, remaining) in [
            (
                ExchangeScript::FailBeforeWrite,
                RequestDeliveryState::NotSent,
                7,
            ),
            (
                ExchangeScript::FailAfterPartialWrite { written_bytes: 3 },
                RequestDeliveryState::PossiblySent,
                0,
            ),
            (
                ExchangeScript::FailAfterPartialRead {
                    written_bytes: 3,
                    response_bytes: vec![1],
                },
                RequestDeliveryState::ResponseStarted,
                0,
            ),
        ] {
            let mut context = context(7, 2);
            let (mut client, transport, _) = client_for(script);
            let error = client
                .execute_guarded(
                    &mut context,
                    protection(
                        Some(UniqueIdentifier::TextString("allocated-object".into())),
                        variant,
                    ),
                    7,
                    None,
                    &CodecLimits::defaults(),
                )
                .expect_err("scripted delivery");
            assert_eq!(error.delivery_state(), Some(delivery));
            assert_eq!(context.available_units(), remaining);
            assert_eq!(transport.borrow().exchange_count(), 1);
        }
    }
}

#[test]
fn usage_allocation_guard_retains_failure_and_pending_reservations() {
    for (operation, variant) in [(0x1f, 0), (0x23, 1), (0x21, 2)] {
        for status in [1, 2] {
            let mut context = context(7, 1);
            let response = asynchronous_response_bytes(
                operation,
                status,
                if status == 1 { Some(1) } else { None },
                if status == 2 {
                    Some(b"guard-pending")
                } else {
                    None
                },
                if status == 2 {
                    Some(test_structure([]))
                } else {
                    None
                },
            );
            let (mut client, transport, _) = client_for(ExchangeScript::Success {
                response,
                request_write_chunks: vec![],
            });
            let result = client
                .execute_guarded(
                    &mut context,
                    protection(
                        Some(UniqueIdentifier::TextString("allocated-object".into())),
                        variant,
                    ),
                    7,
                    Some(2),
                    &CodecLimits::defaults(),
                )
                .expect("complete result");
            assert_eq!(result.outcome().result().status().raw(), status);
            assert_eq!(context.available_units(), 0);
            assert_eq!(context.locally_reserved_units(), 7);
            assert_eq!(transport.borrow().exchange_count(), 1);
        }
    }
}

#[test]
fn usage_allocation_guard_cannot_be_created_from_failure_or_nonpositive_count() {
    for status in [0, 1] {
        let response = asynchronous_response_bytes(
            0x11,
            status,
            if status == 1 { Some(1) } else { None },
            None,
            if status == 0 {
                Some(test_structure([test_item(
                    0x420094,
                    Value::text_string("allocated-object".into()),
                )]))
            } else {
                None
            },
        );
        let (mut client, _, _) = client_for(ExchangeScript::Success {
            response,
            request_write_chunks: vec![],
        });
        let result = client
            .get_usage_allocation(
                GetUsageAllocationRequest::new(None, 7),
                &CodecLimits::defaults(),
            )
            .expect("result");
        let ClientBatchOutcome::GetUsageAllocation(response) = result.outcome() else {
            panic!("allocation")
        };
        for count in [-1, 0] {
            assert!(UsageAllocationContext::from_allocation(response, count, 1).is_err());
        }
        if status == 1 {
            assert!(UsageAllocationContext::from_allocation(response, 7, 1).is_err());
        }
    }
}
