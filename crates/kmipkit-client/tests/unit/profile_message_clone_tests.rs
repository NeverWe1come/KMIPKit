use super::*;

#[test]
fn profiled_request_copy_rejects_the_configured_size_limit() {
    let limits =
        CodecLimits::new(0, 64, 100_000).expect("zero message bytes is a valid per-call limit");
    let request = request_message_for_test(
        ClientBatch::new(ClientBatchItem::new(ClientRequest::discover_versions())),
        &CodecLimits::defaults(),
    )
    .expect("Discover Versions creates a valid request message");

    let error = clone_request_message(&request, &limits)
        .expect_err("profile validation context must respect the configured encoded-size limit");

    assert_eq!(error.kind(), ProtocolErrorKind::InvalidValue);
    assert_eq!(
        error.cause_category(),
        ProtocolCauseCategory::InvalidEncoding
    );
}

#[test]
fn profiled_response_copy_preserves_the_complete_ttlv_tree() {
    let limits = CodecLimits::defaults();
    let bytes = crate::execute_test_support::response_bytes(
        (2, 1),
        &[crate::execute_test_support::ResponseItemFixture::success(
            None,
        )],
    );
    let response = decode_response_message(&bytes, &limits)
        .expect("the response fixture is a valid KMIP 2.1 message");

    let original_tree = response
        .with_ttlv(|message| message.try_clone())
        .expect("the original response tree can be cloned for byte comparison");
    let copied = clone_response_message(&response)
        .expect("profile validation makes an owned validated response copy");
    let copied_tree = copied
        .with_ttlv(|message| message.try_clone())
        .expect("the copied response tree remains valid and inspectable");

    assert_eq!(
        encode_message_for_test(original_tree, &limits)
            .expect("the original tree remains encodable"),
        encode_message_for_test(copied_tree, &limits).expect("the copied tree remains encodable"),
        "copying a response preserves its exact TTLV tree"
    );
}

#[test]
fn profile_validator_trait_forwards_standalone_request_and_response_checks() {
    let limits = CodecLimits::defaults();
    let profiles = ProfileValidationSelection::new(["KMIPKIT-PROFILE-BASELINE-CLIENT"])
        .validate()
        .expect("the Baseline profile selection has a complete typed registry");
    let request = request_message_for_test(
        ClientBatch::new(ClientBatchItem::new(ClientRequest::discover_versions())),
        &limits,
    )
    .expect("Discover Versions creates a valid request message");
    let response_bytes = crate::execute_test_support::response_bytes(
        (2, 1),
        &[crate::execute_test_support::ResponseItemFixture::success(
            None,
        )],
    );
    let response = decode_response_message(&response_bytes, &limits)
        .expect("the response fixture is a valid KMIP 2.1 message");
    let validator: &dyn ProfileExchangeValidator = &profiles;

    assert!(ProfileExchangeValidator::validate_request(validator, &request).is_ok());
    assert!(ProfileExchangeValidator::validate_response(validator, &request, &response).is_ok());
}
