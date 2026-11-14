//! Explicit profile selection must fail closed before an exchange until its
//! complete typed rule registry is available. Source: KMIP Specification v2.1
//! §14.1; KMIP Profiles v2.1 §6.1. Traceability: KMIPKIT-0010 FR-014, SC-009,
//! and SC-014.

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::error::Error;
use std::rc::Rc;

use kmipkit_protocol::{
    AddAttributeRequest, CheckRequest, CurrentAttribute, DeleteAttributeRequest,
    GetAttributesRequest, ModifyAttributeRequest, NewAttribute, ProfileObjectState,
    ProfileTransport, ProfileValidationContext, ProfileValidationError, ProfileValidationErrorCode,
    ProfileValidationSelection, QueryFunction, QueryRequest, RequestMessage, ResponseMessage,
    SetAttributeRequest, SetDefaultsRequest, UniqueIdentifier,
};
use kmipkit_transport::{
    RequestDeliveryState, RequestOptions, Transport, TransportError, TransportResponse,
};
use kmipkit_ttlv::codec::{CodecLimits, decode_with_limits};
use kmipkit_ttlv::{Structure, Value, ValueView};

use crate::execute::{ProfileExchangeValidator, encode_message_for_test};
use crate::execute_test_support::{
    ResponseItemFixture, one_item_response_bytes, response_bytes, test_item, test_structure,
};
use crate::{
    Client, ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientErrorCategory, ClientRequest,
};

const BASELINE: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
const SERVER_MESSAGE_SECRET_MARKER: &str = "profile-response-private-marker";

struct CountingTransport {
    exchanges: Rc<Cell<usize>>,
    requests: Rc<RefCell<Vec<Vec<u8>>>>,
    response: Vec<u8>,
    scripted_responses: VecDeque<Vec<u8>>,
}

impl CountingTransport {
    fn new(exchanges: Rc<Cell<usize>>) -> Self {
        Self {
            exchanges,
            requests: Rc::new(RefCell::new(Vec::new())),
            response: response_bytes((2, 1), &[ResponseItemFixture::success(None)]),
            scripted_responses: VecDeque::new(),
        }
    }

    fn with_responses(
        exchanges: Rc<Cell<usize>>,
        responses: impl IntoIterator<Item = Vec<u8>>,
    ) -> Self {
        Self {
            exchanges,
            requests: Rc::new(RefCell::new(Vec::new())),
            response: Vec::new(),
            scripted_responses: responses.into_iter().collect(),
        }
    }
}

impl Transport for CountingTransport {
    fn exchange(
        &mut self,
        request: &[u8],
        _max_response_bytes: usize,
    ) -> Result<TransportResponse, TransportError> {
        self.exchanges.set(self.exchanges.get() + 1);
        self.requests.borrow_mut().push(request.to_vec());
        let response = self
            .scripted_responses
            .pop_front()
            .unwrap_or_else(|| self.response.clone());
        Ok(TransportResponse::new(response))
    }
}

fn discover_versions_batch() -> ClientBatch {
    ClientBatch::new(ClientBatchItem::new(ClientRequest::discover_versions()))
}

#[test]
fn empty_profile_selection_is_rejected_before_exchange_on_both_entry_points() {
    let exchanges = Rc::new(Cell::new(0));
    let mut client = Client::for_test(CountingTransport::new(Rc::clone(&exchanges)));
    let selection = ProfileValidationSelection::new(std::iter::empty::<&str>());
    let limits = CodecLimits::defaults();
    let options = RequestOptions::default();

    let options_error = client
        .execute_with_profiles_and_options(discover_versions_batch(), &selection, &limits, &options)
        .expect_err("an empty explicit selection fails before transport I/O");
    assert_eq!(options_error.category(), ClientErrorCategory::Validation);
    assert_eq!(
        options_error.delivery_state(),
        Some(RequestDeliveryState::NotSent)
    );
    assert_eq!(
        options_error
            .profile_validation_error()
            .expect("the typed profile failure is preserved")
            .code(),
        ProfileValidationErrorCode::EmptySelection
    );

    let context_error = client
        .execute_with_profiles_and_context_and_options(
            discover_versions_batch(),
            &selection,
            &ProfileValidationContext::new(),
            &limits,
            &options,
        )
        .expect_err("context-aware execution rejects an empty selection before I/O");
    assert_eq!(context_error.category(), ClientErrorCategory::Validation);
    assert_eq!(
        context_error.delivery_state(),
        Some(RequestDeliveryState::NotSent)
    );
    assert_eq!(
        context_error
            .profile_validation_error()
            .expect("the typed profile failure is preserved")
            .code(),
        ProfileValidationErrorCode::EmptySelection
    );
    assert_eq!(exchanges.get(), 0);
}

#[test]
fn baseline_selection_with_incomplete_evidence_fails_before_exchange() {
    let exchanges = Rc::new(Cell::new(0));
    let mut client = Client::for_test(CountingTransport::new(Rc::clone(&exchanges)));
    let error = client
        .execute_with_profiles(
            discover_versions_batch(),
            &ProfileValidationSelection::new([BASELINE]),
            &CodecLimits::defaults(),
        )
        .expect_err("Baseline cannot be selected while evidence remains incomplete");

    assert_eq!(error.category(), ClientErrorCategory::Validation);
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(exchanges.get(), 0);
    let profile_error = error
        .profile_validation_error()
        .expect("the closed selection preserves its structured profile error");
    assert_eq!(
        profile_error.code(),
        ProfileValidationErrorCode::ProfileUnavailable
    );
    assert_eq!(profile_error.profile_id(), Some(BASELINE));
}

fn query_batch() -> ClientBatch {
    ClientBatch::new(ClientBatchItem::new(ClientRequest::query(
        QueryRequest::new([QueryFunction::PROFILES, QueryFunction::DEFAULTS_INFORMATION]),
    )))
}

fn query_response_advertising_baseline_and_defaults() -> Vec<u8> {
    const QUERY: u32 = 0x0000_0018;
    const PROFILE_INFORMATION: u32 = 0x0042_00eb;
    const PROFILE_NAME: u32 = 0x0042_00ec;
    const DEFAULTS_INFORMATION: u32 = 0x0042_0152;
    const OBJECT_DEFAULTS: u32 = 0x0042_0153;
    const PROTECTION_STORAGE_MASKS: u32 = 0x0042_015f;

    let profile_information = test_item(
        PROFILE_INFORMATION,
        Value::structure(test_structure([test_item(
            PROFILE_NAME,
            Value::enumeration(0x0000_012a),
        )])),
    );
    one_item_response_bytes(
        QUERY,
        0,
        None,
        None,
        None,
        None,
        Some(test_structure([
            profile_information,
            test_item(
                DEFAULTS_INFORMATION,
                Value::structure(test_structure([test_item(
                    OBJECT_DEFAULTS,
                    Value::structure(test_structure([])),
                )])),
            ),
            test_item(
                PROTECTION_STORAGE_MASKS,
                Value::structure(test_structure([])),
            ),
        ])),
    )
}

fn discover_versions_response_with_message(message: &str) -> Vec<u8> {
    const DISCOVER_VERSIONS: u32 = 0x0000_001e;

    one_item_response_bytes(
        DISCOVER_VERSIONS,
        1,
        Some(1),
        Some(message),
        None,
        None,
        None,
    )
}

fn request_defaults_information_child_count(encoded: &[u8]) -> Option<usize> {
    const BATCH_ITEM: u32 = 0x0042_000f;
    const REQUEST_PAYLOAD: u32 = 0x0042_0079;
    const DEFAULTS_INFORMATION: u32 = 0x0042_0152;
    let decoded = decode_with_limits(encoded, &CodecLimits::defaults()).ok()?;
    decoded.with_value(|value| {
        let ValueView::Structure(message) = value else {
            return None;
        };
        let batch_item = message
            .children()
            .iter()
            .find(|item| item.tag().raw() == BATCH_ITEM)?;
        batch_item.with_value(|value| {
            let ValueView::Structure(batch_item) = value else {
                return None;
            };
            let payload = batch_item
                .children()
                .iter()
                .find(|item| item.tag().raw() == REQUEST_PAYLOAD)?;
            payload.with_value(|value| {
                let ValueView::Structure(payload) = value else {
                    return None;
                };
                let defaults_information = payload
                    .children()
                    .iter()
                    .find(|item| item.tag().raw() == DEFAULTS_INFORMATION)?;
                defaults_information.with_value(|value| {
                    let ValueView::Structure(defaults_information) = value else {
                        return None;
                    };
                    Some(defaults_information.children().len())
                })
            })
        })
    })
}

#[test]
fn explicit_query_and_discover_responses_do_not_select_profiles_or_change_defaults() {
    const SET_DEFAULTS: u32 = 0x0000_0036;

    let set_defaults_response = one_item_response_bytes(
        SET_DEFAULTS,
        0,
        None,
        None,
        None,
        None,
        Some(test_structure([])),
    );
    let exchanges = Rc::new(Cell::new(0));
    let transport = CountingTransport::with_responses(
        Rc::clone(&exchanges),
        [
            query_response_advertising_baseline_and_defaults(),
            set_defaults_response,
            response_bytes((2, 1), &[ResponseItemFixture::success(None)]),
        ],
    );
    let requests = Rc::clone(&transport.requests);
    let mut client = Client::for_test(transport);
    let limits = CodecLimits::defaults();

    let query_result = client
        .execute(query_batch(), &limits)
        .expect("an explicit Query response may report profiles and defaults");
    let item = query_result
        .get(0)
        .expect("the explicit Query response contains one batch item");
    let ClientBatchOutcome::Query(query) = item.outcome() else {
        panic!("the explicit Query response remains typed");
    };
    assert_eq!(query.profile_information().count(), 1);
    assert!(query.defaults_information().is_some());
    assert_eq!(exchanges.get(), 1);

    let explicit_empty_defaults =
        ClientBatch::new(ClientBatchItem::new(ClientRequest::set_defaults(
            SetDefaultsRequest::new().with_defaults_information(test_structure([])),
        )));
    client
        .execute(explicit_empty_defaults, &limits)
        .expect("the caller's explicit empty defaults structure is sent unchanged");
    assert_eq!(exchanges.get(), 2);
    assert_eq!(
        request_defaults_information_child_count(&requests.borrow()[1]),
        Some(0)
    );

    client
        .execute(discover_versions_batch(), &limits)
        .expect("Discover Versions remains an explicit operation");
    assert_eq!(exchanges.get(), 3);
}

#[test]
fn synthetic_missing_rule_mapping_preflight_error_fails_before_transport_with_stable_source_ids() {
    let exchanges = Rc::new(Cell::new(0));
    let request_checks = Rc::new(Cell::new(0));
    let expected_profile_error = ProfileValidationError::for_test(
        ProfileValidationErrorCode::MissingRuleMapping,
        "KMIPKIT-TEST-PROFILE",
        "KMIPKIT-TEST-REQUIREMENT",
        "KMIPKIT-TEST-CLAUSE",
    );
    let validator = TestProfileValidator {
        error: expected_profile_error.clone(),
        request_checks: Some(Rc::clone(&request_checks)),
        transport_error: None,
        request_error: Some(expected_profile_error.clone()),
    };
    let mut client = Client::for_test(CountingTransport::new(Rc::clone(&exchanges)));
    let error = client
        .execute_inner(
            discover_versions_batch(),
            &CodecLimits::defaults(),
            &kmipkit_transport::RequestOptions::default(),
            Some(&validator),
        )
        .expect_err("a synthetic profile preflight error prevents the exchange");

    assert_eq!(error.category(), ClientErrorCategory::Validation);
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(request_checks.get(), 1);
    assert_eq!(exchanges.get(), 0);
    let profile_error = error
        .profile_validation_error()
        .expect("profile validation keeps its stable structured source error");
    assert_eq!(profile_error, &expected_profile_error);
    assert_eq!(
        profile_error.code(),
        ProfileValidationErrorCode::MissingRuleMapping
    );
    assert_eq!(profile_error.profile_id(), Some("KMIPKIT-TEST-PROFILE"));
    assert_eq!(
        profile_error.requirement_id(),
        Some("KMIPKIT-TEST-REQUIREMENT")
    );
    assert_eq!(
        profile_error.source_clause_id(),
        Some("KMIPKIT-TEST-CLAUSE")
    );
}

#[test]
fn unprofiled_base_operation_still_reaches_transport() {
    let exchanges = Rc::new(Cell::new(0));
    let mut client = Client::for_test(CountingTransport::new(Rc::clone(&exchanges)));

    client
        .execute(discover_versions_batch(), &CodecLimits::defaults())
        .expect("the existing unprofiled API remains usable");

    assert_eq!(exchanges.get(), 1);
}

struct TestProfileValidator {
    error: ProfileValidationError,
    request_checks: Option<Rc<Cell<usize>>>,
    transport_error: Option<ProfileValidationError>,
    request_error: Option<ProfileValidationError>,
}

impl ProfileExchangeValidator for TestProfileValidator {
    fn validate_transport(
        &self,
        _transport: ProfileTransport,
    ) -> Result<(), ProfileValidationError> {
        self.transport_error.clone().map_or(Ok(()), Err)
    }

    fn validate_request(&self, _request: &RequestMessage) -> Result<(), ProfileValidationError> {
        if let Some(request_checks) = &self.request_checks {
            request_checks.set(request_checks.get() + 1);
        }
        self.request_error.clone().map_or(Ok(()), Err)
    }

    fn validate_response(
        &self,
        request: &RequestMessage,
        response: &ResponseMessage,
    ) -> Result<(), ProfileValidationError> {
        let request_operation = request
            .batch_items()
            .next()
            .and_then(kmipkit_protocol::RequestBatchItemView::operation);
        let response_operation = response
            .batch_items()
            .next()
            .and_then(kmipkit_protocol::ResponseBatchItemView::operation);
        let has_matching_single_item = request.batch_items().len() == 1
            && response.batch_items().len() == 1
            && request_operation.is_some()
            && request_operation == response_operation;

        if has_matching_single_item {
            Err(self.error.clone())
        } else {
            Ok(())
        }
    }
}

struct AllowingProfileValidator {
    request_checks: Rc<Cell<usize>>,
    response_checks: Rc<Cell<usize>>,
}

struct BatchRejectingProfileValidator {
    error: ProfileValidationError,
}

impl ProfileExchangeValidator for BatchRejectingProfileValidator {
    fn validate_transport(
        &self,
        _transport: ProfileTransport,
    ) -> Result<(), ProfileValidationError> {
        Ok(())
    }

    fn validate_request(&self, request: &RequestMessage) -> Result<(), ProfileValidationError> {
        assert_eq!(request.batch_items().len(), 2);
        Ok(())
    }

    fn validate_response(
        &self,
        request: &RequestMessage,
        response: &ResponseMessage,
    ) -> Result<(), ProfileValidationError> {
        assert_eq!(request.batch_items().len(), 2);
        assert_eq!(response.batch_items().len(), 2);
        Err(self.error.clone())
    }
}

impl ProfileExchangeValidator for AllowingProfileValidator {
    fn validate_transport(
        &self,
        transport: ProfileTransport,
    ) -> Result<(), ProfileValidationError> {
        assert_eq!(transport, ProfileTransport::RawTtlvTls);
        Ok(())
    }

    fn validate_request(&self, request: &RequestMessage) -> Result<(), ProfileValidationError> {
        assert_eq!(
            request
                .batch_items()
                .next()
                .and_then(kmipkit_protocol::RequestBatchItemView::operation),
            Some(0x0000_001e)
        );
        self.request_checks.set(self.request_checks.get() + 1);
        Ok(())
    }

    fn validate_response(
        &self,
        request: &RequestMessage,
        response: &ResponseMessage,
    ) -> Result<(), ProfileValidationError> {
        assert_eq!(
            request
                .batch_items()
                .next()
                .and_then(kmipkit_protocol::RequestBatchItemView::operation),
            response
                .batch_items()
                .next()
                .and_then(kmipkit_protocol::ResponseBatchItemView::operation)
        );
        self.response_checks.set(self.response_checks.get() + 1);
        Ok(())
    }
}

#[test]
fn profiled_exchange_allows_extra_base_protocol_operation_when_rules_allow_it() {
    let exchanges = Rc::new(Cell::new(0));
    let request_checks = Rc::new(Cell::new(0));
    let response_checks = Rc::new(Cell::new(0));
    let validator = AllowingProfileValidator {
        request_checks: Rc::clone(&request_checks),
        response_checks: Rc::clone(&response_checks),
    };
    let mut client = Client::for_test(CountingTransport::new(Rc::clone(&exchanges)));

    client
        .execute_inner(
            discover_versions_batch(),
            &CodecLimits::defaults(),
            &kmipkit_transport::RequestOptions::default(),
            Some(&validator),
        )
        .expect("an otherwise valid base-protocol operation passes an allowing profile rule set");

    assert_eq!(request_checks.get(), 1);
    assert_eq!(response_checks.get(), 1);
    assert_eq!(exchanges.get(), 1);
}

#[test]
fn response_profile_rejection_after_exchange_preserves_response_started_delivery() {
    let validator = TestProfileValidator {
        error: ProfileValidationError::for_test(
            ProfileValidationErrorCode::ResponseRuleRejected,
            "KMIPKIT-TEST-PROFILE",
            "KMIPKIT-TEST-REQUIREMENT",
            "KMIPKIT-TEST-CLAUSE",
        ),
        request_checks: None,
        transport_error: None,
        request_error: None,
    };
    let exchanges = Rc::new(Cell::new(0));
    let transport = CountingTransport::with_responses(
        Rc::clone(&exchanges),
        [discover_versions_response_with_message(
            SERVER_MESSAGE_SECRET_MARKER,
        )],
    );
    let mut client = Client::for_test(transport);

    let error = client
        .execute_inner(
            discover_versions_batch(),
            &CodecLimits::defaults(),
            &kmipkit_transport::RequestOptions::default(),
            Some(&validator),
        )
        .expect_err("profile response validation rejects the received response");

    assert_eq!(exchanges.get(), 1);
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::ResponseStarted)
    );
    assert_eq!(
        error.cause_category(),
        Some(crate::ClientCauseCategory::Other)
    );
    assert!(error.to_string().contains("ResponseStarted"));
    assert!(!format!("{error} {error:?}").contains(SERVER_MESSAGE_SECRET_MARKER));
    assert!(Error::source(&error).is_some());
    let profile_error = error
        .profile_validation_error()
        .expect("response rejection remains a structured profile error");
    assert_eq!(
        profile_error.code(),
        ProfileValidationErrorCode::ResponseRuleRejected
    );
    assert_eq!(profile_error.profile_id(), Some("KMIPKIT-TEST-PROFILE"));
    assert_eq!(
        profile_error.requirement_id(),
        Some("KMIPKIT-TEST-REQUIREMENT")
    );
    assert_eq!(
        profile_error.source_clause_id(),
        Some("KMIPKIT-TEST-CLAUSE")
    );
}

#[test]
fn batch_response_profile_rejection_preserves_response_started_delivery() {
    let validator = BatchRejectingProfileValidator {
        error: ProfileValidationError::for_test(
            ProfileValidationErrorCode::ResponseRuleRejected,
            "KMIPKIT-TEST-PROFILE",
            "KMIPKIT-TEST-REQUIREMENT",
            "KMIPKIT-TEST-CLAUSE",
        ),
    };
    let exchanges = Rc::new(Cell::new(0));
    let transport = CountingTransport::with_responses(
        Rc::clone(&exchanges),
        [response_bytes(
            (2, 1),
            &[
                ResponseItemFixture::success(Some(&[1])),
                ResponseItemFixture::success(Some(&[2])),
            ],
        )],
    );
    let mut client = Client::for_test(transport);
    let batch = ClientBatch::from_items([
        ClientBatchItem::new(ClientRequest::discover_versions()).with_unique_batch_item_id(vec![1]),
        ClientBatchItem::new(ClientRequest::discover_versions()).with_unique_batch_item_id(vec![2]),
    ]);

    let error = client
        .execute_inner(
            batch,
            &CodecLimits::defaults(),
            &kmipkit_transport::RequestOptions::default(),
            Some(&validator),
        )
        .expect_err("a profile response rule rejects the received multi-item batch");

    assert_eq!(exchanges.get(), 1, "{error:?}");
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::ResponseStarted)
    );
    assert_eq!(
        error
            .profile_validation_error()
            .map(ProfileValidationError::code),
        Some(ProfileValidationErrorCode::ResponseRuleRejected)
    );
    let profile_error = error
        .profile_validation_error()
        .expect("batch response rejection remains a structured profile error");
    assert_eq!(profile_error.profile_id(), Some("KMIPKIT-TEST-PROFILE"));
    assert_eq!(
        profile_error.requirement_id(),
        Some("KMIPKIT-TEST-REQUIREMENT")
    );
    assert_eq!(
        profile_error.source_clause_id(),
        Some("KMIPKIT-TEST-CLAUSE")
    );
}

#[test]
fn request_limits_are_checked_before_profile_evaluation() {
    let exchanges = Rc::new(Cell::new(0));
    let request_checks = Rc::new(Cell::new(0));
    let validator = TestProfileValidator {
        error: ProfileValidationError::for_test(
            ProfileValidationErrorCode::ResponseRuleRejected,
            "KMIPKIT-TEST-PROFILE",
            "KMIPKIT-TEST-REQUIREMENT",
            "KMIPKIT-TEST-CLAUSE",
        ),
        request_checks: Some(Rc::clone(&request_checks)),
        transport_error: None,
        request_error: None,
    };
    let mut client = Client::for_test(CountingTransport::new(Rc::clone(&exchanges)));
    let limits = CodecLimits::new(0, CodecLimits::DEFAULT_MAX_STRUCTURE_DEPTH, 100_000)
        .expect("zero message size is a valid configured limit");

    let error = client
        .execute_inner(
            discover_versions_batch(),
            &limits,
            &kmipkit_transport::RequestOptions::default(),
            Some(&validator),
        )
        .expect_err("an oversized request is rejected before profile traversal");

    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(request_checks.get(), 0);
    assert_eq!(exchanges.get(), 0);
}

#[test]
fn transport_profile_rejection_is_not_sent_and_skips_the_exchange() {
    let exchanges = Rc::new(Cell::new(0));
    let mut client = Client::for_test(CountingTransport::new(Rc::clone(&exchanges)));
    let validator = TestProfileValidator {
        error: ProfileValidationError::for_test(
            ProfileValidationErrorCode::TransportConstraintRejected,
            "KMIPKIT-TEST-PROFILE",
            "KMIPKIT-TEST-REQUIREMENT",
            "KMIPKIT-TEST-CLAUSE",
        ),
        request_checks: None,
        transport_error: Some(ProfileValidationError::for_test(
            ProfileValidationErrorCode::TransportConstraintRejected,
            "KMIPKIT-TEST-PROFILE",
            "KMIPKIT-TEST-REQUIREMENT",
            "KMIPKIT-TEST-CLAUSE",
        )),
        request_error: None,
    };

    let error = client
        .execute_inner(
            discover_versions_batch(),
            &CodecLimits::defaults(),
            &kmipkit_transport::RequestOptions::default(),
            Some(&validator),
        )
        .expect_err("a profile transport restriction fails before request preparation");

    assert_eq!(exchanges.get(), 0);
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(
        error
            .profile_validation_error()
            .map(ProfileValidationError::code),
        Some(ProfileValidationErrorCode::TransportConstraintRejected)
    );
}

#[test]
fn request_profile_rejection_is_not_sent_and_skips_the_exchange() {
    let exchanges = Rc::new(Cell::new(0));
    let request_checks = Rc::new(Cell::new(0));
    let error = ProfileValidationError::for_test(
        ProfileValidationErrorCode::RequestRuleRejected,
        "KMIPKIT-TEST-PROFILE",
        "KMIPKIT-TEST-REQUIREMENT",
        "KMIPKIT-TEST-CLAUSE",
    );
    let validator = TestProfileValidator {
        error: error.clone(),
        request_checks: Some(Rc::clone(&request_checks)),
        transport_error: None,
        request_error: Some(error),
    };
    let mut client = Client::for_test(CountingTransport::new(Rc::clone(&exchanges)));

    let error = client
        .execute_inner(
            discover_versions_batch(),
            &CodecLimits::defaults(),
            &kmipkit_transport::RequestOptions::default(),
            Some(&validator),
        )
        .expect_err("a profile request rule fails before the transport exchange");

    assert_eq!(request_checks.get(), 1);
    assert_eq!(exchanges.get(), 0);
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(
        error
            .profile_validation_error()
            .map(ProfileValidationError::code),
        Some(ProfileValidationErrorCode::RequestRuleRejected)
    );
}

#[test]
fn singleton_check_profile_rule_rejection_is_not_sent_and_skips_the_exchange() {
    const CHECK_REQUIREMENT: &str = "KMIPKIT-REQ-SPEC-6.1.7-001";
    const CHECK_CLAUSE: &str = "KMIPKIT-CLAUSE-SPEC-6.1.7-001";

    let exchanges = Rc::new(Cell::new(0));
    let request_checks = Rc::new(Cell::new(0));
    let error = ProfileValidationError::for_test(
        ProfileValidationErrorCode::RequestRuleRejected,
        BASELINE,
        CHECK_REQUIREMENT,
        CHECK_CLAUSE,
    );
    let validator = TestProfileValidator {
        error: error.clone(),
        request_checks: Some(Rc::clone(&request_checks)),
        transport_error: None,
        request_error: Some(error),
    };
    let mut client = Client::for_test(CountingTransport::new(Rc::clone(&exchanges)));
    let singleton_check = ClientBatch::new(ClientBatchItem::new(ClientRequest::check(
        CheckRequest::new().with_unique_identifier(UniqueIdentifier::TextString(
            "profile-check-singleton".to_owned(),
        )),
    )));

    let error = client
        .execute_inner(
            singleton_check,
            &CodecLimits::defaults(),
            &kmipkit_transport::RequestOptions::default(),
            Some(&validator),
        )
        .expect_err("a singleton Check rejected by profile preflight is not exchanged");

    assert_eq!(request_checks.get(), 1);
    assert_eq!(exchanges.get(), 0);
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    let profile_error = error
        .profile_validation_error()
        .expect("the profile rejection preserves its stable source identifiers");
    assert_eq!(
        profile_error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(profile_error.profile_id(), Some(BASELINE));
    assert_eq!(profile_error.requirement_id(), Some(CHECK_REQUIREMENT));
    assert_eq!(profile_error.source_clause_id(), Some(CHECK_CLAUSE));
}

#[test]
fn server_result_has_no_local_profile_validation_error() {
    let result = kmipkit_protocol::KmipOperationResult::new(
        kmipkit_protocol::ResultStatus::from_raw(1),
        Some(kmipkit_protocol::ResultReason::from_raw(1)),
        None,
    )
    .expect("a Failure result includes a reason");
    let error = crate::ClientError::server_result(result);

    assert_eq!(error.profile_validation_error(), None);
    assert_eq!(error.cause_category(), None);
    assert_eq!(error.delivery_state(), None);
    assert!(Error::source(&error).is_none());
}

#[test]
fn original_creation_date_mutation_with_present_bound_context_is_rejected_before_transport() {
    const REQUIREMENT: &str = "KMIPKIT-REQ-SPEC-4.38-003";
    const CLAUSE: &str = "KMIPKIT-CLAUSE-SPEC-4.38-003";
    const ORIGINAL_CREATION_DATE: u32 = 0x0042_00BC;
    const OBJECT_ID: &str = "profile-context-object-sensitive-id";

    let selection = ProfileValidationSelection::for_test([BASELINE]);
    let date = || test_item(ORIGINAL_CREATION_DATE, Value::date_time(1_700_000_001));
    let requests = [
        ClientRequest::add_attribute(AddAttributeRequest::new(
            Some(OBJECT_ID.to_owned()),
            NewAttribute::new(date()),
        )),
        ClientRequest::modify_attribute(ModifyAttributeRequest::new(
            Some(OBJECT_ID.to_owned()),
            None,
            NewAttribute::new(date()),
        )),
        ClientRequest::delete_attribute(DeleteAttributeRequest::new(
            Some(OBJECT_ID.to_owned()),
            Some(CurrentAttribute::new(date())),
            None,
        )),
        ClientRequest::set_attribute(SetAttributeRequest::new(
            Some(OBJECT_ID.to_owned()),
            NewAttribute::new(date()),
        )),
    ];

    for request in requests {
        let exchanges = Rc::new(Cell::new(0));
        let mut client = Client::for_test(CountingTransport::new(Rc::clone(&exchanges)));
        let batch = ClientBatch::new(ClientBatchItem::new(request));
        let context = ProfileValidationContext::new().with_object_state(
            0,
            ProfileObjectState::new(OBJECT_ID).with_original_creation_date_present(Some(true)),
        );

        let error = client
            .execute_with_profiles_and_context(
                batch,
                &selection,
                &context,
                &CodecLimits::defaults(),
            )
            .expect_err("an Original Creation Date Add/update/removal after set is prohibited");

        assert_eq!(exchanges.get(), 0);
        assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
        let profile_error = error
            .profile_validation_error()
            .expect("the prohibited request carries a typed profile error");
        assert_eq!(
            profile_error.code(),
            ProfileValidationErrorCode::RequestRuleRejected
        );
        assert_eq!(profile_error.requirement_id(), Some(REQUIREMENT));
        assert_eq!(profile_error.source_clause_id(), Some(CLAUSE));
        assert_eq!(profile_error.batch_item_index(), Some(0));
        assert!(!format!("{profile_error}").contains(OBJECT_ID));
    }
}

#[test]
fn original_creation_date_non_add_mutation_with_known_absent_state_is_rejected() {
    const REQUIREMENT: &str = "KMIPKIT-REQ-SPEC-4.38-003";
    const CLAUSE: &str = "KMIPKIT-CLAUSE-SPEC-4.38-003";
    const ORIGINAL_CREATION_DATE: u32 = 0x0042_00BC;
    const OBJECT_ID: &str = "profile-context-known-absent-object-id";

    let selection = ProfileValidationSelection::for_test([BASELINE]);
    let date = || test_item(ORIGINAL_CREATION_DATE, Value::date_time(1_700_000_001));
    let requests = [
        ClientRequest::modify_attribute(ModifyAttributeRequest::new(
            Some(OBJECT_ID.to_owned()),
            None,
            NewAttribute::new(date()),
        )),
        ClientRequest::delete_attribute(DeleteAttributeRequest::new(
            Some(OBJECT_ID.to_owned()),
            Some(CurrentAttribute::new(date())),
            None,
        )),
        ClientRequest::set_attribute(SetAttributeRequest::new(
            Some(OBJECT_ID.to_owned()),
            NewAttribute::new(date()),
        )),
    ];

    for request in requests {
        let exchanges = Rc::new(Cell::new(0));
        let mut client = Client::for_test(CountingTransport::new(Rc::clone(&exchanges)));
        let batch = ClientBatch::new(ClientBatchItem::new(request));
        let context = ProfileValidationContext::new().with_object_state(
            0,
            ProfileObjectState::new(OBJECT_ID).with_original_creation_date_present(Some(false)),
        );

        let error = client
            .execute_with_profiles_and_context(
                batch,
                &selection,
                &context,
                &CodecLimits::defaults(),
            )
            .expect_err("the client cannot modify or delete Original Creation Date");

        assert_eq!(exchanges.get(), 0);
        assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
        let profile_error = error
            .profile_validation_error()
            .expect("the prohibited request carries a typed profile error");
        assert_eq!(
            profile_error.code(),
            ProfileValidationErrorCode::RequestRuleRejected
        );
        assert_eq!(profile_error.requirement_id(), Some(REQUIREMENT));
        assert_eq!(profile_error.source_clause_id(), Some(CLAUSE));
        assert_eq!(profile_error.batch_item_index(), Some(0));
    }
}

#[test]
fn usage_limits_total_change_after_protection_use_is_rejected_with_bound_context() {
    const REQUIREMENT: &str = "KMIPKIT-REQ-SPEC-4.59-002";
    const CLAUSE: &str = "KMIPKIT-CLAUSE-SPEC-4.59-002";
    const USAGE_LIMITS: u32 = 0x0042_0095;
    const TOTAL: u32 = 0x0042_0097;
    const COUNT: u32 = 0x0042_0096;
    const UNIT: u32 = 0x0042_0098;
    const OBJECT_ID: &str = "usage-limits-context-sensitive-id";

    let exchanges = Rc::new(Cell::new(0));
    let mut client = Client::for_test(CountingTransport::new(Rc::clone(&exchanges)));
    let usage_limits = test_item(
        USAGE_LIMITS,
        Value::structure(test_structure([
            test_item(TOTAL, Value::long_integer(40)),
            test_item(COUNT, Value::long_integer(50)),
            test_item(UNIT, Value::enumeration(1)),
        ])),
    );
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::set_attribute(
        SetAttributeRequest::new(Some(OBJECT_ID.to_owned()), NewAttribute::new(usage_limits)),
    )));
    let selection = ProfileValidationSelection::for_test([BASELINE]);
    let context = ProfileValidationContext::new().with_object_state(
        0,
        ProfileObjectState::new(OBJECT_ID)
            .with_usage_limits_present(Some(true))
            .with_protection_use_started(Some(true))
            .with_get_usage_allocation_performed(Some(false))
            .with_usage_limits_total(Some(50))
            .with_usage_limits_count(Some(50)),
    );

    let error = client
        .execute_with_profiles_and_context(batch, &selection, &context, &CodecLimits::defaults())
        .expect_err("Usage Limits Total cannot change after protection use begins");

    assert_eq!(exchanges.get(), 0);
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    let profile_error = error
        .profile_validation_error()
        .expect("the prohibited request carries a typed profile error");
    assert_eq!(
        profile_error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(profile_error.requirement_id(), Some(REQUIREMENT));
    assert_eq!(profile_error.source_clause_id(), Some(CLAUSE));
    assert_eq!(profile_error.batch_item_index(), Some(0));
    assert!(!format!("{profile_error}").contains(OBJECT_ID));
}

#[test]
fn usage_limits_total_change_after_allocation_is_attributed_only_when_count_changed() {
    const REQUIREMENT: &str = "KMIPKIT-REQ-SPEC-4.59-003-001";
    const CLAUSE: &str = "KMIPKIT-CLAUSE-SPEC-4.59-003";
    const USAGE_LIMITS: u32 = 0x0042_0095;
    const TOTAL: u32 = 0x0042_0097;
    const COUNT: u32 = 0x0042_0096;
    const UNIT: u32 = 0x0042_0098;
    const OBJECT_ID: &str = "allocation-context-sensitive-id";

    let exchanges = Rc::new(Cell::new(0));
    let mut client = Client::for_test(CountingTransport::new(Rc::clone(&exchanges)));
    let usage_limits = test_item(
        USAGE_LIMITS,
        Value::structure(test_structure([
            test_item(TOTAL, Value::long_integer(40)),
            test_item(COUNT, Value::long_integer(50)),
            test_item(UNIT, Value::enumeration(1)),
        ])),
    );
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::set_attribute(
        SetAttributeRequest::new(Some(OBJECT_ID.to_owned()), NewAttribute::new(usage_limits)),
    )));
    let selection = ProfileValidationSelection::for_test([BASELINE]);
    let context = ProfileValidationContext::new().with_object_state(
        0,
        ProfileObjectState::new(OBJECT_ID)
            .with_usage_limits_present(Some(true))
            .with_protection_use_started(Some(false))
            .with_get_usage_allocation_performed(Some(true))
            .with_get_usage_allocation_changed_count(Some(true))
            .with_usage_limits_total(Some(50))
            .with_usage_limits_count(Some(50)),
    );

    let error = client
        .execute_with_profiles_and_context(batch, &selection, &context, &CodecLimits::defaults())
        .expect_err("Total cannot change after allocation changed Count");

    assert_eq!(exchanges.get(), 0);
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    let profile_error = error
        .profile_validation_error()
        .expect("the prohibited request carries a typed profile error");
    assert_eq!(
        profile_error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(profile_error.requirement_id(), Some(REQUIREMENT));
    assert_eq!(profile_error.source_clause_id(), Some(CLAUSE));
    assert_eq!(profile_error.batch_item_index(), Some(0));
    assert!(!format!("{profile_error}").contains(OBJECT_ID));
}

#[test]
fn usage_limits_total_change_after_allocation_without_count_change_uses_general_prohibition() {
    const REQUIREMENT: &str = "KMIPKIT-REQ-SPEC-4.59-002";
    const CLAUSE: &str = "KMIPKIT-CLAUSE-SPEC-4.59-002";
    const USAGE_LIMITS: u32 = 0x0042_0095;
    const TOTAL: u32 = 0x0042_0097;
    const COUNT: u32 = 0x0042_0096;
    const UNIT: u32 = 0x0042_0098;
    const OBJECT_ID: &str = "allocation-context-count-unchanged-id";

    let exchanges = Rc::new(Cell::new(0));
    let mut client = Client::for_test(CountingTransport::new(Rc::clone(&exchanges)));
    let usage_limits = test_item(
        USAGE_LIMITS,
        Value::structure(test_structure([
            test_item(TOTAL, Value::long_integer(40)),
            test_item(COUNT, Value::long_integer(50)),
            test_item(UNIT, Value::enumeration(1)),
        ])),
    );
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::set_attribute(
        SetAttributeRequest::new(Some(OBJECT_ID.to_owned()), NewAttribute::new(usage_limits)),
    )));
    let selection = ProfileValidationSelection::for_test([BASELINE]);
    let context = ProfileValidationContext::new().with_object_state(
        0,
        ProfileObjectState::new(OBJECT_ID)
            .with_usage_limits_present(Some(true))
            .with_protection_use_started(Some(false))
            .with_get_usage_allocation_performed(Some(true))
            .with_get_usage_allocation_changed_count(Some(false))
            .with_usage_limits_total(Some(50))
            .with_usage_limits_count(Some(50)),
    );

    let error = client
        .execute_with_profiles_and_context(batch, &selection, &context, &CodecLimits::defaults())
        .expect_err("Total cannot change after allocation even when Count did not change");

    assert_eq!(exchanges.get(), 0);
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    let profile_error = error
        .profile_validation_error()
        .expect("the prohibited request carries a typed profile error");
    assert_eq!(
        profile_error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(profile_error.requirement_id(), Some(REQUIREMENT));
    assert_eq!(profile_error.source_clause_id(), Some(CLAUSE));
    assert_eq!(profile_error.batch_item_index(), Some(0));
    assert!(!format!("{profile_error}").contains(OBJECT_ID));
}

#[test]
fn original_creation_date_can_be_added_when_the_bound_context_says_absent() {
    const ADD_ATTRIBUTE: u32 = 0x0000_000D;
    const ORIGINAL_CREATION_DATE: u32 = 0x0042_00BC;
    const OBJECT_ID: &str = "profile-context-absent-attribute-id";

    let exchanges = Rc::new(Cell::new(0));
    let transport = CountingTransport::with_responses(
        Rc::clone(&exchanges),
        [one_item_response_bytes(
            ADD_ATTRIBUTE,
            0,
            None,
            None,
            None,
            None,
            Some(test_structure([test_item(
                0x0042_0094,
                Value::text_string(OBJECT_ID.to_owned()),
            )])),
        )],
    );
    let mut client = Client::for_test(transport);
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::add_attribute(
        AddAttributeRequest::new(
            Some(OBJECT_ID.to_owned()),
            NewAttribute::new(test_item(
                ORIGINAL_CREATION_DATE,
                Value::date_time(1_700_000_000),
            )),
        ),
    )));
    let selection = ProfileValidationSelection::for_test([BASELINE]);
    let context = ProfileValidationContext::new().with_object_state(
        0,
        ProfileObjectState::new(OBJECT_ID).with_original_creation_date_present(Some(false)),
    );

    client
        .execute_with_profiles_and_context(batch, &selection, &context, &CodecLimits::defaults())
        .expect("§4.38 expressly permits later Add when the attribute is absent");

    assert_eq!(exchanges.get(), 1);
}

#[test]
fn original_creation_date_mutation_without_context_fails_closed_before_transport() {
    const ORIGINAL_CREATION_DATE: u32 = 0x0042_00BC;
    const OBJECT_ID: &str = "missing-state-context-sensitive-id";

    let exchanges = Rc::new(Cell::new(0));
    let mut client = Client::for_test(CountingTransport::new(Rc::clone(&exchanges)));
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::modify_attribute(
        ModifyAttributeRequest::new(
            Some(OBJECT_ID.to_owned()),
            None,
            NewAttribute::new(test_item(
                ORIGINAL_CREATION_DATE,
                Value::date_time(1_700_000_002),
            )),
        ),
    )));
    let selection = ProfileValidationSelection::for_test([BASELINE]);

    let error = client
        .execute_with_profiles(batch, &selection, &CodecLimits::defaults())
        .expect_err("unknown caller state cannot authorize a mutation");

    assert_eq!(exchanges.get(), 0);
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    let profile_error = error
        .profile_validation_error()
        .expect("missing caller state is a profile preflight failure");
    assert_eq!(
        profile_error.code(),
        ProfileValidationErrorCode::ProfileStateContextRequired
    );
    assert_eq!(profile_error.batch_item_index(), Some(0));
    assert!(!format!("{profile_error}").contains(OBJECT_ID));
}

#[test]
fn original_creation_date_mutation_with_unknown_state_fails_closed_before_transport() {
    const ORIGINAL_CREATION_DATE: u32 = 0x0042_00BC;
    const OBJECT_ID: &str = "unknown-state-context-sensitive-id";

    let exchanges = Rc::new(Cell::new(0));
    let mut client = Client::for_test(CountingTransport::new(Rc::clone(&exchanges)));
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::modify_attribute(
        ModifyAttributeRequest::new(
            Some(OBJECT_ID.to_owned()),
            None,
            NewAttribute::new(test_item(
                ORIGINAL_CREATION_DATE,
                Value::date_time(1_700_000_005),
            )),
        ),
    )));
    let selection = ProfileValidationSelection::for_test([BASELINE]);
    let context = ProfileValidationContext::new().with_object_state(
        0,
        ProfileObjectState::new(OBJECT_ID).with_original_creation_date_present(None),
    );

    let error = client
        .execute_with_profiles_and_context(batch, &selection, &context, &CodecLimits::defaults())
        .expect_err("unknown is distinct from an attested absence");

    assert_eq!(exchanges.get(), 0);
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    let profile_error = error
        .profile_validation_error()
        .expect("unknown caller state is a profile preflight failure");
    assert_eq!(
        profile_error.code(),
        ProfileValidationErrorCode::ProfileStateContextRequired
    );
    assert_eq!(profile_error.batch_item_index(), Some(0));
}

#[test]
fn object_state_bound_to_another_identifier_fails_closed_before_transport() {
    const ORIGINAL_CREATION_DATE: u32 = 0x0042_00BC;
    const OBJECT_ID: &str = "request-state-binding-object-id";
    const OTHER_OBJECT_ID: &str = "different-context-object-id";

    let exchanges = Rc::new(Cell::new(0));
    let mut client = Client::for_test(CountingTransport::new(Rc::clone(&exchanges)));
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::modify_attribute(
        ModifyAttributeRequest::new(
            Some(OBJECT_ID.to_owned()),
            None,
            NewAttribute::new(test_item(
                ORIGINAL_CREATION_DATE,
                Value::date_time(1_700_000_003),
            )),
        ),
    )));
    let selection = ProfileValidationSelection::for_test([BASELINE]);
    let context = ProfileValidationContext::new().with_object_state(
        0,
        ProfileObjectState::new(OTHER_OBJECT_ID).with_original_creation_date_present(Some(true)),
    );

    let error = client
        .execute_with_profiles_and_context(batch, &selection, &context, &CodecLimits::defaults())
        .expect_err("state for another object cannot authorize this request");

    assert_eq!(exchanges.get(), 0);
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    let profile_error = error
        .profile_validation_error()
        .expect("a mismatched object binding is a profile preflight failure");
    assert_eq!(
        profile_error.code(),
        ProfileValidationErrorCode::ProfileStateBindingMismatch
    );
    assert_eq!(profile_error.batch_item_index(), Some(0));
    let display = format!("{profile_error}");
    assert!(!display.contains(OBJECT_ID));
    assert!(!display.contains(OTHER_OBJECT_ID));
    assert!(!format!("{context:?}").contains(OTHER_OBJECT_ID));
}

#[test]
fn duplicate_state_entries_for_one_batch_item_fail_closed_before_transport() {
    const ORIGINAL_CREATION_DATE: u32 = 0x0042_00BC;
    const OBJECT_ID: &str = "duplicate-state-context-sensitive-id";

    let exchanges = Rc::new(Cell::new(0));
    let mut client = Client::for_test(CountingTransport::new(Rc::clone(&exchanges)));
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::modify_attribute(
        ModifyAttributeRequest::new(
            Some(OBJECT_ID.to_owned()),
            None,
            NewAttribute::new(test_item(
                ORIGINAL_CREATION_DATE,
                Value::date_time(1_700_000_006),
            )),
        ),
    )));
    let selection = ProfileValidationSelection::for_test([BASELINE]);
    let state =
        || ProfileObjectState::new(OBJECT_ID).with_original_creation_date_present(Some(true));
    let context = ProfileValidationContext::new()
        .with_object_state(0, state())
        .with_object_state(0, state());

    let error = client
        .execute_with_profiles_and_context(batch, &selection, &context, &CodecLimits::defaults())
        .expect_err("duplicate state for one item is ambiguous");

    assert_eq!(exchanges.get(), 0);
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    let profile_error = error
        .profile_validation_error()
        .expect("duplicate caller state is a profile preflight failure");
    assert_eq!(
        profile_error.code(),
        ProfileValidationErrorCode::ProfileStateBindingMismatch
    );
    assert_eq!(profile_error.batch_item_index(), Some(0));
}

#[test]
fn object_state_bound_to_another_batch_item_fails_closed_before_transport() {
    const ORIGINAL_CREATION_DATE: u32 = 0x0042_00BC;
    const OBJECT_ID: &str = "batch-item-context-object-id";

    let exchanges = Rc::new(Cell::new(0));
    let mut client = Client::for_test(CountingTransport::new(Rc::clone(&exchanges)));
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::modify_attribute(
        ModifyAttributeRequest::new(
            Some(OBJECT_ID.to_owned()),
            None,
            NewAttribute::new(test_item(
                ORIGINAL_CREATION_DATE,
                Value::date_time(1_700_000_004),
            )),
        ),
    )));
    let selection = ProfileValidationSelection::for_test([BASELINE]);
    let context = ProfileValidationContext::new().with_object_state(
        1,
        ProfileObjectState::new(OBJECT_ID).with_original_creation_date_present(Some(true)),
    );

    let error = client
        .execute_with_profiles_and_context(batch, &selection, &context, &CodecLimits::defaults())
        .expect_err("state for batch item 1 cannot authorize batch item 0");

    assert_eq!(exchanges.get(), 0);
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    let profile_error = error
        .profile_validation_error()
        .expect("missing per-item state is a profile preflight failure");
    assert_eq!(
        profile_error.code(),
        ProfileValidationErrorCode::ProfileStateContextRequired
    );
    assert_eq!(profile_error.batch_item_index(), Some(0));
}

#[test]
fn usage_limits_total_set_preserves_count_when_protection_has_not_started() {
    const SET_ATTRIBUTE: u32 = 0x0000_0031;
    const USAGE_LIMITS: u32 = 0x0042_0095;
    const TOTAL: u32 = 0x0042_0097;
    const COUNT: u32 = 0x0042_0096;
    const UNIT: u32 = 0x0042_0098;
    const OBJECT_ID: &str = "usage-limits-permitted-set-object-id";

    let exchanges = Rc::new(Cell::new(0));
    let denial = one_item_response_bytes(
        SET_ATTRIBUTE,
        1,
        Some(1),
        Some("fixture server denial after valid preflight"),
        None,
        None,
        None,
    );
    let transport = CountingTransport::with_responses(Rc::clone(&exchanges), [denial]);
    let mut client = Client::for_test(transport);
    let usage_limits = test_item(
        USAGE_LIMITS,
        Value::structure(test_structure([
            test_item(TOTAL, Value::long_integer(40)),
            test_item(COUNT, Value::long_integer(50)),
            test_item(UNIT, Value::enumeration(1)),
        ])),
    );
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::set_attribute(
        SetAttributeRequest::new(Some(OBJECT_ID.to_owned()), NewAttribute::new(usage_limits)),
    )));
    let selection = ProfileValidationSelection::for_test([BASELINE]);
    let context = ProfileValidationContext::new().with_object_state(
        0,
        ProfileObjectState::new(OBJECT_ID)
            .with_usage_limits_present(Some(true))
            .with_protection_use_started(Some(false))
            .with_get_usage_allocation_performed(Some(false))
            .with_get_usage_allocation_changed_count(Some(false))
            .with_usage_limits_total(Some(50))
            .with_usage_limits_count(Some(50)),
    );

    client
        .execute_with_profiles_and_context(batch, &selection, &context, &CodecLimits::defaults())
        .expect("changing Total while preserving Count before use is allowed");

    assert_eq!(exchanges.get(), 1);
}

#[test]
fn usage_limits_total_set_rejects_missing_required_unit_before_transport() {
    // KMIP Specification v2.1 §7.40 Table 392 requires Total, Count, and Unit.
    const USAGE_LIMITS: u32 = 0x0042_0095;
    const TOTAL: u32 = 0x0042_0097;
    const COUNT: u32 = 0x0042_0096;
    const OBJECT_ID: &str = "usage-limits-missing-unit-object-id";

    let exchanges = Rc::new(Cell::new(0));
    let mut client = Client::for_test(CountingTransport::new(Rc::clone(&exchanges)));
    let usage_limits = test_item(
        USAGE_LIMITS,
        Value::structure(test_structure([
            test_item(TOTAL, Value::long_integer(40)),
            test_item(COUNT, Value::long_integer(50)),
        ])),
    );
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::set_attribute(
        SetAttributeRequest::new(Some(OBJECT_ID.to_owned()), NewAttribute::new(usage_limits)),
    )));
    let selection = ProfileValidationSelection::for_test([BASELINE]);
    let context = ProfileValidationContext::new().with_object_state(
        0,
        ProfileObjectState::new(OBJECT_ID)
            .with_usage_limits_present(Some(true))
            .with_protection_use_started(Some(false))
            .with_get_usage_allocation_performed(Some(false))
            .with_get_usage_allocation_changed_count(Some(false))
            .with_usage_limits_total(Some(50))
            .with_usage_limits_count(Some(50)),
    );

    let error = client
        .execute_with_profiles_and_context(batch, &selection, &context, &CodecLimits::defaults())
        .expect_err("Set Attribute must reject an incomplete Usage Limits structure");

    assert_eq!(exchanges.get(), 0);
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(
        error
            .profile_validation_error()
            .expect("malformed profile request has a typed validation error")
            .code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
}

#[test]
fn usage_limits_total_set_cannot_change_count() {
    const REQUIREMENT: &str = "KMIPKIT-REQ-SPEC-4.59-003-002";
    const CLAUSE: &str = "KMIPKIT-CLAUSE-SPEC-4.59-003";
    const USAGE_LIMITS: u32 = 0x0042_0095;
    const TOTAL: u32 = 0x0042_0097;
    const COUNT: u32 = 0x0042_0096;
    const UNIT: u32 = 0x0042_0098;
    const OBJECT_ID: &str = "usage-limits-count-change-object-id";

    let exchanges = Rc::new(Cell::new(0));
    let mut client = Client::for_test(CountingTransport::new(Rc::clone(&exchanges)));
    let usage_limits = test_item(
        USAGE_LIMITS,
        Value::structure(test_structure([
            test_item(TOTAL, Value::long_integer(40)),
            test_item(COUNT, Value::long_integer(49)),
            test_item(UNIT, Value::enumeration(1)),
        ])),
    );
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::set_attribute(
        SetAttributeRequest::new(Some(OBJECT_ID.to_owned()), NewAttribute::new(usage_limits)),
    )));
    let selection = ProfileValidationSelection::for_test([BASELINE]);
    let context = ProfileValidationContext::new().with_object_state(
        0,
        ProfileObjectState::new(OBJECT_ID)
            .with_usage_limits_present(Some(true))
            .with_protection_use_started(Some(false))
            .with_get_usage_allocation_performed(Some(false))
            .with_get_usage_allocation_changed_count(Some(false))
            .with_usage_limits_total(Some(50))
            .with_usage_limits_count(Some(50)),
    );

    let error = client
        .execute_with_profiles_and_context(batch, &selection, &context, &CodecLimits::defaults())
        .expect_err("Set Attribute cannot change Usage Limits Count");

    assert_eq!(exchanges.get(), 0);
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    let profile_error = error
        .profile_validation_error()
        .expect("Count mutation has a typed profile error");
    assert_eq!(
        profile_error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(profile_error.requirement_id(), Some(REQUIREMENT));
    assert_eq!(profile_error.source_clause_id(), Some(CLAUSE));
    assert_eq!(profile_error.batch_item_index(), Some(0));
}

#[test]
fn profile_exchange_preserves_reordered_batch_items_for_response_validation() {
    // KMIP Specification v2.1 §8.3 correlates batch items by their unique IDs.
    const LINK_ATTRIBUTE: u32 = 0x0042_004A;
    const LINK_TYPE: u32 = 0x0042_004B;
    const LINKED_OBJECT_IDENTIFIER: u32 = 0x0042_004C;

    let valid_response = profile_link_batch_response(
        0,
        Some(test_structure([test_item(
            LINK_ATTRIBUTE,
            Value::structure(test_structure([
                test_item(LINK_TYPE, Value::enumeration(1)),
                test_item(
                    LINKED_OBJECT_IDENTIFIER,
                    Value::text_string("linked-object".to_owned()),
                ),
            ])),
        )])),
    );
    execute_link_profile_batch(valid_response, 0)
        .expect("the profile exchange validator receives correlated reordered responses");

    let failed_get_attributes_response = profile_link_batch_response(1, None);
    execute_link_profile_batch(failed_get_attributes_response, 1)
        .expect("a failed Get Attributes result remains correlated by its batch ID");
}

struct ReorderedBatchValidator {
    expected_get_attributes_status: u32,
}

impl ProfileExchangeValidator for ReorderedBatchValidator {
    fn validate_transport(
        &self,
        _transport: ProfileTransport,
    ) -> Result<(), ProfileValidationError> {
        Ok(())
    }

    fn validate_request(&self, request: &RequestMessage) -> Result<(), ProfileValidationError> {
        let items = request.batch_items().collect::<Vec<_>>();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].operation(), Some(0x0000_000B));
        assert_eq!(
            items[0].with_unique_batch_item_id(<[u8]>::to_vec),
            Some(b"get-attributes".to_vec())
        );
        assert_eq!(items[1].operation(), Some(0x0000_001E));
        assert_eq!(
            items[1].with_unique_batch_item_id(<[u8]>::to_vec),
            Some(b"discover-versions".to_vec())
        );
        Ok(())
    }

    fn validate_response(
        &self,
        request: &RequestMessage,
        response: &ResponseMessage,
    ) -> Result<(), ProfileValidationError> {
        assert_eq!(request.batch_items().len(), 2);
        let items = response.batch_items().collect::<Vec<_>>();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].operation(), Some(0x0000_001E));
        assert_eq!(
            items[0].with_unique_batch_item_id(<[u8]>::to_vec),
            Some(b"discover-versions".to_vec())
        );
        assert_eq!(items[1].operation(), Some(0x0000_000B));
        assert_eq!(
            items[1].with_unique_batch_item_id(<[u8]>::to_vec),
            Some(b"get-attributes".to_vec())
        );
        assert_eq!(
            items[1]
                .result_status()
                .map(kmipkit_protocol::ResultStatus::raw),
            Some(self.expected_get_attributes_status)
        );
        Ok(())
    }
}

fn execute_link_profile_batch(
    response: Vec<u8>,
    expected_get_attributes_status: u32,
) -> Result<(), crate::ClientError> {
    let exchanges = Rc::new(Cell::new(0));
    let transport = CountingTransport::with_responses(Rc::clone(&exchanges), [response]);
    let mut client = Client::for_test(transport);
    let validator = ReorderedBatchValidator {
        expected_get_attributes_status,
    };
    let get_attributes = GetAttributesRequest::try_new(Some("object".to_owned()), [])
        .expect("an empty Attribute Reference list requests all attributes");
    let batch = ClientBatch::from_items([
        ClientBatchItem::new(ClientRequest::get_attributes(get_attributes))
            .with_unique_batch_item_id(b"get-attributes".to_vec()),
        ClientBatchItem::new(ClientRequest::discover_versions())
            .with_unique_batch_item_id(b"discover-versions".to_vec()),
    ]);

    client
        .execute_inner(
            batch,
            &CodecLimits::defaults(),
            &kmipkit_transport::RequestOptions::default(),
            Some(&validator),
        )
        .map(|_| ())
}

fn profile_link_batch_response(get_attributes_status: u32, links: Option<Structure>) -> Vec<u8> {
    const RESPONSE_HEADER: u32 = 0x0042_007A;
    const PROTOCOL_VERSION: u32 = 0x0042_0069;
    const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
    const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
    const TIME_STAMP: u32 = 0x0042_0092;
    const BATCH_COUNT: u32 = 0x0042_000D;
    const BATCH_ITEM: u32 = 0x0042_000F;
    const OPERATION: u32 = 0x0042_005C;
    const UNIQUE_BATCH_ITEM_ID: u32 = 0x0042_0093;
    const RESULT_STATUS: u32 = 0x0042_007F;
    const RESULT_REASON: u32 = 0x0042_007E;
    const RESULT_MESSAGE: u32 = 0x0042_007D;
    const RESPONSE_PAYLOAD: u32 = 0x0042_007C;
    const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
    const ATTRIBUTES: u32 = 0x0042_0125;
    const GET_ATTRIBUTES: u32 = 0x0000_000B;
    const DISCOVER_VERSIONS: u32 = 0x0000_001E;

    let response_version = test_structure([
        test_item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        test_item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
    ]);
    let header = test_structure([
        test_item(PROTOCOL_VERSION, Value::structure(response_version)),
        test_item(TIME_STAMP, Value::date_time(1)),
        test_item(BATCH_COUNT, Value::integer(2)),
    ]);
    let mut get_fields = vec![
        test_item(OPERATION, Value::enumeration(GET_ATTRIBUTES)),
        test_item(
            UNIQUE_BATCH_ITEM_ID,
            Value::byte_string(b"get-attributes".to_vec()),
        ),
        test_item(RESULT_STATUS, Value::enumeration(get_attributes_status)),
    ];
    if get_attributes_status == 0 {
        let attributes = links.unwrap_or_else(|| test_structure([]));
        get_fields.push(test_item(
            RESPONSE_PAYLOAD,
            Value::structure(test_structure([
                test_item(UNIQUE_IDENTIFIER, Value::text_string("object".to_owned())),
                test_item(ATTRIBUTES, Value::structure(attributes)),
            ])),
        ));
    } else {
        get_fields.push(test_item(RESULT_REASON, Value::enumeration(1)));
        get_fields.push(test_item(
            RESULT_MESSAGE,
            Value::text_string("fixture failure".to_owned()),
        ));
    }
    let get_response = test_item(BATCH_ITEM, Value::structure(test_structure(get_fields)));
    let discover_payload = test_structure([test_item(
        PROTOCOL_VERSION,
        Value::structure(test_structure([
            test_item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
            test_item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
        ])),
    )]);
    let discover_response = test_item(
        BATCH_ITEM,
        Value::structure(test_structure([
            test_item(OPERATION, Value::enumeration(DISCOVER_VERSIONS)),
            test_item(
                UNIQUE_BATCH_ITEM_ID,
                Value::byte_string(b"discover-versions".to_vec()),
            ),
            test_item(RESULT_STATUS, Value::enumeration(0)),
            test_item(RESPONSE_PAYLOAD, Value::structure(discover_payload)),
        ])),
    );

    let mut message = vec![test_item(RESPONSE_HEADER, Value::structure(header))];
    message.extend([discover_response, get_response]);
    encode_message_for_test(test_structure(message), &CodecLimits::defaults())
        .expect("the reordered mixed-batch response fixture is valid")
}
