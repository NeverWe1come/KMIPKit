//! Unit contract tests for KMIP 2.1 async batch items and KMIP 3.0 typed batch items.

use kmipkit_protocol::{
    AttributeSet, CancelRequest, CancellationResult, CreateCredentialObject,
    CreateCredentialRequest, CreateGroupRequest, CreateUserRequest, CredentialType,
    DeactivateRequest, DecapsulateRequest, EncapsulateRequest, ObliterateRequest,
    PasswordCredentialObject, PasswordCredentialObjectFields, PollRequest, ProcessRequest,
    QueryAsyncRequestsRequest, SecretText, UniqueIdentifier,
};
use kmipkit_test_support::ExchangeScript;
use kmipkit_ttlv::Value;
use kmipkit_ttlv::codec::CodecLimits;

use crate::asynchronous_execution_test_support::client_for;
use crate::execute_test_support::{
    asynchronous_response_bytes, asynchronous_response_bytes_with_version, test_item,
    test_structure,
};
use crate::{
    ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientOperation, ClientRequest,
    RequestOptions,
};

const CANCEL: u32 = 0x0000_0019;
const POLL: u32 = 0x0000_001A;
const QUERY_ASYNC: u32 = 0x0000_0039;
const PROCESS: u32 = 0x0000_003A;

const ASYNC_CORRELATION_TAG: u32 = 0x0042_0006;
const CANCELLATION_RESULT_TAG: u32 = 0x0042_0012;
const ASYNCHRONOUS_CORRELATION_VALUES: u32 = 0x0042_0176;

#[test]
fn async_requests_methods_to_payload_and_debug() {
    let cancel = ClientRequest::cancel(CancelRequest::new(b"corr-1"));
    let poll = ClientRequest::poll(PollRequest::new(b"corr-2"));
    let process = ClientRequest::process(ProcessRequest::new(b"corr-3"));
    let query_async = ClientRequest::query_async_requests(QueryAsyncRequestsRequest::default());

    assert_eq!(cancel.operation(), CANCEL);
    assert_eq!(poll.operation(), POLL);
    assert_eq!(process.operation(), PROCESS);
    assert_eq!(query_async.operation(), QUERY_ASYNC);

    assert!(format!("{cancel:?}").contains("Cancel([REDACTED])"));
    assert!(format!("{poll:?}").contains("Poll([REDACTED])"));
    assert!(format!("{process:?}").contains("Process([REDACTED])"));
    assert!(format!("{query_async:?}").contains("QueryAsynchronousRequests([REDACTED])"));

    assert!(cancel.payload().is_ok());
    assert!(poll.payload().is_ok());
    assert!(process.payload().is_ok());
    assert!(query_async.payload().is_ok());

    let password = PasswordCredentialObject::try_new(PasswordCredentialObjectFields {
        password: Some(SecretText::new("secret-value".to_owned())),
        ..PasswordCredentialObjectFields::default()
    })
    .expect("valid password credential");

    let kmip30_requests = [
        ClientRequest::CreateGroup(CreateGroupRequest::new(AttributeSet::new())),
        ClientRequest::Obliterate(ObliterateRequest::new(UniqueIdentifier::TextString(
            "uid-1".to_owned(),
        ))),
        ClientRequest::CreateUser(CreateUserRequest::new(AttributeSet::new())),
        ClientRequest::CreateCredential(CreateCredentialRequest::new(
            CredentialType::UsernameAndPassword,
            AttributeSet::new(),
            CreateCredentialObject::PasswordCredential(password),
        )),
        ClientRequest::Deactivate(DeactivateRequest::new("uid-1")),
        ClientRequest::Encapsulate(EncapsulateRequest::new("pub-1")),
        ClientRequest::Decapsulate(DecapsulateRequest::new("priv-1", vec![1, 2, 3])),
    ];

    for req in kmip30_requests {
        assert!(req.payload().is_ok());
    }
}

#[test]
fn async_batch_item_cancel_roundtrip() {
    let payload = test_structure([
        test_item(
            ASYNC_CORRELATION_TAG,
            Value::byte_string(b"corr-1".to_vec()),
        ),
        test_item(CANCELLATION_RESULT_TAG, Value::enumeration(3)),
    ]);
    let response = asynchronous_response_bytes(CANCEL, 0, None, None, Some(payload));
    let (mut client, _, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });

    let batch = ClientBatch::new(ClientBatchItem::cancel(CancelRequest::new(b"corr-1")));
    let outcome = client
        .execute_with_options(batch, &CodecLimits::defaults(), &RequestOptions::default())
        .expect("successful Cancel execution");

    let item = outcome.get(0).expect("one response item");
    assert_eq!(item.outcome().operation(), ClientOperation::Cancel);
    let cancel_resp = item.cancel_response().expect("typed Cancel response");
    assert_eq!(
        cancel_resp.cancellation_result(),
        Some(CancellationResult::Completed)
    );

    assert!(item.poll_response().is_none());
    assert!(item.process_response().is_none());
    assert!(item.query_async_requests_response().is_none());
}

#[test]
fn async_batch_item_poll_success_and_pending() {
    let payload = test_structure([test_item(
        0x0042_0012,
        Value::byte_string(b"opaque".to_vec()),
    )]);
    let response = asynchronous_response_bytes(POLL, 0, None, None, Some(payload));
    let (mut client, _, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });

    let batch = ClientBatch::new(ClientBatchItem::poll(PollRequest::new(b"corr-2")));
    let outcome = client
        .execute_with_options(batch, &CodecLimits::defaults(), &RequestOptions::default())
        .expect("successful Poll execution");

    let item = outcome.get(0).expect("one response item");
    assert_eq!(item.outcome().operation(), ClientOperation::Poll);
    assert!(item.poll_response().is_some());
    assert!(item.cancel_response().is_none());

    // Test pending outcome (status 2, no payload for Poll)
    let pending_resp = asynchronous_response_bytes(POLL, 2, None, Some(b"corr-2"), None);
    let (mut client_pending, _, _) = client_for(ExchangeScript::Success {
        response: pending_resp,
        request_write_chunks: Vec::new(),
    });
    let batch_pending = ClientBatch::new(ClientBatchItem::poll(PollRequest::new(b"corr-2")))
        .with_asynchronous_indicator(2);
    let outcome_pending = client_pending
        .execute_with_options(
            batch_pending,
            &CodecLimits::defaults(),
            &RequestOptions::default(),
        )
        .expect("successful Poll pending response");
    assert!(matches!(
        outcome_pending.get(0).expect("item").outcome(),
        ClientBatchOutcome::Pending(_)
    ));
}

#[test]
fn async_batch_item_process_success_and_pending() {
    // Process success has empty payload
    let response = asynchronous_response_bytes(PROCESS, 0, None, None, Some(test_structure([])));
    let (mut client, _, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });

    let batch = ClientBatch::new(ClientBatchItem::process(ProcessRequest::new(b"corr-3")));
    let outcome = client
        .execute_with_options(batch, &CodecLimits::defaults(), &RequestOptions::default())
        .expect("successful Process execution");

    let item = outcome.get(0).expect("one response item");
    assert_eq!(item.outcome().operation(), ClientOperation::Process);
    assert!(item.process_response().is_some());
    assert!(item.cancel_response().is_none());

    // Test pending outcome (status 2, empty payload)
    let pending_resp =
        asynchronous_response_bytes(PROCESS, 2, None, Some(b"corr-3"), Some(test_structure([])));
    let (mut client_pending, _, _) = client_for(ExchangeScript::Success {
        response: pending_resp,
        request_write_chunks: Vec::new(),
    });
    let batch_pending = ClientBatch::new(ClientBatchItem::process(ProcessRequest::new(b"corr-3")))
        .with_asynchronous_indicator(2);
    let outcome_pending = client_pending
        .execute_with_options(
            batch_pending,
            &CodecLimits::defaults(),
            &RequestOptions::default(),
        )
        .expect("successful Process pending response");
    assert!(matches!(
        outcome_pending.get(0).expect("item").outcome(),
        ClientBatchOutcome::Pending(_)
    ));
}

#[test]
fn async_batch_item_query_async_success_and_pending() {
    let payload = test_structure([test_item(
        ASYNCHRONOUS_CORRELATION_VALUES,
        Value::structure(test_structure([test_item(
            0x0042_0006,
            Value::byte_string(b"corr-4".to_vec()),
        )])),
    )]);
    let response = asynchronous_response_bytes(QUERY_ASYNC, 0, None, None, Some(payload));
    let (mut client, _, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });

    let batch = ClientBatch::new(ClientBatchItem::query_async_requests(
        QueryAsyncRequestsRequest::default(),
    ));
    let outcome = client
        .execute_with_options(batch, &CodecLimits::defaults(), &RequestOptions::default())
        .expect("successful QueryAsync execution");

    let item = outcome.get(0).expect("one response item");
    assert_eq!(
        item.outcome().operation(),
        ClientOperation::QueryAsyncRequests
    );
    assert!(item.query_async_requests_response().is_some());
    assert!(item.cancel_response().is_none());

    // Test pending outcome (status 2)
    let pending_resp = asynchronous_response_bytes(
        QUERY_ASYNC,
        2,
        None,
        Some(b"corr-4"),
        Some(test_structure([])),
    );
    let (mut client_pending, _, _) = client_for(ExchangeScript::Success {
        response: pending_resp,
        request_write_chunks: Vec::new(),
    });
    let batch_pending = ClientBatch::new(ClientBatchItem::query_async_requests(
        QueryAsyncRequestsRequest::default(),
    ))
    .with_asynchronous_indicator(2);
    let outcome_pending = client_pending
        .execute_with_options(
            batch_pending,
            &CodecLimits::defaults(),
            &RequestOptions::default(),
        )
        .expect("successful QueryAsync pending response");
    assert!(matches!(
        outcome_pending.get(0).expect("item").outcome(),
        ClientBatchOutcome::Pending(_)
    ));
}

#[test]
fn kmip30_batch_items_constructors_and_draft_constructors() {
    let password = PasswordCredentialObject::try_new(PasswordCredentialObjectFields {
        password: Some(SecretText::new("secret-value".to_owned())),
        ..PasswordCredentialObjectFields::default()
    })
    .expect("valid password credential");

    let _items = [
        ClientBatchItem::create_group(CreateGroupRequest::new(AttributeSet::new())),
        ClientBatchItem::obliterate(ObliterateRequest::new(UniqueIdentifier::TextString(
            "uid-1".to_owned(),
        ))),
        ClientBatchItem::create_user(CreateUserRequest::new(AttributeSet::new())),
        ClientBatchItem::create_credential(CreateCredentialRequest::new(
            CredentialType::UsernameAndPassword,
            AttributeSet::new(),
            CreateCredentialObject::PasswordCredential(password),
        )),
        ClientBatchItem::deactivate(DeactivateRequest::new("uid-1")),
        ClientBatchItem::encapsulate(EncapsulateRequest::new("pub-1")),
        ClientBatchItem::decapsulate(DecapsulateRequest::new("priv-1", vec![1, 2, 3])),
        ClientBatchItem::create_group_draft(test_structure([])),
        ClientBatchItem::obliterate_draft(test_structure([])),
        ClientBatchItem::create_user_draft(test_structure([])),
        ClientBatchItem::create_credential_draft(test_structure([])),
        ClientBatchItem::deactivate_draft(test_structure([])),
        ClientBatchItem::encapsulate_draft(test_structure([])),
        ClientBatchItem::decapsulate_draft(test_structure([])),
    ];
}

#[test]
fn client_kmip30_group_and_user_methods_execute_roundtrip() {
    use kmipkit_protocol::ProtocolVersion;

    const CREATE_GROUP_OPERATION: u32 = 0x0000_003C;
    const OBLITERATE_OPERATION: u32 = 0x0000_003D;
    const CREATE_USER_OPERATION: u32 = 0x0000_003E;

    let v3_options = RequestOptions::default().with_protocol_version(ProtocolVersion::V3_0);
    let uid_payload = || {
        test_structure([test_item(
            0x0042_0094,
            Value::text_string("obj-1".to_string()),
        )])
    };

    // Default KMIP 2.1 client rejects 3.0 operations as NotSent invalid input
    let (mut client, _, _) = client_for(ExchangeScript::Success {
        response: asynchronous_response_bytes(
            CREATE_GROUP_OPERATION,
            0,
            None,
            None,
            Some(uid_payload()),
        ),
        request_write_chunks: Vec::new(),
    });
    assert!(
        client
            .create_group(
                CreateGroupRequest::new(AttributeSet::new()),
                &CodecLimits::defaults()
            )
            .is_err()
    );

    // When KMIP 3.0 is specified, they execute successfully
    let (mut client, _, _) = client_for(ExchangeScript::Success {
        response: asynchronous_response_bytes_with_version(
            ProtocolVersion::V3_0,
            CREATE_GROUP_OPERATION,
            0,
            None,
            None,
            Some(uid_payload()),
        ),
        request_write_chunks: Vec::new(),
    });
    assert!(
        client
            .create_group_with_options(
                CreateGroupRequest::new(AttributeSet::new()),
                &CodecLimits::defaults(),
                &v3_options
            )
            .is_ok()
    );

    let (mut client, _, _) = client_for(ExchangeScript::Success {
        response: asynchronous_response_bytes_with_version(
            ProtocolVersion::V3_0,
            OBLITERATE_OPERATION,
            0,
            None,
            None,
            Some(test_structure([])),
        ),
        request_write_chunks: Vec::new(),
    });
    assert!(
        client
            .obliterate_with_options(
                ObliterateRequest::new(UniqueIdentifier::TextString("uid-1".to_owned())),
                &CodecLimits::defaults(),
                &v3_options
            )
            .is_ok()
    );

    let (mut client, _, _) = client_for(ExchangeScript::Success {
        response: asynchronous_response_bytes_with_version(
            ProtocolVersion::V3_0,
            CREATE_USER_OPERATION,
            0,
            None,
            None,
            Some(uid_payload()),
        ),
        request_write_chunks: Vec::new(),
    });
    assert!(
        client
            .create_user_with_options(
                CreateUserRequest::new(AttributeSet::new()),
                &CodecLimits::defaults(),
                &v3_options
            )
            .is_ok()
    );
}

#[test]
fn client_kmip30_credential_and_deactivate_methods_execute_roundtrip() {
    use kmipkit_protocol::ProtocolVersion;

    const CREATE_CREDENTIAL_OPERATION: u32 = 0x0000_003F;
    const DEACTIVATE_OPERATION: u32 = 0x0000_0040;

    let v3_options = RequestOptions::default().with_protocol_version(ProtocolVersion::V3_0);
    let uid_payload = || {
        test_structure([test_item(
            0x0042_0094,
            Value::text_string("obj-1".to_string()),
        )])
    };

    let password = PasswordCredentialObject::try_new(PasswordCredentialObjectFields {
        password: Some(SecretText::new("secret-value".to_owned())),
        ..PasswordCredentialObjectFields::default()
    })
    .expect("valid password credential");

    let (mut client, _, _) = client_for(ExchangeScript::Success {
        response: asynchronous_response_bytes_with_version(
            ProtocolVersion::V3_0,
            CREATE_CREDENTIAL_OPERATION,
            0,
            None,
            None,
            Some(uid_payload()),
        ),
        request_write_chunks: Vec::new(),
    });
    assert!(
        client
            .create_credential_with_options(
                CreateCredentialRequest::new(
                    CredentialType::UsernameAndPassword,
                    AttributeSet::new(),
                    CreateCredentialObject::PasswordCredential(password),
                ),
                &CodecLimits::defaults(),
                &v3_options
            )
            .is_ok()
    );

    let (mut client, _, _) = client_for(ExchangeScript::Success {
        response: asynchronous_response_bytes_with_version(
            ProtocolVersion::V3_0,
            DEACTIVATE_OPERATION,
            0,
            None,
            None,
            Some(uid_payload()),
        ),
        request_write_chunks: Vec::new(),
    });
    assert!(
        client
            .deactivate_with_options(
                DeactivateRequest::new("uid-1"),
                &CodecLimits::defaults(),
                &v3_options
            )
            .is_ok()
    );
}

#[test]
fn client_kmip30_kem_methods_execute_roundtrip() {
    use kmipkit_protocol::ProtocolVersion;

    const ENCAPSULATE_OPERATION: u32 = 0x0000_0041;
    const DECAPSULATE_OPERATION: u32 = 0x0000_0042;

    let v3_options = RequestOptions::default().with_protocol_version(ProtocolVersion::V3_0);
    let uid_payload = || {
        test_structure([test_item(
            0x0042_0094,
            Value::text_string("obj-1".to_string()),
        )])
    };

    let encap_payload = test_structure([
        test_item(0x0042_0094, Value::text_string("obj-1".to_string())),
        test_item(0x0042_00C2, Value::byte_string(vec![1, 2, 3])),
    ]);
    let (mut client, _, _) = client_for(ExchangeScript::Success {
        response: asynchronous_response_bytes_with_version(
            ProtocolVersion::V3_0,
            ENCAPSULATE_OPERATION,
            0,
            None,
            None,
            Some(encap_payload),
        ),
        request_write_chunks: Vec::new(),
    });
    assert!(
        client
            .encapsulate_with_options(
                EncapsulateRequest::new("pub-1"),
                &CodecLimits::defaults(),
                &v3_options
            )
            .is_ok()
    );

    let (mut client, _, _) = client_for(ExchangeScript::Success {
        response: asynchronous_response_bytes_with_version(
            ProtocolVersion::V3_0,
            DECAPSULATE_OPERATION,
            0,
            None,
            None,
            Some(uid_payload()),
        ),
        request_write_chunks: Vec::new(),
    });
    assert!(
        client
            .decapsulate_with_options(
                DecapsulateRequest::new("priv-1", vec![1, 2, 3]),
                &CodecLimits::defaults(),
                &v3_options
            )
            .is_ok()
    );
}

#[test]
fn client_async_convenience_methods_execute_roundtrip() {
    let payload = test_structure([
        test_item(
            ASYNC_CORRELATION_TAG,
            Value::byte_string(b"corr-1".to_vec()),
        ),
        test_item(CANCELLATION_RESULT_TAG, Value::enumeration(3)),
    ]);
    let response = asynchronous_response_bytes(CANCEL, 0, None, None, Some(payload));
    let (mut client, _, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let cancel_outcome = client
        .execute_cancel(CancelRequest::new(b"corr-1"), &CodecLimits::defaults())
        .expect("successful execute_cancel");
    assert_eq!(cancel_outcome.operation(), ClientOperation::Cancel);
    assert_eq!(
        cancel_outcome.with_cancel_echo(<[u8]>::to_vec),
        Some(b"corr-1".to_vec())
    );
    assert_eq!(
        cancel_outcome
            .cancel_response()
            .and_then(|r| r.cancellation_result()),
        Some(CancellationResult::Completed)
    );
    assert!(cancel_outcome.poll_response().is_none());
    assert!(cancel_outcome.process_response().is_none());
    assert!(cancel_outcome.query_async_requests_response().is_none());

    let (mut client, _, _) = client_for(ExchangeScript::Success {
        response: asynchronous_response_bytes(
            POLL,
            0,
            None,
            None,
            Some(test_structure([test_item(
                0x0042_0012,
                Value::byte_string(b"poll-resp".to_vec()),
            )])),
        ),
        request_write_chunks: Vec::new(),
    });
    let poll_outcome = client
        .execute_poll(PollRequest::new(b"corr-2"), &CodecLimits::defaults())
        .expect("successful execute_poll");
    assert_eq!(poll_outcome.operation(), ClientOperation::Poll);
    assert!(poll_outcome.poll_response().is_some());
    assert!(poll_outcome.cancel_response().is_none());
    assert!(poll_outcome.process_response().is_none());
    assert!(poll_outcome.query_async_requests_response().is_none());

    let (mut client, _, _) = client_for(ExchangeScript::Success {
        response: asynchronous_response_bytes(PROCESS, 0, None, None, Some(test_structure([]))),
        request_write_chunks: Vec::new(),
    });
    let process_outcome = client
        .execute_process(
            ProcessRequest::new(b"corr-3"),
            None,
            &CodecLimits::defaults(),
        )
        .expect("successful execute_process");
    assert_eq!(process_outcome.operation(), ClientOperation::Process);
    assert!(process_outcome.process_response().is_some());
    assert!(process_outcome.cancel_response().is_none());
    assert!(process_outcome.poll_response().is_none());
    assert!(process_outcome.query_async_requests_response().is_none());

    let (mut client, _, _) = client_for(ExchangeScript::Success {
        response: asynchronous_response_bytes(
            QUERY_ASYNC,
            0,
            None,
            None,
            Some(test_structure([test_item(
                ASYNCHRONOUS_CORRELATION_VALUES,
                Value::structure(test_structure([test_item(
                    0x0042_0006,
                    Value::byte_string(b"corr-4".to_vec()),
                )])),
            )])),
        ),
        request_write_chunks: Vec::new(),
    });
    let query_outcome = client
        .execute_query_async_requests(
            QueryAsyncRequestsRequest::default(),
            None,
            &CodecLimits::defaults(),
        )
        .expect("successful execute_query_async_requests");
    assert_eq!(
        query_outcome.operation(),
        ClientOperation::QueryAsyncRequests
    );
    assert!(query_outcome.query_async_requests_response().is_some());
    assert!(query_outcome.cancel_response().is_none());
    assert!(query_outcome.poll_response().is_none());
    assert!(query_outcome.process_response().is_none());
}

#[test]
fn async_batch_item_negative_branches() {
    // 1. Poll item negative branches
    let poll_payload = test_structure([test_item(
        0x0042_0012,
        Value::byte_string(b"opaque".to_vec()),
    )]);
    let response = asynchronous_response_bytes(POLL, 0, None, None, Some(poll_payload));
    let (mut client, _, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let batch = ClientBatch::new(ClientBatchItem::poll(PollRequest::new(b"corr-2")));
    let outcome = client
        .execute_with_options(batch, &CodecLimits::defaults(), &RequestOptions::default())
        .expect("successful Poll execution");
    let item = outcome.get(0).expect("one response item");
    assert!(item.cancel_response().is_none());
    assert!(item.process_response().is_none());
    assert!(item.query_async_requests_response().is_none());

    // 2. Process item negative branches
    let response = asynchronous_response_bytes(PROCESS, 0, None, None, Some(test_structure([])));
    let (mut client, _, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let batch = ClientBatch::new(ClientBatchItem::process(ProcessRequest::new(b"corr-3")));
    let outcome = client
        .execute_with_options(batch, &CodecLimits::defaults(), &RequestOptions::default())
        .expect("successful Process execution");
    let item = outcome.get(0).expect("one response item");
    assert!(item.cancel_response().is_none());
    assert!(item.poll_response().is_none());
    assert!(item.query_async_requests_response().is_none());

    // 3. QueryAsync item negative branches
    let query_payload = test_structure([test_item(
        ASYNCHRONOUS_CORRELATION_VALUES,
        Value::structure(test_structure([test_item(
            0x0042_0006,
            Value::byte_string(b"corr-4".to_vec()),
        )])),
    )]);
    let response = asynchronous_response_bytes(QUERY_ASYNC, 0, None, None, Some(query_payload));
    let (mut client, _, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let batch = ClientBatch::new(ClientBatchItem::query_async_requests(
        QueryAsyncRequestsRequest::default(),
    ));
    let outcome = client
        .execute_with_options(batch, &CodecLimits::defaults(), &RequestOptions::default())
        .expect("successful QueryAsync execution");
    let item = outcome.get(0).expect("one response item");
    assert!(item.cancel_response().is_none());
    assert!(item.poll_response().is_none());
    assert!(item.process_response().is_none());

    // 4. KMIP 3.0 item negative branches on async accessors
    let v3_options =
        RequestOptions::default().with_protocol_version(kmipkit_protocol::ProtocolVersion::V3_0);
    let uid_payload = test_structure([test_item(
        0x0042_0094,
        Value::text_string("obj-1".to_string()),
    )]);
    let response = asynchronous_response_bytes_with_version(
        kmipkit_protocol::ProtocolVersion::V3_0,
        0x0000_003C,
        0,
        None,
        None,
        Some(uid_payload),
    );
    let (mut client, _, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let batch = ClientBatch::new(ClientBatchItem::create_group(CreateGroupRequest::new(
        AttributeSet::new(),
    )));
    let outcome = client
        .execute_with_options(batch, &CodecLimits::defaults(), &v3_options)
        .expect("successful CreateGroup execution");
    let item = outcome.get(0).expect("one response item");
    assert!(item.cancel_response().is_none());
    assert!(item.poll_response().is_none());
    assert!(item.process_response().is_none());
    assert!(item.query_async_requests_response().is_none());
}

#[test]
fn async_batch_malformed_response_handling() {
    // Malformed payload errors mapping to asynchronous_operation_error
    let bad_cancel = asynchronous_response_bytes(CANCEL, 0, None, None, None);
    let (mut client, _, _) = client_for(ExchangeScript::Success {
        response: bad_cancel,
        request_write_chunks: Vec::new(),
    });
    assert!(
        client
            .execute_cancel(CancelRequest::new(b"corr-1"), &CodecLimits::defaults())
            .is_err()
    );

    let mismatched_echo = test_structure([
        test_item(
            ASYNC_CORRELATION_TAG,
            Value::byte_string(b"different-corr".to_vec()),
        ),
        test_item(CANCELLATION_RESULT_TAG, Value::enumeration(3)),
    ]);
    let response = asynchronous_response_bytes(CANCEL, 0, None, None, Some(mismatched_echo));
    let (mut client, _, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    assert!(
        client
            .execute_cancel(CancelRequest::new(b"corr-1"), &CodecLimits::defaults())
            .is_err()
    );

    let bad_cancel_batch = asynchronous_response_bytes(CANCEL, 0, None, None, None);
    let (mut client, _, _) = client_for(ExchangeScript::Success {
        response: bad_cancel_batch,
        request_write_chunks: Vec::new(),
    });
    let batch = ClientBatch::new(ClientBatchItem::cancel(CancelRequest::new(b"corr-1")));
    assert!(
        client
            .execute_with_options(batch, &CodecLimits::defaults(), &RequestOptions::default())
            .is_err()
    );

    let bad_query_batch = asynchronous_response_bytes(QUERY_ASYNC, 0, None, None, None);
    let (mut client, _, _) = client_for(ExchangeScript::Success {
        response: bad_query_batch,
        request_write_chunks: Vec::new(),
    });
    let batch = ClientBatch::new(ClientBatchItem::query_async_requests(
        QueryAsyncRequestsRequest::default(),
    ));
    assert!(
        client
            .execute_with_options(batch, &CodecLimits::defaults(), &RequestOptions::default())
            .is_err()
    );
}
