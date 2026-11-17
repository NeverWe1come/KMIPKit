//! Fluent Derive Key builder contracts from KMIP 2.1 §6.1.14 Tables 205–207
//! and §7.12 Table 364. These source-derived tests do not claim official test
//! case conformance.

use kmipkit_protocol::{
    AttributeSet, DerivationMethod, DerivationParameters, DeriveKeyRequest, ObjectType,
    SecretBytes, UniqueIdentifier, ValidateKmipRequest,
};
use kmipkit_test_support::ExchangeScript;
use kmipkit_ttlv::Value;
use kmipkit_ttlv::codec::CodecLimits;

use crate::asynchronous_execution_test_support::{client_for, request_contains};
use crate::execute::{
    ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientOperation, ClientRequest,
};
use crate::execute_test_support::{asynchronous_response_bytes, test_item, test_structure};
use crate::{DeriveKeyBuilder, FluentBuildError};

const DERIVE_KEY: u32 = 0x0000_0005;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const CRYPTOGRAPHIC_LENGTH: u32 = 0x0042_002a;
const CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_0028;
const DERIVATION_INPUT: &[u8] = b"FLUENT_DERIVE_KEY_SECRET_SENTINEL\x00\xff";
const PENDING_CORRELATION: &[u8] = b"DERIVE_KEY_PENDING\x00\xff";

fn output_attributes() -> AttributeSet {
    AttributeSet::try_new([
        test_item(CRYPTOGRAPHIC_LENGTH, Value::integer(256)),
        test_item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3)),
    ])
    .expect("the symmetric-key output attributes have valid TTLV types")
}

fn derivation_parameters() -> DerivationParameters {
    DerivationParameters::new()
        .with_derivation_data(SecretBytes::new(DERIVATION_INPUT.to_vec()))
        .with_salt(SecretBytes::new(b"caller-selected-salt".to_vec()))
        .with_iteration_count(120_000)
}

fn configured_builder<'a>(client: &'a mut crate::Client) -> DeriveKeyBuilder<'a> {
    client
        .request()
        .derive_key()
        .object_type(ObjectType::from_raw(2))
        .unique_identifier(UniqueIdentifier::from("source-secret"))
        .derivation_method(DerivationMethod::from_raw(1))
        .derivation_parameters(derivation_parameters())
        .attributes(output_attributes())
}

fn valid_request(client: &mut crate::Client) -> DeriveKeyRequest {
    configured_builder(client)
        .build()
        .expect("the caller supplied every required Derive Key field")
}

#[test]
fn fluent_derive_key_reports_missing_required_fields() {
    let (mut client, transport, _) = client_for(ExchangeScript::FailBeforeWrite);

    assert_eq!(
        client.request().derive_key().validate(),
        Err(FluentBuildError::MissingDeriveKeyObjectType)
    );
    assert_eq!(
        client
            .request()
            .derive_key()
            .object_type(ObjectType::from_raw(2))
            .validate(),
        Err(FluentBuildError::MissingDeriveKeyUniqueIdentifier)
    );
    assert_eq!(
        client
            .request()
            .derive_key()
            .object_type(ObjectType::from_raw(2))
            .unique_identifier(UniqueIdentifier::from("source-secret"))
            .validate(),
        Err(FluentBuildError::MissingDeriveKeyMethod)
    );
    assert_eq!(
        client
            .request()
            .derive_key()
            .object_type(ObjectType::from_raw(2))
            .unique_identifier(UniqueIdentifier::from("source-secret"))
            .derivation_method(DerivationMethod::from_raw(1))
            .validate(),
        Err(FluentBuildError::MissingDeriveKeyParameters)
    );
    assert_eq!(
        client
            .request()
            .derive_key()
            .object_type(ObjectType::from_raw(2))
            .unique_identifier(UniqueIdentifier::from("source-secret"))
            .derivation_method(DerivationMethod::from_raw(1))
            .derivation_parameters(derivation_parameters())
            .validate(),
        Err(FluentBuildError::MissingDeriveKeyAttributes)
    );
    assert_eq!(transport.borrow().exchange_count(), 0);
}

#[test]
fn fluent_derive_key_builds_typed_request_and_batch_without_sending() {
    let (mut client, transport, _) = client_for(ExchangeScript::FailBeforeWrite);
    let request = valid_request(&mut client);

    assert!(request.validate().is_valid());
    assert!(!format!("{request:?}").contains("FLUENT_DERIVE_KEY_SECRET_SENTINEL"));

    let item = configured_builder(&mut client)
        .to_batch_item()
        .expect("the builder produces the existing typed batch item");

    assert_eq!(item.operation(), DERIVE_KEY);
    assert_eq!(transport.borrow().exchange_count(), 0);
}

#[test]
fn fluent_derive_key_executes_once_and_preserves_server_result_and_secrets() {
    let response = asynchronous_response_bytes(
        DERIVE_KEY,
        0,
        None,
        None,
        Some(test_structure([test_item(
            UNIQUE_IDENTIFIER,
            Value::text_string("derived-key".to_owned()),
        )])),
    );
    let (mut client, transport, captured) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![3, 7],
    });

    let result = configured_builder(&mut client)
        .unique_identifier(UniqueIdentifier::from("second-source"))
        .execute(&CodecLimits::defaults())
        .expect("the existing typed Derive Key route returns its server result");

    assert_eq!(result.outcome().operation(), ClientOperation::DeriveKey);
    assert_eq!(result.outcome().result().status().raw(), 0);
    assert!(request_contains(&captured, DERIVATION_INPUT));
    assert!(request_contains(&captured, b"source-secret"));
    assert!(request_contains(&captured, b"second-source"));
    assert_eq!(transport.borrow().exchange_count(), 1);
}

#[test]
fn fluent_derive_key_pending_keeps_correlation_without_follow_up() {
    let response = asynchronous_response_bytes(
        DERIVE_KEY,
        2,
        None,
        Some(PENDING_CORRELATION),
        Some(test_structure([])),
    );
    let (mut client, transport, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::DeriveKey(
        valid_request(&mut client),
    )))
    .with_asynchronous_indicator(1);

    let response = client
        .execute(batch, &CodecLimits::defaults())
        .expect("the caller explicitly permits Pending");
    let outcome = response.get(0).expect("one result is associated").outcome();
    let ClientBatchOutcome::Pending(pending) = outcome else {
        panic!("Pending remains explicit and resumable by the caller");
    };

    assert_eq!(pending.operation(), ClientOperation::DeriveKey);
    assert_eq!(
        pending.asynchronous_correlation_value(),
        PENDING_CORRELATION
    );
    assert_eq!(transport.borrow().exchange_count(), 1);
}
