//! Compile-time client surface for the seven KMIP 3.0 operations in Table 583.

use kmipkit_protocol::{
    AttributeSet, CreateCredentialObject, CreateCredentialRequest, CreateGroupRequest,
    CreateUserRequest, CredentialType, DeactivateRequest, DecapsulateRequest, EncapsulateRequest,
    ObliterateRequest, PasswordCredentialObject, PasswordCredentialObjectFields, ProtocolVersion,
    SecretText,
};
use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::{RequestDeliveryState, RequestOptions};
use kmipkit_ttlv::codec::{CodecLimits, decode_with_limits};
use kmipkit_ttlv::{Item, RawTag, Structure, Value, ValueView};

use super::{
    Client, ClientBatch, ClientBatchItem, ClientBatchItemResponse, ClientError, ClientOperation,
    ClientRequest,
};

type DirectMethod<Request> =
    fn(&mut Client, Request, &CodecLimits) -> Result<ClientBatchItemResponse, ClientError>;
type DirectMethodWithOptions<Request> = fn(
    &mut Client,
    Request,
    &CodecLimits,
    &RequestOptions,
) -> Result<ClientBatchItemResponse, ClientError>;

macro_rules! assert_direct_methods {
    ($request:ty, $name:ident, $options_name:ident) => {
        let _: DirectMethod<$request> = Client::$name;
        let _: DirectMethodWithOptions<$request> = Client::$options_name;
    };
}

#[test]
fn all_seven_kmip30_operations_have_exact_table_583_codes() {
    assert_eq!(ClientOperation::CreateGroup.raw(), 0x3C);
    assert_eq!(ClientOperation::Obliterate.raw(), 0x3D);
    assert_eq!(ClientOperation::CreateUser.raw(), 0x3E);
    assert_eq!(ClientOperation::CreateCredential.raw(), 0x3F);
    assert_eq!(ClientOperation::Deactivate.raw(), 0x40);
    assert_eq!(ClientOperation::Encapsulate.raw(), 0x41);
    assert_eq!(ClientOperation::Decapsulate.raw(), 0x42);
}

#[test]
fn all_seven_kmip30_operations_have_direct_and_options_methods() {
    assert_direct_methods!(
        CreateCredentialRequest,
        create_credential,
        create_credential_with_options
    );
    assert_direct_methods!(CreateGroupRequest, create_group, create_group_with_options);
    assert_direct_methods!(CreateUserRequest, create_user, create_user_with_options);
    assert_direct_methods!(DeactivateRequest, deactivate, deactivate_with_options);
    assert_direct_methods!(DecapsulateRequest, decapsulate, decapsulate_with_options);
    assert_direct_methods!(EncapsulateRequest, encapsulate, encapsulate_with_options);
    assert_direct_methods!(ObliterateRequest, obliterate, obliterate_with_options);
}

#[test]
fn all_kmip30_operations_are_rejected_before_any_exchange_under_kmip21() {
    for version in [ProtocolVersion::V2_0, ProtocolVersion::V2_1] {
        for request in kmip30_requests() {
            let (mut client, transport, captured) =
                crate::asynchronous_execution_test_support::client_for(
                    ExchangeScript::FailBeforeWrite,
                );
            let error = client
                .execute_with_options(
                    ClientBatch::new(ClientBatchItem::new(request)),
                    &CodecLimits::defaults(),
                    &RequestOptions::default().with_protocol_version(version),
                )
                .expect_err("KMIP 3.0 operation must be gated under an older version");

            assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
            assert_eq!(transport.borrow().exchange_count(), 0);
            assert!(captured.borrow().is_none());
        }
    }
}

fn kmip30_requests() -> [ClientRequest; 7] {
    let credential_fields = PasswordCredentialObjectFields {
        password: Some(SecretText::new("sentinel".to_owned())),
        ..PasswordCredentialObjectFields::default()
    };
    let password = PasswordCredentialObject::try_new(credential_fields)
        .expect("password credential fixture is valid");
    [
        ClientRequest::create_group(CreateGroupRequest::new(AttributeSet::new())),
        ClientRequest::obliterate(ObliterateRequest::new(
            kmipkit_protocol::UniqueIdentifier::TextString("uid".to_owned()),
        )),
        ClientRequest::create_user(CreateUserRequest::new(AttributeSet::new())),
        ClientRequest::create_credential(CreateCredentialRequest::new(
            CredentialType::UsernameAndPassword,
            AttributeSet::new(),
            CreateCredentialObject::PasswordCredential(password),
        )),
        ClientRequest::deactivate(DeactivateRequest::new("uid")),
        ClientRequest::encapsulate(EncapsulateRequest::new("public-key")),
        ClientRequest::decapsulate(DecapsulateRequest::new("private-key", vec![1, 2, 3])),
    ]
}

#[test]
fn all_kmip30_requests_use_the_table_583_wire_code_and_preserve_typed_response() {
    for request in kmip30_requests() {
        let operation = request.operation();
        let payload = match operation {
            0x3D => Structure::new(),
            0x41 => test_structure([
                test_item(
                    0x0042_0094,
                    Value::text_string("new-secret-object".to_owned()),
                ),
                test_item(0x0042_00C2, Value::byte_string(vec![0xCA, 0xFE])),
            ]),
            _ => test_structure([test_item(
                0x0042_0094,
                Value::text_string("created-object".to_owned()),
            )]),
        };
        let (mut client, transport, captured) =
            crate::asynchronous_execution_test_support::client_for(ExchangeScript::Success {
                response: response_bytes(operation, payload),
                request_write_chunks: Vec::new(),
            });
        let batch = ClientBatch::new(ClientBatchItem::new(request))
            .with_protocol_version(ProtocolVersion::V3_0);
        let response = client
            .execute_with_options(batch, &CodecLimits::defaults(), &RequestOptions::default())
            .expect("KMIP 3.0 request and typed response are valid");

        assert_eq!(transport.borrow().exchange_count(), 1);
        assert_eq!(captured_request_operation(&captured), operation);
        let outcome = response.get(0).expect("one response item").outcome();
        assert_eq!(outcome.operation().raw(), operation);
        assert_eq!(outcome.result().status().raw(), 0);
        let typed = outcome.kmip30_response().expect("typed KMIP 3.0 response");
        assert_eq!(typed.operation().raw(), operation);
        if operation == 0x41 {
            let super::Kmip30OperationResponse::Encapsulate(response) = typed else {
                panic!("Encapsulate response variant is selected");
            };
            assert_eq!(
                response.unique_identifier(),
                Some(&kmipkit_protocol::UniqueIdentifier::TextString(
                    "new-secret-object".to_owned()
                ))
            );
            assert_eq!(response.data(), Some(&[0xCA, 0xFE][..]));
        }
        assert!(!format!("{outcome:?}").contains("shared_secret"));
    }
}

#[test]
fn kmip30_pending_response_keeps_typed_response_and_correlation_value() {
    let correlation = b"kmip30-pending-correlation";
    let (mut client, transport, _) =
        crate::asynchronous_execution_test_support::client_for(ExchangeScript::Success {
            response: pending_response_bytes(0x3C, correlation),
            request_write_chunks: Vec::new(),
        });
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::create_group(
        CreateGroupRequest::new(AttributeSet::new()),
    )))
    .with_protocol_version(ProtocolVersion::V3_0)
    .with_asynchronous_indicator(1);
    let response = client
        .execute_with_options(batch, &CodecLimits::defaults(), &RequestOptions::default())
        .expect("KMIP 3.0 Pending response is surfaced without polling");

    assert_eq!(transport.borrow().exchange_count(), 1);
    let outcome = response.get(0).expect("one response item").outcome();
    assert_eq!(outcome.operation(), ClientOperation::CreateGroup);
    assert_eq!(outcome.result().status().raw(), 2);
    assert_eq!(
        outcome.asynchronous_correlation_value(),
        Some(correlation.as_slice())
    );
    assert!(matches!(
        outcome.kmip30_response(),
        Some(super::Kmip30OperationResponse::CreateGroup(_))
    ));
}

fn captured_request_operation(
    request: &std::rc::Rc<std::cell::RefCell<Option<zeroize::Zeroizing<Vec<u8>>>>>,
) -> u32 {
    let captured = request.borrow();
    let bytes = captured
        .as_ref()
        .expect("transport captured the encoded request");
    let root = decode_with_limits(bytes, &CodecLimits::defaults())
        .expect("captured request is valid TTLV");
    root.with_value(|value| {
        let ValueView::Structure(message) = value else {
            panic!("request message is a Structure");
        };
        let batch = message
            .children()
            .iter()
            .find(|field| field.tag().raw() == 0x0042_000F)
            .expect("request contains Batch Item");
        batch.with_value(|value| {
            let ValueView::Structure(batch) = value else {
                panic!("Batch Item is a Structure");
            };
            let operation = batch
                .children()
                .iter()
                .find(|field| field.tag().raw() == 0x0042_005C)
                .expect("Batch Item contains Operation");
            operation.with_value(|value| match value {
                ValueView::Enumeration(operation) => *operation,
                _ => panic!("Operation uses Enumeration"),
            })
        })
    })
}

fn response_bytes(operation: u32, payload: Structure) -> Vec<u8> {
    response_with_result(operation, 0, Some(payload), None)
}

fn pending_response_bytes(operation: u32, correlation: &[u8]) -> Vec<u8> {
    response_with_result(operation, 2, Some(Structure::new()), Some(correlation))
}

fn response_with_result(
    operation: u32,
    status: u32,
    payload: Option<Structure>,
    correlation: Option<&[u8]>,
) -> Vec<u8> {
    let version = test_structure([
        test_item(0x0042_006A, Value::integer(3)),
        test_item(0x0042_006B, Value::integer(0)),
    ]);
    let header = test_structure([
        test_item(0x0042_0069, Value::structure(version)),
        test_item(0x0042_0092, Value::date_time(1)),
        test_item(0x0042_000D, Value::integer(1)),
    ]);
    let mut batch_items = vec![
        test_item(0x0042_005C, Value::enumeration(operation)),
        test_item(0x0042_007F, Value::enumeration(status)),
    ];
    if let Some(correlation) = correlation {
        batch_items.push(test_item(
            0x0042_0006,
            Value::byte_string(correlation.to_vec()),
        ));
    }
    if let Some(payload) = payload {
        batch_items.push(test_item(0x0042_007C, Value::structure(payload)));
    }
    let batch = test_structure(batch_items);
    let message = test_structure([
        test_item(0x0042_007A, Value::structure(header)),
        test_item(0x0042_000F, Value::structure(batch)),
    ]);
    super::encode_message_for_test(message, &CodecLimits::defaults())
        .expect("KMIP 3.0 response fixture is encodable")
}

fn test_structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut structure = Structure::new();
    for item in items {
        structure.try_push(item).expect("test Structure is valid");
    }
    structure
}

fn test_item(raw_tag: u32, value: Value) -> Item {
    let tag = RawTag::new(raw_tag)
        .expect("fixture Tag is representable")
        .try_checked()
        .expect("fixture Tag is catalogued or an allocated extension");
    Item::new(tag, value).expect("fixture Item is structurally valid")
}
