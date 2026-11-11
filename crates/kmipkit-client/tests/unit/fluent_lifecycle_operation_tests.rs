//! Fluent lifecycle request tests derived from KMIP v2.1 §§6.1.1, 6.1.4,
//! 6.1.28, 6.1.42, and 6.1.44–6.1.47, plus KMIPKIT-0101.

use kmipkit_protocol::{ProtocolVersion, RevocationReason, UniqueIdentifier};
use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::{RequestDeliveryState, RequestOptions};
use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::{Item, RawTag, Value};

use crate::asynchronous_execution_test_support::client_for;
use crate::execute::{ClientBatchItem, ClientBatchOutcome, ClientOperation, ClientRequest};
use crate::execute_test_support::{asynchronous_response_bytes, test_item, test_structure};
use crate::{ClientErrorCategory, FluentBuildError};

const REVOKE_OPERATION: u32 = 0x0000_0013;
const UNIQUE_IDENTIFIER_TAG: u32 = 0x0042_0094;
const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const OPERATION_PENDING: u32 = 2;
const OBJECT_NOT_FOUND: u32 = 0x0000_0037;
const PENDING_CORRELATION: &[u8] = b"LIFECYCLE_PENDING\x00\xff";

fn assert_operation(
    item: Result<ClientBatchItem, crate::fluent::FluentBuildError>,
    expected: ClientOperation,
) {
    let item = item.expect("the valid operation fixture builds locally");
    match item.request() {
        ClientRequest::Activate(_) => assert_eq!(expected, ClientOperation::Activate),
        ClientRequest::Archive(_) => assert_eq!(expected, ClientOperation::Archive),
        ClientRequest::Recover(_) => assert_eq!(expected, ClientOperation::Recover),
        ClientRequest::Revoke(_) => assert_eq!(expected, ClientOperation::Revoke),
        ClientRequest::Locate(_) => assert_eq!(expected, ClientOperation::Locate),
        ClientRequest::Rekey(_) => assert_eq!(expected, ClientOperation::Rekey),
        ClientRequest::RekeyKeyPair(_) => assert_eq!(expected, ClientOperation::RekeyKeyPair),
        ClientRequest::Recertify(_) => assert_eq!(expected, ClientOperation::Recertify),
        _ => panic!("builder constructed a different typed request variant"),
    }
}

#[test]
fn lifecycle_constructors_build_their_typed_request_variants() {
    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    assert_operation(
        client
            .request()
            .activate()
            .unique_identifier(UniqueIdentifier::from("key-activate"))
            .to_batch_item(),
        ClientOperation::Activate,
    );

    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    assert_operation(
        client
            .request()
            .archive()
            .unique_identifier(UniqueIdentifier::from("key-archive"))
            .to_batch_item(),
        ClientOperation::Archive,
    );

    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    assert_operation(
        client
            .request()
            .recover()
            .unique_identifier(UniqueIdentifier::from("key-recover"))
            .to_batch_item(),
        ClientOperation::Recover,
    );

    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    assert_operation(
        client
            .request()
            .revoke(RevocationReason::from_raw(1))
            .unique_identifier(UniqueIdentifier::from("key-revoke"))
            .to_batch_item(),
        ClientOperation::Revoke,
    );

    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    assert_operation(
        client
            .request()
            .locate(kmipkit_protocol::AttributeSet::new())
            .maximum_items(20)
            .to_batch_item(),
        ClientOperation::Locate,
    );

    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    assert_operation(
        client
            .request()
            .rekey()
            .unique_identifier(UniqueIdentifier::from("key-rekey"))
            .offset(3600)
            .to_batch_item(),
        ClientOperation::Rekey,
    );

    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    assert_operation(
        client
            .request()
            .rekey_key_pair()
            .private_key_unique_identifier(UniqueIdentifier::from("private-rekey"))
            .to_batch_item(),
        ClientOperation::RekeyKeyPair,
    );

    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    assert_operation(
        client
            .request()
            .recertify()
            .unique_identifier(UniqueIdentifier::from("certificate-recertify"))
            .to_batch_item(),
        ClientOperation::Recertify,
    );
}

#[test]
fn lifecycle_builders_preserve_optional_identifiers_and_activation_identifier() {
    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    let request = client
        .request()
        .activate()
        .build_request()
        .expect("the typed Activate model permits an omitted identifier");
    assert_eq!(request.unique_identifier(), None);

    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    let request = client
        .request()
        .rekey()
        .build_request()
        .expect("the typed Re-key model permits an omitted identifier");
    assert_eq!(request.unique_identifier(), None);

    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    let request = client
        .request()
        .rekey_key_pair()
        .build_request()
        .expect("the typed Re-key Key Pair model permits an omitted identifier");
    assert_eq!(request.private_key_unique_identifier(), None);

    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    let activation_id = UniqueIdentifier::from("activation-id");
    let item = client
        .request()
        .activate()
        .unique_identifier(activation_id.clone())
        .to_batch_item()
        .expect("Activate converts locally");
    let ClientRequest::Activate(request) = item.request() else {
        panic!("the request remains Activate");
    };
    assert_eq!(request.unique_identifier(), Some(&activation_id));
}

#[test]
fn revoke_builder_preserves_semantic_revocation_fields() {
    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    let revoke_id = UniqueIdentifier::from("revoke-id");
    let item = client
        .request()
        .revoke(RevocationReason::KEY_COMPROMISE)
        .unique_identifier(revoke_id.clone())
        .revocation_message("operator supplied message")
        .compromise_occurrence_date(1_700_000_000)
        .to_batch_item()
        .expect("valid Revoke converts locally");
    let ClientRequest::Revoke(request) = item.request() else {
        panic!("the request remains Revoke");
    };
    assert_eq!(request.unique_identifier(), Some(&revoke_id));
    assert_eq!(
        request.revocation_reason(),
        RevocationReason::KEY_COMPROMISE
    );
    assert_eq!(
        request.revocation_message(),
        Some("operator supplied message")
    );
    assert_eq!(request.compromise_occurrence_date(), Some(1_700_000_000));
}

#[test]
fn locate_builder_preserves_explicit_attributes_and_filters() {
    let comment_tag = RawTag::new(0x0042_00FD)
        .expect("Comment has an assigned KMIP tag")
        .try_checked()
        .expect("Comment tag is valid");
    let attributes = kmipkit_protocol::AttributeSet::try_new([Item::new(
        comment_tag,
        Value::text_string("ledger".to_owned()),
    )
    .expect("Comment attribute is a valid TTLV Item")])
    .expect("Comment is accepted as a Locate criterion");
    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    let item = client
        .request()
        .locate(attributes)
        .maximum_items(20)
        .offset_items(3)
        .storage_status_mask(kmipkit_protocol::StorageStatusMask::from_raw(5))
        .object_group_member(kmipkit_protocol::ObjectGroupMember::from_raw(9))
        .to_batch_item()
        .expect("Locate filters convert locally");
    let ClientRequest::Locate(request) = item.request() else {
        panic!("the request remains Locate");
    };
    assert_eq!(request.maximum_items(), Some(20));
    assert_eq!(request.offset_items(), Some(3));
    assert_eq!(request.attributes().len(), 1);
    assert_eq!(request.attributes().as_items()[0].tag().raw(), 0x0042_00FD);
    assert_eq!(
        request
            .storage_status_mask()
            .map(kmipkit_protocol::StorageStatusMask::raw),
        Some(5)
    );
    assert_eq!(
        request
            .object_group_member()
            .map(kmipkit_protocol::ObjectGroupMember::raw),
        Some(9)
    );
}

#[test]
fn rekey_builder_preserves_identifier_and_emitted_offset() {
    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    let rekey_id = UniqueIdentifier::from("rekey-id");
    let request = client
        .request()
        .rekey()
        .unique_identifier(rekey_id.clone())
        .offset(3600)
        .build_request()
        .expect("Re-key builds locally");
    assert_eq!(request.unique_identifier(), Some(&rekey_id));
    let payload = request
        .into_ttlv_payload()
        .expect("the typed Re-key payload is valid");
    assert!(
        payload
            .view()
            .children()
            .iter()
            .any(|item| item.tag().raw() == 0x0042_0058)
    );
}

#[test]
fn rekey_key_pair_keeps_common_private_and_public_attribute_groups_separate() {
    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    let request = client
        .request()
        .rekey_key_pair()
        .common_attributes(kmipkit_protocol::AttributeSet::new())
        .private_key_attributes(kmipkit_protocol::AttributeSet::new())
        .public_key_attributes(kmipkit_protocol::AttributeSet::new())
        .build_request()
        .expect("the three explicit attribute groups are accepted");
    let payload = request
        .into_ttlv_payload()
        .expect("the existing Re-key Key Pair model serializes the groups");
    let tags = payload
        .view()
        .children()
        .iter()
        .map(|item| item.tag().raw())
        .collect::<Vec<_>>();
    assert_eq!(tags, [0x0042_0126, 0x0042_0127, 0x0042_0128]);
}

#[test]
fn recertify_requires_a_source_identifier_or_certificate_request_value() {
    let (mut client, fake, _) = client_for(ExchangeScript::FailBeforeWrite);
    let error = client
        .request()
        .recertify()
        .validate()
        .expect_err("the existing Recertify validator requires a source input");
    assert_eq!(error, FluentBuildError::InvalidRequest);
    assert_eq!(fake.borrow().exchange_count(), 0);
}

#[test]
fn replacement_operation_aliases_keep_the_typed_builder_contract() {
    let (mut rekey_client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    let rekey_builder: crate::RekeyBuilder<'_> = rekey_client.request().rekey();
    let rekey_identifier = UniqueIdentifier::from("alias-rekey-id");
    let rekey = rekey_builder
        .unique_identifier(rekey_identifier.clone())
        .offset(60)
        .build_request()
        .expect("the RekeyBuilder alias retains its concrete typed request");
    assert_eq!(rekey.unique_identifier(), Some(&rekey_identifier));

    let (mut recertify_client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    let recertify_builder: crate::RecertifyBuilder<'_> = recertify_client.request().recertify();
    let certificate_id = UniqueIdentifier::from("alias-certificate-id");
    let recertify = recertify_builder
        .unique_identifier(certificate_id.clone())
        .certificate_request_type(1)
        .offset(120)
        .build_request()
        .expect("the RecertifyBuilder alias retains its operation-specific setters");
    assert_eq!(recertify.unique_identifier(), Some(&certificate_id));
}

#[test]
fn revoke_invalid_compromise_date_is_rejected_before_the_transport_exchange() {
    let (mut client, fake, _) = client_for(ExchangeScript::FailBeforeWrite);
    let error = client
        .request()
        .revoke(RevocationReason::UNSPECIFIED)
        .unique_identifier(UniqueIdentifier::from("key-revoke"))
        .compromise_occurrence_date(1_700_000_000)
        .execute(&CodecLimits::defaults())
        .expect_err("the typed Revoke validator rejects an unrelated compromise date");
    assert_eq!(error.category(), ClientErrorCategory::Validation);
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(fake.borrow().exchange_count(), 0);
}

#[test]
fn unsupported_protocol_version_is_rejected_before_the_transport_exchange() {
    let (mut client, fake, _) = client_for(ExchangeScript::FailBeforeWrite);
    let error = client
        .request()
        .revoke(RevocationReason::UNSPECIFIED)
        .unique_identifier(UniqueIdentifier::from("key-revoke"))
        .with_options(
            RequestOptions::default().with_protocol_version(ProtocolVersion::from_raw(1, 4)),
        )
        .execute(&CodecLimits::defaults())
        .expect_err("unsupported KMIP v1.x stays behind the existing client version gate");
    assert_eq!(error.category(), ClientErrorCategory::Validation);
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(fake.borrow().exchange_count(), 0);
}

#[test]
fn revoke_execute_exposes_success_failure_and_pending_without_follow_up() {
    let success_response = asynchronous_response_bytes(
        REVOKE_OPERATION,
        SUCCESS,
        None,
        None,
        Some(test_structure([test_item(
            UNIQUE_IDENTIFIER_TAG,
            Value::text_string("revoked-key".to_owned()),
        )])),
    );
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response: success_response,
        request_write_chunks: Vec::new(),
    });
    let response = client
        .request()
        .revoke(RevocationReason::KEY_COMPROMISE)
        .unique_identifier(UniqueIdentifier::from("key"))
        .execute(&CodecLimits::defaults())
        .expect("Success is returned through the existing typed path");
    let ClientBatchOutcome::Revoke(revoke) = response.outcome() else {
        panic!("Success remains the typed Revoke outcome");
    };
    assert_eq!(
        revoke.unique_identifier(),
        Some(&UniqueIdentifier::from("revoked-key"))
    );
    assert_eq!(revoke.result().status().raw(), SUCCESS);
    assert_eq!(fake.borrow().exchange_count(), 1);

    let failed_response = asynchronous_response_bytes(
        REVOKE_OPERATION,
        OPERATION_FAILED,
        Some(OBJECT_NOT_FOUND),
        None,
        None,
    );
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response: failed_response,
        request_write_chunks: Vec::new(),
    });
    let response = client
        .request()
        .revoke(RevocationReason::UNSPECIFIED)
        .unique_identifier(UniqueIdentifier::from("missing-key"))
        .execute(&CodecLimits::defaults())
        .expect("KMIP Failure remains an operation result, not a local error");
    assert_eq!(response.outcome().result().status().raw(), OPERATION_FAILED);
    assert_eq!(
        response
            .outcome()
            .result()
            .reason()
            .map(kmipkit_protocol::ResultReason::raw),
        Some(OBJECT_NOT_FOUND)
    );
    assert_eq!(fake.borrow().exchange_count(), 1);

    let pending_response = asynchronous_response_bytes(
        REVOKE_OPERATION,
        OPERATION_PENDING,
        None,
        Some(PENDING_CORRELATION),
        Some(test_structure([])),
    );
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response: pending_response,
        request_write_chunks: Vec::new(),
    });
    let response = client
        .request()
        .revoke(RevocationReason::KEY_COMPROMISE)
        .unique_identifier(UniqueIdentifier::from("pending-key"))
        .with_options(RequestOptions::default().with_protocol_version(ProtocolVersion::V2_1))
        .with_asynchronous_indicator(1)
        .execute(&CodecLimits::defaults())
        .expect("Pending remains visible without an implicit Poll");
    let ClientBatchOutcome::Pending(pending) = response.outcome() else {
        panic!("the server's Pending result is preserved");
    };
    assert_eq!(
        pending.asynchronous_correlation_value(),
        PENDING_CORRELATION
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn recertify_request_debug_redacts_certificate_request_material() {
    let sentinel = b"FLUENT_RECERTIFY_SECRET_SENTINEL";
    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    let item = client
        .request()
        .recertify()
        .certificate_request_type(1)
        .certificate_request_value(kmipkit_protocol::SecretBytes::new(sentinel.to_vec()))
        .to_batch_item()
        .expect("certificate request material can be built with its type");
    let ClientRequest::Recertify(request) = item.request() else {
        panic!("the request remains Re-certify");
    };
    assert!(!format!("{request:?}").contains(std::str::from_utf8(sentinel).unwrap()));
}
