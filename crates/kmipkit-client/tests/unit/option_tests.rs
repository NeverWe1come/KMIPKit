//! Derived client-policy tests for KMIP 2.1 options, Pending results, and
//! message extensions. These are not official OASIS test vectors.
//!
//! OASIS KMIP Specification v2.1 §§9.2, 9.6, 9.13, 9.19, 11.3, and 11.5;
//! Tables 432 and 435. `KMIPKIT-DISC-001` remains open: these tests make no
//! assertion about Continue or Undo execution, rollback, or retry effects.
//! `KMIPKIT-DEC-002` applies only the assigned-value outbound policy and does
//! not call the §11.5 extension allocation invalid.
//!
//! Traceability: `KMIPKIT-0007-FR-007`, `-FR-008`, `-FR-010`, `-SC-002`,
//! `KMIPKIT-REQ-SPEC-8-003-002`, `KMIPKIT-REQ-SPEC-9.2-001`,
//! `KMIPKIT-REQ-SPEC-9.6-001-001`, `-001-002`, `-001-003`,
//! `KMIPKIT-REQ-SPEC-9.13-001-004`, `-001-005`,
//! `KMIPKIT-REQ-SPEC-9.19-002`, `KMIPKIT-DISC-001`, and
//! `KMIPKIT-DISC-043` / `KMIPKIT-DEC-002`.

use kmipkit_protocol::RequestMessage;
use kmipkit_ttlv::{Item, RawTag, Structure, Tag, Value};

use crate::execute::{
    OutcomeValidationError, PendingState, validate_async_indicator_for_test,
    validate_batch_error_continuation_for_test, validate_pending_states_for_test,
    validate_unknown_extension_for_test,
};

const ASYNCHRONOUS_INDICATOR_MANDATORY: u32 = 1;
const ASYNCHRONOUS_INDICATOR_OPTIONAL: u32 = 2;
const ASYNCHRONOUS_INDICATOR_PROHIBITED: u32 = 3;
const BATCH_ERROR_CONTINUATION_CONTINUE: u32 = 1;
const BATCH_ERROR_CONTINUATION_STOP: u32 = 2;
const BATCH_ERROR_CONTINUATION_UNDO: u32 = 3;
const ENUMERATION_EXTENSION_MIN: u32 = 0x8000_0000;
const RESULT_STATUS_SUCCESS: u32 = 0;
const RESULT_STATUS_PENDING: u32 = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CandidateValidationError {
    InvalidOption,
    SingleItemOption,
    RepeatedOption,
    PendingNotPermitted,
    PendingCorrelationMissing,
    UnknownCriticalExtension,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct CandidateBatchErrorOption {
    encoded: Option<u32>,
    effective: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum CandidateBatchOutcome {
    Completed,
    Pending {
        asynchronous_correlation_value: Option<Vec<u8>>,
    },
}

impl CandidateBatchOutcome {
    const fn result_status_raw(&self) -> u32 {
        match self {
            Self::Completed => RESULT_STATUS_SUCCESS,
            Self::Pending { .. } => RESULT_STATUS_PENDING,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CandidateUnknownExtension {
    critical: bool,
    opaque_ttlv: Vec<u8>,
}

// Deliberately incomplete test-only seams. T011 replaces these candidates
// with the production request-option, response, and extension policies.
fn candidate_validate_outbound_async_indicator(
    raw: Option<u32>,
) -> Result<Option<u32>, CandidateValidationError> {
    validate_async_indicator_for_test(raw).map_err(|()| CandidateValidationError::InvalidOption)
}

fn candidate_build_batch_error_option(
    batch_count: usize,
    values: &[u32],
) -> Result<CandidateBatchErrorOption, CandidateValidationError> {
    let value = validate_batch_error_continuation_for_test(batch_count, values).map_err(
        |error| match error {
            crate::execute::BatchValidationError::RepeatedBatchErrorContinuation => {
                CandidateValidationError::RepeatedOption
            }
            crate::execute::BatchValidationError::SingleItemBatchErrorContinuation => {
                CandidateValidationError::SingleItemOption
            }
            crate::execute::BatchValidationError::InvalidBatchErrorContinuation
            | crate::execute::BatchValidationError::EmptyBatch
            | crate::execute::BatchValidationError::MissingBatchItemId
            | crate::execute::BatchValidationError::DuplicateBatchItemId
            | crate::execute::BatchValidationError::IneligibleIdPlaceholder
            | crate::execute::BatchValidationError::InvalidAsynchronousIndicator
            | crate::execute::BatchValidationError::ExtensionRegistryMismatch => {
                CandidateValidationError::InvalidOption
            }
        },
    )?;
    Ok(CandidateBatchErrorOption {
        encoded: value.encoded,
        effective: value.effective,
    })
}

fn candidate_validate_outbound_batch_error_continuation(
    raw: u32,
) -> Result<u32, CandidateValidationError> {
    validate_batch_error_continuation_for_test(2, &[raw])
        .map(|value| value.encoded.unwrap_or(BATCH_ERROR_CONTINUATION_STOP))
        .map_err(|_| CandidateValidationError::InvalidOption)
}

fn candidate_validate_response_outcomes(
    asynchronous_indicator: Option<u32>,
    outcomes: &[CandidateBatchOutcome],
) -> Result<Vec<CandidateBatchOutcome>, CandidateValidationError> {
    let states = outcomes
        .iter()
        .map(|outcome| match outcome {
            CandidateBatchOutcome::Completed => PendingState {
                pending: false,
                has_correlation_value: false,
            },
            CandidateBatchOutcome::Pending {
                asynchronous_correlation_value,
            } => PendingState {
                pending: true,
                has_correlation_value: asynchronous_correlation_value
                    .as_ref()
                    .is_some_and(|value| !value.is_empty()),
            },
        })
        .collect::<Vec<_>>();
    validate_pending_states_for_test(asynchronous_indicator, &states)
        .map(|()| outcomes.to_vec())
        .map_err(|error| match error {
            OutcomeValidationError::PendingNotPermitted => {
                CandidateValidationError::PendingNotPermitted
            }
            OutcomeValidationError::PendingCorrelationMissing => {
                CandidateValidationError::PendingCorrelationMissing
            }
            OutcomeValidationError::UnknownCriticalExtension => {
                CandidateValidationError::UnknownCriticalExtension
            }
        })
}

fn candidate_process_unknown_extension(
    extension: &CandidateUnknownExtension,
) -> Result<Option<Vec<u8>>, CandidateValidationError> {
    validate_unknown_extension_for_test(extension.critical)
        .map(|()| Some(extension.opaque_ttlv.clone()))
        .map_err(|_| CandidateValidationError::UnknownCriticalExtension)
}

#[test]
fn outbound_asynchronous_indicator_preserves_each_assigned_value() {
    for assigned in [
        ASYNCHRONOUS_INDICATOR_MANDATORY,
        ASYNCHRONOUS_INDICATOR_OPTIONAL,
        ASYNCHRONOUS_INDICATOR_PROHIBITED,
    ] {
        assert_eq!(
            candidate_validate_outbound_async_indicator(Some(assigned)),
            Ok(Some(assigned))
        );
    }
}

#[test]
fn outbound_asynchronous_indicator_preserves_extension_allocation_values() {
    let extension_value = ENUMERATION_EXTENSION_MIN;

    assert_eq!(
        candidate_validate_outbound_async_indicator(Some(extension_value)),
        Ok(Some(extension_value))
    );
}

#[test]
fn outbound_asynchronous_indicator_rejects_unassigned_values() {
    for unassigned in [0, 4, ENUMERATION_EXTENSION_MIN - 1] {
        assert_eq!(
            candidate_validate_outbound_async_indicator(Some(unassigned)),
            Err(CandidateValidationError::InvalidOption)
        );
    }
}

#[test]
fn omitted_batch_error_continuation_defaults_to_stop() {
    for batch_count in [1, 2] {
        let options = candidate_build_batch_error_option(batch_count, &[])
            .expect("an omitted option is valid for any non-empty batch");

        assert_eq!(options.encoded, None);
        assert_eq!(options.effective, BATCH_ERROR_CONTINUATION_STOP);
    }
}

#[test]
fn batch_error_continuation_may_be_present_for_a_multi_item_batch() {
    let options = candidate_build_batch_error_option(2, &[BATCH_ERROR_CONTINUATION_CONTINUE])
        .expect("a single option is allowed for a multi-item batch");

    assert_eq!(options.encoded, Some(BATCH_ERROR_CONTINUATION_CONTINUE));
    assert_eq!(options.effective, BATCH_ERROR_CONTINUATION_CONTINUE);
}

#[test]
fn batch_error_continuation_is_rejected_for_a_single_item_batch() {
    assert_eq!(
        candidate_build_batch_error_option(1, &[BATCH_ERROR_CONTINUATION_STOP]),
        Err(CandidateValidationError::SingleItemOption)
    );
}

#[test]
fn batch_error_continuation_has_at_most_one_header_value() {
    assert_eq!(
        candidate_build_batch_error_option(
            2,
            &[
                BATCH_ERROR_CONTINUATION_CONTINUE,
                BATCH_ERROR_CONTINUATION_UNDO,
            ],
        ),
        Err(CandidateValidationError::RepeatedOption)
    );
}

#[test]
fn outbound_batch_error_continuation_accepts_the_three_assigned_values() {
    for assigned in [
        BATCH_ERROR_CONTINUATION_CONTINUE,
        BATCH_ERROR_CONTINUATION_STOP,
        BATCH_ERROR_CONTINUATION_UNDO,
    ] {
        assert_eq!(
            candidate_validate_outbound_batch_error_continuation(assigned),
            Ok(assigned)
        );
    }
}

#[test]
fn outbound_batch_error_continuation_rejects_extension_allocation_value() {
    assert_eq!(
        candidate_validate_outbound_batch_error_continuation(ENUMERATION_EXTENSION_MIN),
        Err(CandidateValidationError::InvalidOption)
    );
}

#[test]
fn generic_message_model_preserves_unassigned_and_extension_batch_enumerations() {
    for raw in [0, ENUMERATION_EXTENSION_MIN, u32::MAX] {
        let message = RequestMessage::try_from_ttlv(request_tree_with_batch_option(raw))
            .expect("the generic message model does not apply outbound option policy");
        assert_eq!(
            message.header().batch_error_continuation_option(),
            Some(raw)
        );

        let reparsed = RequestMessage::try_from_ttlv(message.into_ttlv())
            .expect("the lossless raw message remains structurally valid");
        assert_eq!(
            reparsed.header().batch_error_continuation_option(),
            Some(raw)
        );
    }
}

#[test]
fn pending_response_is_accepted_when_mandatory_or_optional_is_set() {
    let pending = [CandidateBatchOutcome::Pending {
        asynchronous_correlation_value: Some(vec![0xA5]),
    }];

    for indicator in [
        ASYNCHRONOUS_INDICATOR_MANDATORY,
        ASYNCHRONOUS_INDICATOR_OPTIONAL,
    ] {
        assert_eq!(
            candidate_validate_response_outcomes(Some(indicator), &pending),
            Ok(pending.to_vec())
        );
    }
}

#[test]
fn pending_response_is_rejected_when_asynchronous_responses_are_prohibited() {
    let pending = [CandidateBatchOutcome::Pending {
        asynchronous_correlation_value: Some(vec![0xA5]),
    }];

    assert_eq!(
        candidate_validate_response_outcomes(Some(ASYNCHRONOUS_INDICATOR_PROHIBITED), &pending,),
        Err(CandidateValidationError::PendingNotPermitted)
    );
}

#[test]
fn pending_response_is_rejected_when_the_indicator_is_omitted() {
    let pending = [CandidateBatchOutcome::Pending {
        asynchronous_correlation_value: Some(vec![0xA5]),
    }];

    assert_eq!(
        candidate_validate_response_outcomes(None, &pending),
        Err(CandidateValidationError::PendingNotPermitted)
    );
}

#[test]
fn extension_indicator_does_not_grant_pending_permission_without_registry_policy() {
    let pending = [CandidateBatchOutcome::Pending {
        asynchronous_correlation_value: Some(vec![0xA5]),
    }];

    assert_eq!(
        candidate_validate_response_outcomes(Some(ENUMERATION_EXTENSION_MIN), &pending),
        Err(CandidateValidationError::PendingNotPermitted)
    );
}

#[test]
fn mixed_completed_and_pending_batch_is_preserved_when_pending_is_permitted() {
    let outcomes = [
        CandidateBatchOutcome::Completed,
        CandidateBatchOutcome::Pending {
            asynchronous_correlation_value: Some(vec![0xA5]),
        },
    ];

    assert_eq!(
        outcomes
            .iter()
            .map(CandidateBatchOutcome::result_status_raw)
            .collect::<Vec<_>>(),
        [RESULT_STATUS_SUCCESS, RESULT_STATUS_PENDING]
    );
    assert_eq!(
        candidate_validate_response_outcomes(Some(ASYNCHRONOUS_INDICATOR_OPTIONAL), &outcomes,),
        Ok(outcomes.to_vec())
    );
}

#[test]
fn mixed_batch_rejects_pending_when_asynchronous_responses_are_prohibited() {
    let outcomes = [
        CandidateBatchOutcome::Completed,
        CandidateBatchOutcome::Pending {
            asynchronous_correlation_value: Some(vec![0xA5]),
        },
    ];

    assert_eq!(
        candidate_validate_response_outcomes(Some(ASYNCHRONOUS_INDICATOR_PROHIBITED), &outcomes,),
        Err(CandidateValidationError::PendingNotPermitted)
    );
}

/// A Pending response carries the field required by OASIS KMIP Specification
/// v2.1 §8.6, Table 399 (with related context in §9.1), catalog element
/// `KMIPKIT-ELEM-MESSAGE-FIELD-8-6-ASYNCHRONOUS-CORRELATION-VALUE`. The
/// separately cited §9.19 / `KMIPKIT-REQ-SPEC-9.19-002` concerns using that
/// value in a subsequent Poll request.
#[test]
fn pending_response_requires_its_asynchronous_correlation_value() {
    let pending = [CandidateBatchOutcome::Pending {
        asynchronous_correlation_value: None,
    }];

    assert_eq!(
        candidate_validate_response_outcomes(Some(ASYNCHRONOUS_INDICATOR_OPTIONAL), &pending),
        Err(CandidateValidationError::PendingCorrelationMissing)
    );
}

#[test]
fn unknown_critical_message_extension_rejects_the_response() {
    let extension = CandidateUnknownExtension {
        critical: true,
        opaque_ttlv: opaque_extension_ttlv(),
    };

    assert_eq!(
        candidate_process_unknown_extension(&extension),
        Err(CandidateValidationError::UnknownCriticalExtension)
    );
}

#[test]
fn unknown_non_critical_message_extension_is_preserved_opaque() {
    let opaque_ttlv = opaque_extension_ttlv();
    let extension = CandidateUnknownExtension {
        critical: false,
        opaque_ttlv: opaque_ttlv.clone(),
    };

    assert_eq!(
        candidate_process_unknown_extension(&extension),
        Ok(Some(opaque_ttlv))
    );
}

fn opaque_extension_ttlv() -> Vec<u8> {
    vec![
        0x54, 0x53, 0x54, 0x08, 0x00, 0x00, 0x00, 0x04, 0xA5, 0x5A, 0x11, 0x22, 0x00, 0x00, 0x00,
        0x00,
    ]
}

fn request_tree_with_batch_option(raw: u32) -> Structure {
    const REQUEST_HEADER: u32 = 0x0042_0077;
    const PROTOCOL_VERSION: u32 = 0x0042_0069;
    const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
    const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
    const BATCH_ERROR_CONTINUATION_OPTION: u32 = 0x0042_000E;
    const BATCH_COUNT: u32 = 0x0042_000D;
    const BATCH_ITEM: u32 = 0x0042_000F;
    const OPERATION: u32 = 0x0042_005C;
    const UNIQUE_BATCH_ITEM_ID: u32 = 0x0042_0093;
    const REQUEST_PAYLOAD: u32 = 0x0042_0079;

    let protocol_version = structure([
        item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
    ]);
    let request_header = structure([
        item(PROTOCOL_VERSION, Value::structure(protocol_version)),
        item(BATCH_ERROR_CONTINUATION_OPTION, Value::enumeration(raw)),
        item(BATCH_COUNT, Value::integer(2)),
    ]);
    let batch_items = [1_u8, 2].map(|id| {
        item(
            BATCH_ITEM,
            Value::structure(structure([
                item(OPERATION, Value::enumeration(1)),
                item(UNIQUE_BATCH_ITEM_ID, Value::byte_string(vec![id])),
                item(REQUEST_PAYLOAD, Value::structure(Structure::new())),
            ])),
        )
    });

    structure(
        [item(REQUEST_HEADER, Value::structure(request_header))]
            .into_iter()
            .chain(batch_items),
    )
}

fn item(raw_tag: u32, value: Value) -> Item {
    Item::new(tag(raw_tag), value).expect("fixture tag and value have compatible TTLV types")
}

fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut structure = Structure::new();
    for child in items {
        structure
            .try_push(child)
            .expect("the fixture stays within the generic model limits");
    }
    structure
}

fn tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("fixture tag fits the KMIP tag width")
        .try_checked()
        .expect("fixture tag has a valid KMIP tag allocation")
}
