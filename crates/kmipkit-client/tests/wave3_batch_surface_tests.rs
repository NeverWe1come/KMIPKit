//! Wave 3 Red/Green contract tests for KMIP 2.1 async batch items and KMIP 3.0 typed batch items.

use kmipkit_client::{ClientBatchItem, ClientRequest};
use kmipkit_protocol::{
    AttributeSet, CancelRequest, CreateCredentialObject, CreateCredentialRequest,
    CreateGroupRequest, CreateUserRequest, CredentialType, DeactivateRequest, DecapsulateRequest,
    EncapsulateRequest, ObliterateRequest, PasswordCredentialObject,
    PasswordCredentialObjectFields, PollRequest, ProcessRequest, QueryAsyncRequestsRequest,
    SecretText, UniqueIdentifier,
};

#[test]
fn typed_kmip30_batch_item_constructors_exist_and_keep_table_584_operations() {
    let password = PasswordCredentialObject::try_new(PasswordCredentialObjectFields {
        password: Some(SecretText::new("secret-value".to_owned())),
        ..PasswordCredentialObjectFields::default()
    })
    .expect("valid password credential");

    for (item, expected_name) in [
        (
            ClientBatchItem::create_group(CreateGroupRequest::new(AttributeSet::new())),
            "CreateGroup",
        ),
        (
            ClientBatchItem::obliterate(ObliterateRequest::new(UniqueIdentifier::TextString(
                "uid-1".to_owned(),
            ))),
            "Obliterate",
        ),
        (
            ClientBatchItem::create_user(CreateUserRequest::new(AttributeSet::new())),
            "CreateUser",
        ),
        (
            ClientBatchItem::create_credential(CreateCredentialRequest::new(
                CredentialType::UsernameAndPassword,
                AttributeSet::new(),
                CreateCredentialObject::PasswordCredential(password),
            )),
            "CreateCredential",
        ),
        (
            ClientBatchItem::deactivate(DeactivateRequest::new("uid-1")),
            "Deactivate",
        ),
        (
            ClientBatchItem::encapsulate(EncapsulateRequest::new("pub-1")),
            "Encapsulate",
        ),
        (
            ClientBatchItem::decapsulate(DecapsulateRequest::new("priv-1", vec![1, 2, 3])),
            "Decapsulate",
        ),
    ] {
        assert!(item.request().validation_report().is_valid());
        assert!(format!("{:?}", item.request()).starts_with(expected_name));
    }
}

#[test]
fn async_operations_are_first_class_client_requests_and_batch_items() {
    let cancel = ClientBatchItem::cancel(CancelRequest::new(b"corr-1"));
    let poll = ClientBatchItem::poll(PollRequest::new(b"corr-2"));
    let process = ClientBatchItem::process(ProcessRequest::new(b"corr-3"));
    let query_async = ClientBatchItem::query_async_requests(QueryAsyncRequestsRequest::default());

    assert!(matches!(cancel.request(), ClientRequest::Cancel(_)));
    assert!(matches!(poll.request(), ClientRequest::Poll(_)));
    assert!(matches!(process.request(), ClientRequest::Process(_)));
    assert!(matches!(
        query_async.request(),
        ClientRequest::QueryAsynchronousRequests(_)
    ));

    let _: ClientRequest = ClientRequest::cancel(CancelRequest::new(b"corr-1"));
    let _: ClientRequest = ClientRequest::poll(PollRequest::new(b"corr-2"));
    let _: ClientRequest = ClientRequest::process(ProcessRequest::new(b"corr-3"));
    let _: ClientRequest =
        ClientRequest::query_async_requests(QueryAsyncRequestsRequest::default());
}
