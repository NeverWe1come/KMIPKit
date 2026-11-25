//! Compile-time client surface for the seven KMIP 3.0 operations in Table 584.

use kmipkit_protocol::{
    AttributeSet, CreateCredentialObject, CreateCredentialRequest, CreateGroupRequest,
    CreateUserRequest, DeactivateRequest, DecapsulateRequest, EncapsulateRequest,
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
fn all_seven_kmip30_operations_have_exact_table_584_codes() {
    assert_eq!(ClientOperation::CreateGroup.raw(), 0x3C);
    assert_eq!(ClientOperation::Obliterate.raw(), 0x3D);
    assert_eq!(ClientOperation::CreateUser.raw(), 0x3E);
    assert_eq!(ClientOperation::CreateCredential.raw(), 0x3F);
    assert_eq!(ClientOperation::Deactivate.raw(), 0x40);
    assert_eq!(ClientOperation::Encapsulate.raw(), 0x41);
    assert_eq!(ClientOperation::Decapsulate.raw(), 0x42);
}

#[test]
fn draft_batch_item_routes_keep_each_fixed_operation_identity() {
    let routes = [
        (ClientBatchItem::create_group_draft(Structure::new()), 0x3C),
        (ClientBatchItem::obliterate_draft(Structure::new()), 0x3D),
        (ClientBatchItem::create_user_draft(Structure::new()), 0x3E),
        (
            ClientBatchItem::create_credential_draft(Structure::new()),
            0x3F,
        ),
        (ClientBatchItem::deactivate_draft(Structure::new()), 0x40),
        (ClientBatchItem::encapsulate_draft(Structure::new()), 0x41),
        (ClientBatchItem::decapsulate_draft(Structure::new()), 0x42),
    ];
    for (item, expected_operation) in routes {
        assert_eq!(item.operation(), expected_operation);
    }
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

macro_rules! assert_direct_calls {
    ($operation:expr, $request:expr, $method:ident, $options_method:ident) => {{
        let (mut client, transport, _) =
            crate::asynchronous_execution_test_support::client_for(ExchangeScript::FailBeforeWrite);
        let error = client
            .$method($request, &CodecLimits::defaults())
            .expect_err("KMIP 3.0 method is gated by the default KMIP 2.1 configuration");
        assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
        assert_eq!(transport.borrow().exchange_count(), 0);

        let (mut client, transport, _) =
            crate::asynchronous_execution_test_support::client_for(ExchangeScript::Success {
                response: direct_response_bytes($operation),
                request_write_chunks: Vec::new(),
            });
        let response = client
            .$options_method(
                $request,
                &CodecLimits::defaults(),
                &RequestOptions::default().with_protocol_version(ProtocolVersion::V3_0),
            )
            .expect("KMIP 3.0 direct method executes with an explicit version");
        assert_eq!(
            response
                .kmip30_response()
                .map(|value| value.operation().raw()),
            Some($operation)
        );
        assert_eq!(transport.borrow().exchange_count(), 1);
    }};
}

#[test]
fn all_seven_kmip30_direct_client_methods_follow_the_configured_version() {
    assert_direct_calls!(
        0x3C,
        CreateGroupRequest::new(AttributeSet::new()),
        create_group,
        create_group_with_options
    );
    assert_direct_calls!(
        0x3D,
        ObliterateRequest::new(kmipkit_protocol::UniqueIdentifier::TextString(
            "uid".to_owned()
        )),
        obliterate,
        obliterate_with_options
    );
    assert_direct_calls!(
        0x3E,
        CreateUserRequest::new(AttributeSet::new()),
        create_user,
        create_user_with_options
    );
    assert_direct_calls!(
        0x3F,
        CreateCredentialRequest::new(
            AttributeSet::new(),
            CreateCredentialObject::PasswordCredential(
                PasswordCredentialObject::try_new(PasswordCredentialObjectFields {
                    password: Some(SecretText::new("sentinel".to_owned())),
                    ..PasswordCredentialObjectFields::default()
                })
                .expect("password credential fixture is valid"),
            ),
        ),
        create_credential,
        create_credential_with_options
    );
    assert_direct_calls!(
        0x40,
        DeactivateRequest::new("uid"),
        deactivate,
        deactivate_with_options
    );
    assert_direct_calls!(
        0x41,
        EncapsulateRequest::new("public-key"),
        encapsulate,
        encapsulate_with_options
    );
    assert_direct_calls!(
        0x42,
        DecapsulateRequest::new("private-key", vec![1, 2, 3]),
        decapsulate,
        decapsulate_with_options
    );
}

fn direct_response_bytes(operation: u32) -> Vec<u8> {
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
    response_bytes(operation, payload)
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

#[test]
fn kmip30_typed_requests_have_explicit_validation_dispatch() {
    for request in kmip30_requests() {
        assert!(
            request.validation_report().is_valid(),
            "KMIP 3.0 typed requests do not use KMIP 2.1 Request Payload schemas"
        );
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
            AttributeSet::new(),
            CreateCredentialObject::PasswordCredential(password),
        )),
        ClientRequest::deactivate(DeactivateRequest::new("uid")),
        ClientRequest::encapsulate(EncapsulateRequest::new("public-key")),
        ClientRequest::decapsulate(DecapsulateRequest::new("private-key", vec![1, 2, 3])),
    ]
}

#[test]
// KMIP 3.0 Working Draft §6.1 operation tables and Table 584 define this
// request/response surface; the exact pinned source is recorded in the implementation.
fn all_kmip30_requests_use_the_table_584_wire_code_and_preserve_typed_response() {
    for request in kmip30_requests() {
        let operation = request.operation();
        let request_debug = format!("{request:?}");
        assert!(!request_debug.contains("sentinel"));
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
        assert_eq!(outcome.response().result().status().raw(), 0);
        assert_eq!(
            outcome
                .response()
                .kmip30()
                .map(|value| value.operation().raw()),
            Some(operation)
        );
        assert!(format!("{:?}", outcome.response()).contains("Kmip30"));
        let outcome_display = format!("{outcome}");
        assert_ne!(outcome_display, "");
        assert!(!outcome_display.contains("sentinel"));
        assert_eq!(
            response
                .get(0)
                .and_then(ClientBatchItemResponse::kmip30_response)
                .map(|value| value.operation().raw()),
            Some(operation)
        );
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
    crate::execute_test_support::asynchronous_response_bytes_with_version(
        ProtocolVersion::V3_0,
        operation,
        status,
        None,
        correlation,
        payload,
    )
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

#[test]
fn kmip30_multi_item_batch_uses_wire_order_without_batch_ids() {
    let response = test_structure([
        test_item(
            0x0042_007a,
            Value::structure(test_structure([
                test_item(
                    0x0042_0069,
                    Value::structure(test_structure([
                        test_item(0x0042_006a, Value::integer(3)),
                        test_item(0x0042_006b, Value::integer(0)),
                    ])),
                ),
                test_item(0x0042_0092, Value::date_time(1)),
            ])),
        ),
        test_item(
            0x0042_000f,
            Value::structure(test_structure([
                test_item(0x0042_005c, Value::enumeration(ClientOperation::Ping.raw())),
                test_item(0x0042_007f, Value::enumeration(0)),
                test_item(0x0042_007c, Value::structure(Structure::new())),
            ])),
        ),
        test_item(
            0x0042_000f,
            Value::structure(test_structure([
                test_item(0x0042_005c, Value::enumeration(ClientOperation::Ping.raw())),
                test_item(0x0042_007f, Value::enumeration(1)),
                test_item(0x0042_007e, Value::enumeration(1)),
            ])),
        ),
    ]);
    let limits = CodecLimits::defaults();
    let response = super::encode_message_for_test(response, &limits).unwrap();
    let (mut client, fake, _) =
        crate::asynchronous_execution_test_support::client_for(ExchangeScript::Success {
            response,
            request_write_chunks: Vec::new(),
        });
    let batch = ClientBatch::from_items([
        ClientBatchItem::new(ClientRequest::Ping(kmipkit_protocol::PingRequest::new())),
        ClientBatchItem::new(ClientRequest::Ping(kmipkit_protocol::PingRequest::new())),
    ])
    .with_protocol_version(ProtocolVersion::V3_0);
    let result = client.execute(batch, &limits).unwrap();
    let statuses: Vec<_> = result
        .iter()
        .map(|item| item.outcome().result().status().raw())
        .collect();
    assert_eq!(statuses, [0, 1]);
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn kmip30_poll_uses_a_header_without_batch_count() {
    use crate::asynchronous_execution_test_support::client_for;
    let response = response_with_result(
        ClientOperation::Poll.raw(),
        2,
        None,
        Some(b"test-correlation"),
    );
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let result = client
        .execute_poll_with_options(
            kmipkit_protocol::PollRequest::new(b"test-correlation"),
            &CodecLimits::defaults(),
            &RequestOptions::default().with_protocol_version(ProtocolVersion::V3_0),
        )
        .unwrap();
    assert!(result.is_pending());
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn query_credential_information_is_rejected_in_both_older_editions() {
    use crate::asynchronous_execution_test_support::client_for;
    use kmipkit_protocol::{QueryFunction, QueryRequest};
    let (mut client, fake, _) =
        client_for(ExchangeScript::FailAfterPartialWrite { written_bytes: 2 });
    for version in [ProtocolVersion::V2_0, ProtocolVersion::V2_1] {
        let request =
            ClientRequest::Query(QueryRequest::new([QueryFunction::CREDENTIAL_INFORMATION]));
        let batch = ClientBatch::new(ClientBatchItem::new(request)).with_protocol_version(version);
        let validation = super::validate_batch(&batch, version)
            .expect_err("Query Credential Information is unavailable before KMIP 3.0");
        assert_eq!(
            validation.to_string(),
            "Query Credential Information requires KMIP 3.0"
        );
        let error = client.execute(batch, &CodecLimits::defaults()).unwrap_err();
        assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    }
    assert_eq!(fake.borrow().exchange_count(), 0);
}

#[test]
fn kmip30_locate_object_class_mask_survives_fluent_attribute_replacement() {
    use kmipkit_protocol::ObjectClassMask;
    use kmipkit_ttlv::KmipTag;

    let (mut client, _, _) = crate::asynchronous_execution_test_support::client_for(
        ExchangeScript::FailAfterPartialWrite { written_bytes: 2 },
    );
    let mask = ObjectClassMask::USER | ObjectClassMask::SYSTEM;
    let mask = mask | ObjectClassMask::from_raw(0x8000_0000);
    let request = client
        .request()
        .locate(AttributeSet::new())
        .object_class_mask(mask)
        .attributes(AttributeSet::new())
        .build_request()
        .unwrap();
    assert_eq!(request.object_class_mask(), Some(mask));
    let payload = request.to_ttlv_payload().unwrap();
    assert!(payload.view().children().iter().any(|field| {
        field.tag().raw() == KmipTag::ObjectClassMask.as_u32()
            && field.with_value(|value| matches!(value, ValueView::Integer(bits) if bits.cast_unsigned() == 0x8000_0003))
    }));
}

#[test]
fn kmip30_create_key_pair_seed_is_encoded_and_rejected_in_older_editions() {
    use kmipkit_protocol::{CreateKeyPairRequest, SecretBytes};
    use kmipkit_ttlv::KmipTag;

    use crate::fluent::CreateKeyPairBuilder;
    use kmipkit_protocol::CryptographicAlgorithm;
    use kmipkit_protocol::named_values_generated::values::enumeration::cryptographic_algorithm::RSA;

    let seed = b"fixture-seed";
    let built = CreateKeyPairBuilder::detached()
        .algorithm(CryptographicAlgorithm::from_raw(RSA))
        .length(2048)
        .seed(SecretBytes::new(seed.to_vec()))
        .build()
        .unwrap();
    let debug = format!("{built:?}");
    assert!(debug.contains("has_seed: true"));
    assert!(!debug.contains("fixture-seed"));
    let request = || CreateKeyPairRequest::new().with_seed(SecretBytes::new(seed.to_vec()));
    let payload = built.into_ttlv_payload().unwrap();
    assert!(payload.view().children().iter().any(|field| {
        field.tag().raw() == KmipTag::Seed.as_u32()
            && field
                .with_value(|value| matches!(value, ValueView::ByteString(bytes) if bytes == seed))
    }));
    let (mut client, fake, _) = crate::asynchronous_execution_test_support::client_for(
        ExchangeScript::FailAfterPartialWrite { written_bytes: 2 },
    );
    for version in [ProtocolVersion::V2_0, ProtocolVersion::V2_1] {
        let batch = ClientBatch::new(ClientBatchItem::new(
            ClientRequest::CreateKeyPair(request()),
        ))
        .with_protocol_version(version);
        let error = client.execute(batch, &CodecLimits::defaults()).unwrap_err();
        assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    }
    assert_eq!(fake.borrow().exchange_count(), 0);
}

#[test]
fn kmip30_response_association_checks_operations_and_allows_only_a_missing_suffix() {
    use super::{BatchIdentity, ResponseAssociationError, associate_v3_batch_items};

    let identity = |operation: ClientOperation| BatchIdentity {
        operation: operation.raw(),
        unique_batch_item_id: None,
        response_context: None,
    };
    let requests = [
        identity(ClientOperation::Ping),
        identity(ClientOperation::Query),
    ];
    let prefix = [identity(ClientOperation::Ping)];
    assert_eq!(
        associate_v3_batch_items(&requests, &prefix, true),
        Ok(vec![Some(0), None])
    );
    assert_eq!(
        associate_v3_batch_items(&requests, &prefix, false),
        Err(ResponseAssociationError::ItemCountMismatch)
    );
    let reordered = [
        identity(ClientOperation::Query),
        identity(ClientOperation::Ping),
    ];
    assert_eq!(
        associate_v3_batch_items(&requests, &reordered, false),
        Err(ResponseAssociationError::ItemOperationMismatch)
    );
}

#[test]
fn kmip30_rejects_explicit_batch_item_ids_before_transport() {
    let (mut client, transport, _) =
        crate::asynchronous_execution_test_support::client_for(ExchangeScript::FailBeforeWrite);
    let batch = ClientBatch::new(
        ClientBatchItem::new(ClientRequest::Ping(kmipkit_protocol::PingRequest::new()))
            .with_unique_batch_item_id(vec![1]),
    )
    .with_protocol_version(ProtocolVersion::V3_0);
    let validation = super::validate_batch(&batch, ProtocolVersion::V3_0)
        .expect_err("KMIP 3.0 reserves Unique Batch Item ID");
    assert_eq!(
        validation.to_string(),
        "Unique Batch Item ID is reserved in KMIP 3.0"
    );
    let error = client.execute(batch, &CodecLimits::defaults()).unwrap_err();
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(transport.borrow().exchange_count(), 0);
}
