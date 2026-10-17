//! Explicit profile selection must fail closed before an exchange until its
//! complete typed rule registry is available. Source: KMIP Specification v2.1
//! §14.1; KMIP Profiles v2.1 §6.1. Traceability: KMIPKIT-0010 FR-014, SC-009,
//! and SC-014.

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::error::Error;
use std::rc::Rc;

use kmipkit_protocol::{
    ProfileTransport, ProfileValidationError, ProfileValidationErrorCode,
    ProfileValidationSelection, QueryFunction, QueryRequest, RequestMessage, ResponseMessage,
    SetDefaultsRequest, profiles_generated::CLIENT_PROFILES,
};
use kmipkit_transport::{RequestDeliveryState, Transport, TransportError, TransportResponse};
use kmipkit_ttlv::codec::{CodecLimits, decode_with_limits};
use kmipkit_ttlv::{Value, ValueView};

use crate::execute::ProfileExchangeValidator;
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

    let selection = ProfileValidationSelection::new([BASELINE]);
    let after_query = client
        .execute_with_profiles(discover_versions_batch(), &selection, &limits)
        .expect_err("Query profile information is not a local conformance claim");
    assert_eq!(
        after_query.delivery_state(),
        Some(RequestDeliveryState::NotSent)
    );
    assert_eq!(
        after_query
            .profile_validation_error()
            .map(ProfileValidationError::code),
        Some(ProfileValidationErrorCode::UnresolvedRequirementScope)
    );
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

    let after_discover_versions = client
        .execute_with_profiles(discover_versions_batch(), &selection, &limits)
        .expect_err("a protocol-version response is not a profile conformance claim");
    assert_eq!(
        after_discover_versions.delivery_state(),
        Some(RequestDeliveryState::NotSent)
    );
    assert_eq!(
        after_discover_versions
            .profile_validation_error()
            .map(ProfileValidationError::code),
        Some(ProfileValidationErrorCode::UnresolvedRequirementScope)
    );
    assert_eq!(exchanges.get(), 3);
}

#[test]
fn incomplete_profile_selection_fails_before_transport_with_stable_source_ids() {
    let exchanges = Rc::new(Cell::new(0));
    let mut client = Client::for_test(CountingTransport::new(Rc::clone(&exchanges)));
    let profile = CLIENT_PROFILES
        .iter()
        .find(|profile| profile.profile_id == BASELINE)
        .expect("Baseline Client exists in generated profile metadata");
    let obligation = profile
        .unresolved_scope_obligations
        .first()
        .expect("Baseline contains a reviewed applicability gap");

    let error = client
        .execute_with_profiles(
            discover_versions_batch(),
            &ProfileValidationSelection::new([BASELINE]),
            &CodecLimits::defaults(),
        )
        .expect_err("an incomplete profile cannot reach the network");

    assert_eq!(error.category(), ClientErrorCategory::Validation);
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    let profile_error = error
        .profile_validation_error()
        .expect("profile validation keeps its stable structured source error");
    assert_eq!(
        profile_error.code(),
        ProfileValidationErrorCode::UnresolvedRequirementScope
    );
    assert_eq!(profile_error.profile_id(), Some(BASELINE));
    assert_eq!(
        profile_error.requirement_id(),
        Some(obligation.requirement_id)
    );
    assert_eq!(
        profile_error.source_clause_id(),
        Some(obligation.source_clause_id)
    );
    assert_eq!(exchanges.get(), 0);
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
