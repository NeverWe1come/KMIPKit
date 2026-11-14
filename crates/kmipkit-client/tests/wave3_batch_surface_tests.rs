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

    let items = [
        (
            ClientBatchItem::create_group(CreateGroupRequest::new(AttributeSet::new())),
            0x3C,
        ),
        (
            ClientBatchItem::obliterate(ObliterateRequest::new(UniqueIdentifier::TextString(
                "uid-1".to_owned(),
            ))),
            0x3D,
        ),
        (
            ClientBatchItem::create_user(CreateUserRequest::new(AttributeSet::new())),
            0x3E,
        ),
        (
            ClientBatchItem::create_credential(CreateCredentialRequest::new(
                CredentialType::UsernameAndPassword,
                AttributeSet::new(),
                CreateCredentialObject::PasswordCredential(password),
            )),
            0x3F,
        ),
        (
            ClientBatchItem::deactivate(DeactivateRequest::new("uid-1")),
            0x40,
        ),
        (
            ClientBatchItem::encapsulate(EncapsulateRequest::new("pub-1")),
            0x41,
        ),
        (
            ClientBatchItem::decapsulate(DecapsulateRequest::new("priv-1", vec![1, 2, 3])),
            0x42,
        ),
    ];

    for (item, expected_op) in items {
        assert_eq!(item.request().operation(), expected_op);
    }
}

#[test]
fn async_operations_are_first_class_client_requests_and_batch_items() {
    let cancel = ClientBatchItem::cancel(CancelRequest::new(b"corr-1"));
    let poll = ClientBatchItem::poll(PollRequest::new(b"corr-2"));
    let process = ClientBatchItem::process(ProcessRequest::new(b"corr-3"));
    let query_async = ClientBatchItem::query_async_requests(QueryAsyncRequestsRequest::default());

    assert_eq!(cancel.request().operation(), 0x23);
    assert_eq!(poll.request().operation(), 0x24);
    assert_eq!(process.request().operation(), 0x2B);
    assert_eq!(query_async.request().operation(), 0x2D);

    let _: ClientRequest = ClientRequest::cancel(CancelRequest::new(b"corr-1"));
    let _: ClientRequest = ClientRequest::poll(PollRequest::new(b"corr-2"));
    let _: ClientRequest = ClientRequest::process(ProcessRequest::new(b"corr-3"));
    let _: ClientRequest =
        ClientRequest::query_async_requests(QueryAsyncRequestsRequest::default());
}
