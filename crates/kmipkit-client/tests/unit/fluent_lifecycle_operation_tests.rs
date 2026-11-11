//! Fluent lifecycle request tests derived from KMIP v2.1 §§6.1.1, 6.1.4,
//! 6.1.28, 6.1.42, and 6.1.44–6.1.47, plus KMIPKIT-0101.

use kmipkit_protocol::{RevocationReason, UniqueIdentifier};

use crate::asynchronous_execution_test_support::client_for;
use crate::execute::{ClientBatchItem, ClientOperation, ClientRequest};
use kmipkit_test_support::ExchangeScript;

fn assert_operation(item: ClientBatchItem, expected: ClientOperation) {
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
            .revoke()
            .reason(RevocationReason::from_raw(1))
            .unique_identifier(UniqueIdentifier::from("key-revoke"))
            .to_batch_item(),
        ClientOperation::Revoke,
    );

    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    assert_operation(
        client.request().locate().maximum_items(20).to_batch_item(),
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
