//! Derived tests for the KMIP 2.1 Discover Versions operation.
//!
//! The cases trace to `KMIPKIT-REQ-SPEC-6.1.16-001-001`,
//! `KMIPKIT-REQ-SPEC-6.1.16-001-002`, `KMIPKIT-REQ-SPEC-6.1.16-004`, and
//! `KMIPKIT-0007-FR-002`: OASIS KMIP Specification v2.1 §6.1.16, Tables
//! 211–213. Table 211 describes the client preference list, Table 212 allows
//! repeated response Protocol Version fields and defines their server
//! preference order without a uniqueness rule, and Table 213 lists ordinary
//! operation errors. Missing major/minor components are malformed under
//! §9.16, Table 421 and the catalog elements
//! `KMIPKIT-ELEM-MESSAGE-FIELD-9-16-PROTOCOL-VERSION-MAJOR/-MINOR`. These are
//! derived project tests, not official OASIS vectors.

use crate::{KmipOperationResult, ResultReason, ResultStatus};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct CandidateProtocolVersion {
    major: Option<i32>,
    minor: Option<i32>,
}

impl CandidateProtocolVersion {
    const fn complete(major: i32, minor: i32) -> Self {
        Self {
            major: Some(major),
            minor: Some(minor),
        }
    }

    const fn missing_major(minor: i32) -> Self {
        Self {
            major: None,
            minor: Some(minor),
        }
    }

    const fn missing_minor(major: i32) -> Self {
        Self {
            major: Some(major),
            minor: None,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum CandidateOperationResponse {
    Success(Vec<CandidateProtocolVersion>),
    OperationError(KmipOperationResult),
}

#[derive(Debug, Eq, PartialEq)]
enum CandidateValidatedResponse {
    SupportedVersions(Vec<CandidateProtocolVersion>),
    OperationError(KmipOperationResult),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CandidateValidationError {
    MalformedProtocolVersion,
    UnofferedProtocolVersion,
}

// Deliberately incomplete test seam. T009 replaces it with the typed model
// and operation validator; these tests must fail behaviorally until then.
fn candidate_request_versions() -> Vec<CandidateProtocolVersion> {
    Vec::new()
}

fn candidate_validate_response(
    response: CandidateOperationResponse,
) -> Result<CandidateValidatedResponse, CandidateValidationError> {
    match response {
        CandidateOperationResponse::Success(versions) => {
            Ok(CandidateValidatedResponse::SupportedVersions(versions))
        }
        CandidateOperationResponse::OperationError(result) => {
            Ok(CandidateValidatedResponse::OperationError(result))
        }
    }
}

fn kmip_2_1() -> CandidateProtocolVersion {
    CandidateProtocolVersion::complete(2, 1)
}

#[test]
fn request_advertises_only_kmip_2_1() {
    assert_eq!(candidate_request_versions(), vec![kmip_2_1()]);
}

#[test]
fn empty_success_response_is_accepted() {
    let actual = candidate_validate_response(CandidateOperationResponse::Success(Vec::new()));

    assert_eq!(
        actual,
        Ok(CandidateValidatedResponse::SupportedVersions(Vec::new()))
    );
}

#[test]
fn offered_response_versions_are_preserved_in_order_with_repetitions() {
    let offered = vec![kmip_2_1(), kmip_2_1()];
    let actual = candidate_validate_response(CandidateOperationResponse::Success(offered.clone()));

    assert_eq!(
        actual,
        Ok(CandidateValidatedResponse::SupportedVersions(offered))
    );
}

#[test]
fn response_rejects_a_protocol_version_that_was_not_offered() {
    let actual = candidate_validate_response(CandidateOperationResponse::Success(vec![
        CandidateProtocolVersion::complete(3, 0),
    ]));

    assert_eq!(
        actual,
        Err(CandidateValidationError::UnofferedProtocolVersion)
    );
}

#[test]
fn response_rejects_a_protocol_version_missing_its_major_component() {
    let actual = candidate_validate_response(CandidateOperationResponse::Success(vec![
        CandidateProtocolVersion::missing_major(1),
    ]));

    assert_eq!(
        actual,
        Err(CandidateValidationError::MalformedProtocolVersion)
    );
}

#[test]
fn response_rejects_a_protocol_version_missing_its_minor_component() {
    let actual = candidate_validate_response(CandidateOperationResponse::Success(vec![
        CandidateProtocolVersion::missing_minor(2),
    ]));

    assert_eq!(
        actual,
        Err(CandidateValidationError::MalformedProtocolVersion)
    );
}

#[test]
fn operation_errors_are_preserved_without_becoming_success_payloads() {
    let operation_error = KmipOperationResult::new(
        ResultStatus::from_raw(1),
        Some(ResultReason::from_raw(5)),
        None,
    )
    .expect("Table 213 Operation Failed / Operation Not Supported is a valid result");

    let actual = candidate_validate_response(CandidateOperationResponse::OperationError(
        operation_error.clone(),
    ));

    assert_eq!(
        actual,
        Ok(CandidateValidatedResponse::OperationError(operation_error))
    );
}
