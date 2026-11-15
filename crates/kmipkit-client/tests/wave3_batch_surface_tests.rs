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

#[test]
fn typed_kmip30_batch_outcome_and_response_readers_exist() {
    use kmipkit_client::{ClientBatchOutcome, Kmip30OperationResponse};
    use kmipkit_protocol::{
        CreateCredentialResponse, CreateGroupResponse, CreateUserResponse, DeactivateResponse,
        DecapsulateResponse, EncapsulateResponse, ObliterateResponse, UniqueIdentifier,
    };

    let group_resp = CreateGroupResponse::new(UniqueIdentifier::TextString("group-1".to_owned()));
    let outcome = ClientBatchOutcome::Kmip30(Kmip30OperationResponse::CreateGroup(group_resp));
    assert!(outcome.create_group_response().is_some());
    assert!(outcome.create_user_response().is_none());
    assert!(outcome.create_credential_response().is_none());
    assert!(outcome.obliterate_response().is_none());
    assert!(outcome.deactivate_response().is_none());
    assert!(outcome.encapsulate_response().is_none());
    assert!(outcome.decapsulate_response().is_none());

    let user_resp = CreateUserResponse::new(UniqueIdentifier::TextString("user-1".to_owned()));
    let user_outcome = ClientBatchOutcome::Kmip30(Kmip30OperationResponse::CreateUser(user_resp));
    assert!(user_outcome.create_user_response().is_some());

    let cred_resp =
        CreateCredentialResponse::new(UniqueIdentifier::TextString("cred-1".to_owned()));
    let cred_outcome =
        ClientBatchOutcome::Kmip30(Kmip30OperationResponse::CreateCredential(cred_resp));
    assert!(cred_outcome.create_credential_response().is_some());

    let deact_resp = DeactivateResponse::new(UniqueIdentifier::TextString("deact-1".to_owned()));
    let deact_outcome = ClientBatchOutcome::Kmip30(Kmip30OperationResponse::Deactivate(deact_resp));
    assert!(deact_outcome.deactivate_response().is_some());

    let oblit_outcome = ClientBatchOutcome::Kmip30(Kmip30OperationResponse::Obliterate(
        ObliterateResponse::new(),
    ));
    assert!(oblit_outcome.obliterate_response().is_some());
    assert!(oblit_outcome.create_group_response().is_none());

    let encap_outcome = ClientBatchOutcome::Kmip30(Kmip30OperationResponse::Encapsulate(
        EncapsulateResponse::new("pub-1", vec![1, 2, 3]),
    ));
    assert!(encap_outcome.encapsulate_response().is_some());
    assert_eq!(
        encap_outcome.encapsulate_response().unwrap().data(),
        Some(&[1, 2, 3][..])
    );

    let decap_outcome = ClientBatchOutcome::Kmip30(Kmip30OperationResponse::Decapsulate(
        DecapsulateResponse::new("priv-1"),
    ));
    assert!(decap_outcome.decapsulate_response().is_some());
}
