//! Explicit profile selection must fail closed before an exchange until its
//! complete typed rule registry is available. Source: KMIP Specification v2.1
//! §14.1; KMIP Profiles v2.1 §6.1. Traceability: KMIPKIT-0010 FR-014, SC-009,
//! and SC-014.

use std::cell::Cell;
use std::rc::Rc;

use kmipkit_protocol::{
    ProfileTransport, ProfileValidationError, ProfileValidationErrorCode,
    ProfileValidationSelection, RequestMessage, ResponseMessage,
    profiles_generated::CLIENT_PROFILES,
};
use kmipkit_transport::{RequestDeliveryState, Transport, TransportError, TransportResponse};
use kmipkit_ttlv::codec::CodecLimits;

use crate::execute::ProfileExchangeValidator;
use crate::execute_test_support::{ResponseItemFixture, response_bytes};
use crate::{Client, ClientBatch, ClientBatchItem, ClientErrorCategory, ClientRequest};

const BASELINE: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";

struct CountingTransport {
    exchanges: Rc<Cell<usize>>,
    response: Vec<u8>,
}

impl CountingTransport {
    fn new(exchanges: Rc<Cell<usize>>) -> Self {
        Self {
            exchanges,
            response: response_bytes((2, 1), &[ResponseItemFixture::success(None)]),
        }
    }
}

impl Transport for CountingTransport {
    fn exchange(
        &mut self,
        _request: &[u8],
        _max_response_bytes: usize,
    ) -> Result<TransportResponse, TransportError> {
        self.exchanges.set(self.exchanges.get() + 1);
        Ok(TransportResponse::new(self.response.clone()))
    }
}

fn discover_versions_batch() -> ClientBatch {
    ClientBatch::new(ClientBatchItem::new(ClientRequest::discover_versions()))
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

struct ResponseRejectingProfile {
    error: ProfileValidationError,
    request_checks: Option<Rc<Cell<usize>>>,
}

impl ProfileExchangeValidator for ResponseRejectingProfile {
    fn validate_transport(
        &self,
        _transport: ProfileTransport,
    ) -> Result<(), ProfileValidationError> {
        Ok(())
    }

    fn validate_request(&self, _request: &RequestMessage) -> Result<(), ProfileValidationError> {
        if let Some(request_checks) = &self.request_checks {
            request_checks.set(request_checks.get() + 1);
        }
        Ok(())
    }

    fn validate_response(
        &self,
        request: &RequestMessage,
        response: &ResponseMessage,
    ) -> Result<(), ProfileValidationError> {
        let request_operation = request
            .batch_items()
            .next()
            .and_then(|item| item.operation());
        let response_operation = response
            .batch_items()
            .next()
            .and_then(|item| item.operation());
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
    let validator = ResponseRejectingProfile {
        error: ProfileValidationError::for_test(
            ProfileValidationErrorCode::ResponseRuleRejected,
            "KMIPKIT-TEST-PROFILE",
            "KMIPKIT-TEST-REQUIREMENT",
            "KMIPKIT-TEST-CLAUSE",
        ),
        request_checks: None,
    };
    let exchanges = Rc::new(Cell::new(0));
    let mut client = Client::for_test(CountingTransport::new(Rc::clone(&exchanges)));

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
    let validator = ResponseRejectingProfile {
        error: ProfileValidationError::for_test(
            ProfileValidationErrorCode::ResponseRuleRejected,
            "KMIPKIT-TEST-PROFILE",
            "KMIPKIT-TEST-REQUIREMENT",
            "KMIPKIT-TEST-CLAUSE",
        ),
        request_checks: Some(Rc::clone(&request_checks)),
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
