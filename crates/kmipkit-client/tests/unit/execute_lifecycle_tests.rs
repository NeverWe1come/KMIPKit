//! End-to-end tests for the synchronous `Client::execute` lifecycle boundary.
//!
//! These are derived project-policy tests, not official OASIS vectors. The
//! Pending Asynchronous Correlation Value is required immediately by OASIS
//! KMIP v2.1 §8.6, Table 399, catalog element
//! `KMIPKIT-ELEM-MESSAGE-FIELD-8-6-ASYNCHRONOUS-CORRELATION-VALUE`; §9.19 and
//! `KMIPKIT-REQ-SPEC-9.19-002` concern its later use by Poll. Discover Versions
//! has no secret payload and does not resolve ADR-0012 OD-006, which remains a
//! gate for the first secret-bearing operation.
//!
//! Traceability: `KMIPKIT-0007-FR-005`, `-FR-008`, `-FR-011`, `-FR-012`,
//! `-FR-013`, `-FR-017`; `KMIPKIT-0007-SC-002`, `-SC-004`, `-SC-005`,
//! `-SC-008`; ADR-0012; `AGENTS.md` §8.

use std::cell::{Cell, RefCell};
use std::error::Error;
use std::rc::Rc;

use kmipkit_test_support::{ExchangeScript, ScriptedTransport};
use kmipkit_transport::{RequestDeliveryState, Transport, TransportError, TransportResponse};
use kmipkit_ttlv::codec::CodecLimits;

use crate::ClientErrorCategory;
use crate::execute::{Client, ClientBatch, ClientBatchItem, ClientRequest, ZeroizationObserver};
use crate::execute_test_support::{ResponseItemFixture, response_bytes};

const CORRELATION_VALUE: &[u8] = b"KMIP_ASYNC_CORRELATION_SENTINEL_73";
const CORRELATION_PREFIX: &str = "KMIP_ASYNC_CORRELATION";

#[derive(Clone)]
struct SharedScriptedTransport {
    fake: Rc<RefCell<ScriptedTransport>>,
    request_owner_observer: Option<ZeroizationObserver>,
    owner_live_when_exchange_returns: Rc<Cell<bool>>,
}

impl Transport for SharedScriptedTransport {
    fn exchange(
        &mut self,
        request: &[u8],
        max_response_bytes: usize,
    ) -> Result<TransportResponse, TransportError> {
        let result = self.fake.borrow_mut().exchange(request, max_response_bytes);
        if let Some(observer) = &self.request_owner_observer {
            self.owner_live_when_exchange_returns
                .set(observer.result().is_none());
        }
        result
    }
}

fn client_for(
    script: ExchangeScript,
    request_observer: Option<ZeroizationObserver>,
) -> (Client, Rc<RefCell<ScriptedTransport>>, Rc<Cell<bool>>) {
    let fake = Rc::new(RefCell::new(ScriptedTransport::new(script)));
    let owner_live = Rc::new(Cell::new(false));
    let transport = SharedScriptedTransport {
        fake: Rc::clone(&fake),
        request_owner_observer: request_observer.clone(),
        owner_live_when_exchange_returns: Rc::clone(&owner_live),
    };
    let client = if let Some(observer) = request_observer {
        Client::for_test_with_request_observer(transport, observer)
    } else {
        Client::for_test(transport)
    };
    (client, fake, owner_live)
}

fn discover_versions_batch(asynchronous_indicator: Option<u32>) -> ClientBatch {
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::discover_versions()));
    match asynchronous_indicator {
        Some(value) => batch.with_asynchronous_indicator(value),
        None => batch,
    }
}

#[test]
fn public_batch_and_result_accessors_preserve_order_and_redact_identifiers() {
    let id = b"PUBLIC_BATCH_ID_SENTINEL".to_vec();
    let first = ClientBatchItem::new(ClientRequest::discover_versions())
        .with_unique_batch_item_id(id.clone());
    let second = ClientBatchItem::new(ClientRequest::discover_versions());
    assert!(matches!(
        first.request(),
        ClientRequest::DiscoverVersions(_)
    ));
    assert_eq!(first.unique_batch_item_id(), Some(id.as_slice()));
    assert_eq!(second.unique_batch_item_id(), None);
    assert!(!format!("{first:?}").contains("PUBLIC_BATCH_ID_SENTINEL"));

    let mut request = ClientBatch::from_items([first]);
    request.push(second);
    let request = request
        .with_request_time_stamp(1_700_000_000)
        .with_batch_order_option(true);
    assert_eq!(request.items().len(), 2);
    assert_eq!(request.request_time_stamp(), Some(1_700_000_000));
    assert!(format!("{request:?}").contains("item_count: 2"));
    assert!(!format!("{request:?}").contains("PUBLIC_BATCH_ID_SENTINEL"));
    assert_eq!(request.to_string(), "ClientBatch (2 items)");

    let mut extension = ResponseItemFixture::success(Some(&id));
    extension.extension_criticality = Some(false);
    let response = response_bytes((2, 1), &[extension]);
    let (mut client, _, _) = client_for(
        ExchangeScript::Success {
            response,
            request_write_chunks: Vec::new(),
        },
        None,
    );
    let result = client
        .execute(
            ClientBatch::new(
                ClientBatchItem::new(ClientRequest::discover_versions())
                    .with_unique_batch_item_id(id.clone()),
            ),
            &CodecLimits::defaults(),
        )
        .expect("the fixture is a valid completed response with a non-critical extension");

    assert_eq!(result.len(), 1);
    assert!(!result.is_empty());
    assert!(result.get(1).is_none());
    let mut results = result.iter();
    let item = results.next().expect("one associated response exists");
    assert_eq!(item.unique_batch_item_id(), Some(id.as_slice()));
    assert_eq!(item.extensions().len(), 1);
    assert!(!format!("{item:?}").contains("PUBLIC_BATCH_ID_SENTINEL"));
    assert!(format!("{item:?}").contains("extension_count: 1"));
    assert_eq!(results.len(), 0);
    assert!(!format!("{result:?}").contains("PUBLIC_BATCH_ID_SENTINEL"));
    assert!(format!("{result:?}").contains("item_count: 1"));
    assert_eq!(
        item.extensions()[0].with_ttlv(|view| view.children().len()),
        3
    );
    assert_eq!(
        format!("{:?}", item.extensions()[0]),
        "ClientMessageExtension([REDACTED])"
    );

    let outcome = item.outcome();
    assert!(matches!(
        outcome,
        crate::execute::ClientBatchOutcome::Completed(_)
    ));
    assert_eq!(outcome.asynchronous_correlation_value(), None);
    assert_eq!(outcome.result().status().raw(), 0);
    assert!(format!("{outcome:?}").starts_with("Completed("));
    assert!(outcome.to_string().starts_with("Completed("));
}

#[test]
fn invalid_batch_and_option_inputs_are_rejected_before_exchange() {
    let item = || ClientBatchItem::new(ClientRequest::discover_versions());
    let with_ids = || {
        ClientBatch::from_items([
            item().with_unique_batch_item_id(b"first".to_vec()),
            item().with_unique_batch_item_id(b"second".to_vec()),
        ])
    };
    let invalid_batches = [
        ClientBatch::from_items(std::iter::empty::<ClientBatchItem>()),
        ClientBatch::from_items([item(), item()]),
        ClientBatch::from_items([
            item().with_unique_batch_item_id(b"duplicate".to_vec()),
            item().with_unique_batch_item_id(b"duplicate".to_vec()),
        ]),
        ClientBatch::new(item()).with_asynchronous_indicator(0),
        ClientBatch::new(item()).with_batch_error_continuation_option(2),
        with_ids()
            .with_batch_error_continuation_option(1)
            .with_batch_error_continuation_option(2),
        with_ids().with_batch_error_continuation_option(0),
    ];

    for batch in invalid_batches {
        let (mut client, fake, _) = client_for(
            ExchangeScript::Success {
                response: response_bytes((2, 1), &[ResponseItemFixture::success(None)]),
                request_write_chunks: Vec::new(),
            },
            None,
        );
        let error = client
            .execute(batch, &CodecLimits::defaults())
            .expect_err("invalid batch input is rejected before serialization or exchange");

        assert_eq!(error.category(), ClientErrorCategory::Validation);
        assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
        assert_eq!(
            error
                .cause_category()
                .map(|cause| cause.to_string())
                .as_deref(),
            Some("invalid input")
        );
        assert_eq!(
            error.source().map(ToString::to_string).as_deref(),
            Some("invalid input")
        );
        assert!(error.to_string().contains("client validation failure"));
        assert_eq!(fake.borrow().exchange_count(), 0);
        assert_eq!(fake.borrow().write_call_count(), 0);
    }
}

fn pending_response() -> Vec<u8> {
    response_bytes(
        (2, 1),
        &[ResponseItemFixture::pending(None, CORRELATION_VALUE)],
    )
}

#[test]
fn present_empty_pending_correlation_value_is_preserved() {
    let response = response_bytes((2, 1), &[ResponseItemFixture::pending(None, b"")]);
    let (mut client, fake, _) = client_for(
        ExchangeScript::Success {
            response,
            request_write_chunks: Vec::new(),
        },
        None,
    );

    let result = client
        .execute(discover_versions_batch(Some(2)), &CodecLimits::defaults())
        .expect("a present Byte String correlation value is valid even when empty");
    let outcome = result.get(0).expect("one result").outcome();

    assert_eq!(outcome.asynchronous_correlation_value(), Some(&[][..]));
    assert_eq!(outcome.result().status().raw(), 2);
    assert!(format!("{outcome:?}").starts_with("Pending("));
    assert!(outcome.to_string().starts_with("Pending("));
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn success_after_partial_writes_performs_one_exchange_and_returns_pending() {
    let (mut client, fake, _) = client_for(
        ExchangeScript::Success {
            response: pending_response(),
            request_write_chunks: vec![2, 3],
        },
        None,
    );
    let result = client
        .execute(discover_versions_batch(Some(2)), &CodecLimits::defaults())
        .expect("scripted Discover Versions Pending response is valid");

    assert_eq!(fake.borrow().exchange_count(), 1);
    assert!(fake.borrow().write_call_count() >= 2);
    let outcome = &result.get(0).expect("one result exists").outcome();
    assert!(outcome.asynchronous_correlation_value().is_some());
}

#[test]
fn successful_empty_discover_versions_list_is_preserved() {
    let response = response_bytes(
        (2, 1),
        &[ResponseItemFixture::success(None).with_empty_supported_versions()],
    );
    let (mut client, fake, _) = client_for(
        ExchangeScript::Success {
            response,
            request_write_chunks: Vec::new(),
        },
        None,
    );

    let result = client
        .execute(discover_versions_batch(None), &CodecLimits::defaults())
        .expect("empty Discover Versions result list is valid");

    let outcome = result.get(0).expect("one result").outcome();
    let response_view = outcome.response();
    assert_eq!(response_view.result(), outcome.result());
    assert_eq!(response_view.supported_versions(), Some(&[][..]));
    assert!(response_view.discover_versions().is_some());
    assert!(response_view.create().is_none());
    assert!(response_view.create_key_pair().is_none());
    assert!(response_view.create_split_key().is_none());
    assert!(response_view.add_attribute().is_none());
    assert!(response_view.adjust_attribute().is_none());
    assert!(response_view.delete_attribute().is_none());
    assert!(response_view.modify_attribute().is_none());
    assert!(response_view.set_attribute().is_none());
    assert!(response_view.get_attributes().is_none());
    assert!(response_view.get_attribute_list().is_none());
    assert!(format!("{response_view:?}").starts_with("DiscoverVersions("));

    assert_eq!(
        result
            .get(0)
            .expect("one result")
            .outcome()
            .discover_versions_response()
            .expect("completed Discover Versions response stays typed")
            .supported_versions(),
        Some(&[][..])
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn server_failure_result_is_preserved_as_a_completed_operation_result() {
    let response = response_bytes((2, 1), &[ResponseItemFixture::failure(None)]);
    let (mut client, _, _) = client_for(
        ExchangeScript::Success {
            response,
            request_write_chunks: Vec::new(),
        },
        None,
    );

    let result = client
        .execute(discover_versions_batch(None), &CodecLimits::defaults())
        .expect("a valid KMIP Failure is an operation result");
    let outcome = result.get(0).expect("one result").outcome();

    assert_eq!(
        outcome.operation(),
        crate::ClientOperation::DiscoverVersions
    );
    assert_eq!(outcome.result().status().raw(), 1);
    assert_eq!(outcome.asynchronous_correlation_value(), None);
    let response_view = outcome.response();
    assert_eq!(response_view.result(), outcome.result());
    assert_eq!(response_view.supported_versions(), None);
    assert!(response_view.discover_versions().is_some());
    assert!(outcome.create_response().is_none());
    assert!(outcome.create_key_pair_response().is_none());
    assert!(outcome.create_split_key_response().is_none());
    assert!(format!("{outcome:?}").starts_with("Completed("));
    assert_eq!(
        outcome
            .discover_versions_response()
            .expect("completed Discover Versions response stays typed")
            .supported_versions(),
        None
    );
    assert!(response_view.create().is_none());
    assert!(response_view.create_key_pair().is_none());
    assert!(response_view.create_split_key().is_none());
    assert!(response_view.add_attribute().is_none());
    assert!(response_view.adjust_attribute().is_none());
    assert!(response_view.delete_attribute().is_none());
    assert!(response_view.modify_attribute().is_none());
    assert!(response_view.set_attribute().is_none());
    assert!(response_view.get_attributes().is_none());
    assert!(response_view.get_attribute_list().is_none());
    assert!(format!("{response_view:?}").starts_with("DiscoverVersions("));
}

#[test]
fn reordered_response_batch_is_associated_by_unique_id_and_returned_in_request_order() {
    let request = ClientBatch::from_items([
        ClientBatchItem::new(ClientRequest::discover_versions())
            .with_unique_batch_item_id(b"request-a".to_vec()),
        ClientBatchItem::new(ClientRequest::discover_versions())
            .with_unique_batch_item_id(b"request-b".to_vec()),
    ]);
    let response = response_bytes(
        (2, 1),
        &[
            ResponseItemFixture::success(Some(b"request-b")),
            ResponseItemFixture::success(Some(b"request-a")),
        ],
    );
    let (mut client, _, _) = client_for(
        ExchangeScript::Success {
            response,
            request_write_chunks: Vec::new(),
        },
        None,
    );

    let result = client
        .execute(request, &CodecLimits::defaults())
        .expect("reordered response items are associated by their unique IDs");

    assert_eq!(result.len(), 2);
    assert_eq!(
        result.get(0).and_then(|item| item.unique_batch_item_id()),
        Some(&b"request-a"[..])
    );
    assert_eq!(
        result.get(1).and_then(|item| item.unique_batch_item_id()),
        Some(&b"request-b"[..])
    );
}

#[test]
fn mixed_completed_and_pending_results_preserve_exact_pending_correlation_value() {
    let request = ClientBatch::from_items([
        ClientBatchItem::new(ClientRequest::discover_versions())
            .with_unique_batch_item_id(b"request-a".to_vec()),
        ClientBatchItem::new(ClientRequest::discover_versions())
            .with_unique_batch_item_id(b"request-b".to_vec()),
    ])
    .with_asynchronous_indicator(2);
    let response = response_bytes(
        (2, 1),
        &[
            ResponseItemFixture::pending(Some(b"request-b"), CORRELATION_VALUE),
            ResponseItemFixture::success(Some(b"request-a")),
        ],
    );
    let (mut client, _, _) = client_for(
        ExchangeScript::Success {
            response,
            request_write_chunks: Vec::new(),
        },
        None,
    );

    let result = client
        .execute(request, &CodecLimits::defaults())
        .expect("permitted mixed results retain their request association");

    assert!(
        result
            .get(0)
            .expect("first result")
            .outcome()
            .asynchronous_correlation_value()
            .is_none()
    );
    assert_eq!(
        result
            .get(1)
            .expect("second result")
            .outcome()
            .asynchronous_correlation_value(),
        Some(CORRELATION_VALUE)
    );
}

#[test]
fn noncritical_response_extension_is_preserved_and_critical_extension_is_rejected() {
    let mut noncritical = ResponseItemFixture::success(None);
    noncritical.extension_criticality = Some(false);
    let response = response_bytes((2, 1), &[noncritical]);
    let (mut client, _, _) = client_for(
        ExchangeScript::Success {
            response,
            request_write_chunks: Vec::new(),
        },
        None,
    );
    let result = client
        .execute(discover_versions_batch(None), &CodecLimits::defaults())
        .expect("unknown non-critical extension is preserved");
    assert_eq!(result.get(0).expect("one result").extensions().len(), 1);
    assert_eq!(
        result.get(0).expect("one result").extensions()[0]
            .with_ttlv(|extension| extension.children().len()),
        3
    );

    let mut critical = ResponseItemFixture::success(None);
    critical.extension_criticality = Some(true);
    let response = response_bytes((2, 1), &[critical]);
    let (mut client, _, _) = client_for(
        ExchangeScript::Success {
            response,
            request_write_chunks: Vec::new(),
        },
        None,
    );
    let error = client
        .execute(discover_versions_batch(None), &CodecLimits::defaults())
        .expect_err("unknown critical extension is rejected");
    assert_eq!(error.category(), crate::ClientErrorCategory::Protocol);
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::ResponseStarted)
    );
}

#[test]
fn unsupported_response_version_is_rejected_after_the_response_starts() {
    let response = response_bytes((2, 0), &[ResponseItemFixture::success(None)]);
    let (mut client, _, _) = client_for(
        ExchangeScript::Success {
            response,
            request_write_chunks: Vec::new(),
        },
        None,
    );

    let error = client
        .execute(discover_versions_batch(None), &CodecLimits::defaults())
        .expect_err("response version outside KMIP 2.1 is rejected");

    assert_eq!(error.category(), crate::ClientErrorCategory::Protocol);
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::ResponseStarted)
    );
}

#[test]
fn unoffered_typed_discover_versions_result_is_rejected_with_delivery_evidence() {
    let response = response_bytes(
        (2, 1),
        &[ResponseItemFixture::success(None).with_supported_version((3, 0))],
    );
    let (mut client, _, _) = client_for(
        ExchangeScript::Success {
            response,
            request_write_chunks: Vec::new(),
        },
        None,
    );

    let error = client
        .execute(discover_versions_batch(None), &CodecLimits::defaults())
        .expect_err("the result cannot advertise a version not included in the request");

    assert_eq!(error.category(), ClientErrorCategory::Protocol);
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::ResponseStarted)
    );
    assert!(error.to_string().contains("invalid protocol value"));
}

#[test]
fn failure_before_write_reports_not_sent_and_does_not_retry() {
    let (mut client, fake, _) = client_for(ExchangeScript::FailBeforeWrite, None);
    let error = client
        .execute(discover_versions_batch(None), &CodecLimits::defaults())
        .expect_err("scripted transport fails before writing");

    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(fake.borrow().exchange_count(), 1);
    assert_eq!(fake.borrow().write_call_count(), 0);
}

#[test]
fn failure_after_partial_write_reports_possibly_sent_and_does_not_retry() {
    let (mut client, fake, _) = client_for(
        ExchangeScript::FailAfterPartialWrite { written_bytes: 2 },
        None,
    );
    let error = client
        .execute(discover_versions_batch(None), &CodecLimits::defaults())
        .expect_err("scripted transport fails after a partial write");

    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::PossiblySent)
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
    assert_eq!(fake.borrow().write_call_count(), 1);
    assert!(fake.borrow().written_byte_count() > 0);
}

#[test]
fn failure_after_response_started_preserves_state_and_redacts_error_chain() {
    let partial_response = CORRELATION_VALUE.to_vec();
    let (mut client, fake, _) = client_for(
        ExchangeScript::FailAfterPartialRead {
            written_bytes: 2,
            response_bytes: partial_response,
        },
        None,
    );
    let error = client
        .execute(discover_versions_batch(None), &CodecLimits::defaults())
        .expect_err("scripted transport fails after response reception starts");
    let mut source = error.source();
    let mut source_chain_redacts = true;
    while let Some(error_source) = source {
        source_chain_redacts &= !format!("{error_source:?}").contains(CORRELATION_PREFIX)
            && !error_source.to_string().contains(CORRELATION_PREFIX);
        source = error_source.source();
    }

    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::ResponseStarted)
    );
    assert!(!format!("{error:?}").contains(CORRELATION_PREFIX));
    assert!(!error.to_string().contains(CORRELATION_PREFIX));
    assert!(source_chain_redacts);
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn pending_correlation_is_exact_borrowed_and_redacted_from_formatted_output() {
    let (mut client, fake, _) = client_for(
        ExchangeScript::Success {
            response: pending_response(),
            request_write_chunks: vec![2, 3],
        },
        None,
    );
    let result = client
        .execute(discover_versions_batch(Some(2)), &CodecLimits::defaults())
        .expect("scripted Pending response is valid");
    let outcome = result.get(0).expect("one result exists").outcome();

    assert_eq!(
        outcome.asynchronous_correlation_value(),
        Some(CORRELATION_VALUE)
    );
    assert!(!format!("{outcome:?}").contains(CORRELATION_PREFIX));
    assert!(!outcome.to_string().contains(CORRELATION_PREFIX));
    assert_eq!(fake.borrow().captured_logs(), &[] as &[String]);
}

#[test]
fn request_owner_is_live_through_exchange_and_zeroized_before_release() {
    let request_observer = ZeroizationObserver::new(None);
    let (mut client, fake, owner_live) = client_for(
        ExchangeScript::Success {
            response: response_bytes((2, 1), &[ResponseItemFixture::success(None)]),
            request_write_chunks: vec![2, 3],
        },
        Some(request_observer.clone()),
    );

    client
        .execute(discover_versions_batch(None), &CodecLimits::defaults())
        .expect("scripted Discover Versions success is valid");

    assert!(owner_live.get());
    assert_eq!(request_observer.result(), Some(true));
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn pending_correlation_owner_zeroizes_before_release() {
    let request_observer = ZeroizationObserver::new(None);
    let pending_observer = ZeroizationObserver::new(None);
    let fake = Rc::new(RefCell::new(ScriptedTransport::new(
        ExchangeScript::Success {
            response: pending_response(),
            request_write_chunks: vec![2, 3],
        },
    )));
    let owner_live = Rc::new(Cell::new(false));
    let transport = SharedScriptedTransport {
        fake: Rc::clone(&fake),
        request_owner_observer: Some(request_observer.clone()),
        owner_live_when_exchange_returns: Rc::clone(&owner_live),
    };
    let mut client = Client::for_test_with_lifecycle_observers(
        transport,
        request_observer,
        pending_observer.clone(),
    );
    let result = client
        .execute(discover_versions_batch(Some(2)), &CodecLimits::defaults())
        .expect("scripted Pending response is valid");
    assert!(owner_live.get());
    assert_eq!(fake.borrow().exchange_count(), 1);

    drop(result);

    assert_eq!(pending_observer.result(), Some(true));
}
