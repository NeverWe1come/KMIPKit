//! Fluent cryptographic request builders from KMIPKIT-0102 and KMIP v2.1.

use kmipkit_protocol::UniqueIdentifier;

use crate::asynchronous_execution_test_support::client_for;
use crate::execute::{ClientBatchItem, ClientOperation, ClientRequest};
use crate::execute_test_support::test_structure;
use crate::FluentBuildError;
use kmipkit_test_support::ExchangeScript;

fn assert_operation(
    item: Result<ClientBatchItem, FluentBuildError>,
    expected: ClientOperation,
) {
    let item = item.expect("a valid cryptographic builder creates a typed batch item");
    match item.request() {
        ClientRequest::Encrypt(_) => assert_eq!(expected, ClientOperation::Encrypt),
        ClientRequest::Decrypt(_) => assert_eq!(expected, ClientOperation::Decrypt),
        ClientRequest::Sign(_) => assert_eq!(expected, ClientOperation::Sign),
        ClientRequest::SignatureVerify(_) => {
            assert_eq!(expected, ClientOperation::SignatureVerify);
        }
        ClientRequest::Mac(_) => assert_eq!(expected, ClientOperation::Mac),
        ClientRequest::MacVerify(_) => assert_eq!(expected, ClientOperation::MacVerify),
        ClientRequest::Hash(_) => assert_eq!(expected, ClientOperation::Hash),
        _ => panic!("builder constructed a different typed request variant"),
    }
}

fn typed_parameters() -> kmipkit_protocol::CryptographicParameters {
    kmipkit_protocol::CryptographicParametersBuilder::new()
        .with_cryptographic_algorithm(
            kmipkit_protocol::named_values_generated::values::enumeration::cryptographic_algorithm::AES,
        )
        .build()
        .expect("AES parameters are a valid caller-selected structure")
}

#[test]
fn cryptographic_constructors_create_the_existing_typed_request_variants() {
    let parameters = typed_parameters();
    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    assert_operation(
        client
            .request()
            .encrypt()
            .unique_identifier(UniqueIdentifier::from("encrypt-key"))
            .data(b"cleartext".to_vec())
            .cryptographic_parameters(parameters)
            .to_batch_item(),
        ClientOperation::Encrypt,
    );

    let parameters = typed_parameters();
    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    assert_operation(
        client
            .request()
            .decrypt()
            .unique_identifier(UniqueIdentifier::from("decrypt-key"))
            .data(b"ciphertext".to_vec())
            .cryptographic_parameters(parameters)
            .to_batch_item(),
        ClientOperation::Decrypt,
    );

    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    assert_operation(
        client
            .request()
            .sign()
            .unique_identifier(UniqueIdentifier::from("sign-key"))
            .data(b"message".to_vec())
            .to_batch_item(),
        ClientOperation::Sign,
    );

    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    assert_operation(
        client
            .request()
            .signature_verify()
            .unique_identifier(UniqueIdentifier::from("verify-key"))
            .data(b"message".to_vec())
            .signature_data(b"signature".to_vec())
            .to_batch_item(),
        ClientOperation::SignatureVerify,
    );

    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    assert_operation(
        client
            .request()
            .mac()
            .unique_identifier(UniqueIdentifier::from("mac-key"))
            .data(b"message".to_vec())
            .to_batch_item(),
        ClientOperation::Mac,
    );

    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    assert_operation(
        client
            .request()
            .mac_verify()
            .unique_identifier(UniqueIdentifier::from("mac-verify-key"))
            .data(b"message".to_vec())
            .mac_data(b"mac".to_vec())
            .to_batch_item(),
        ClientOperation::MacVerify,
    );

    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    assert_operation(
        client
            .request()
            .hash()
            .cryptographic_parameters(typed_parameters())
            .data(b"message".to_vec())
            .to_batch_item(),
        ClientOperation::Hash,
    );
}

#[test]
fn cryptographic_builders_preserve_explicit_ttlv_expert_parameters() {
    let parameter_tag = kmipkit_ttlv::RawTag::new(0x0042_00AA)
        .expect("the parameter tag has a nonzero KMIP identifier")
        .try_checked()
        .expect("the expert tag is a valid assigned KMIP tag");
    let parameters = test_structure([kmipkit_ttlv::Item::new(
        parameter_tag,
        kmipkit_ttlv::Value::integer(7),
    )
    .expect("the expert parameter is structurally valid")]);
    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    let item = client
        .request()
        .hash()
        .cryptographic_parameters_structure(parameters)
        .data(b"message".to_vec())
        .to_batch_item()
        .expect("the expert Structure passes through typed Hash validation");
    let ClientRequest::Hash(request) = item.request() else {
        panic!("the request remains Hash");
    };
    let payload = request
        .to_ttlv_payload()
        .expect("the typed Hash model builds the payload");
    assert_eq!(payload.view().children()[0].tag().raw(), 0x0042_002B);
}
