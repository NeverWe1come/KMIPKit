//! Fluent Register tests derived from KMIP v2.1 §6.1.43, Tables 291–293,
//! and KMIPKIT-0100-FR-001 through FR-006.

use kmipkit_protocol::{
    AttributeSet, Certificate, ManagedObject, ObjectType, OpaqueObjectContext,
};
use kmipkit_test_support::ExchangeScript;
use kmipkit_ttlv::{Item, RawTag, Value, ValueView};

use crate::asynchronous_execution_test_support::client_for;
use crate::execute::{Client, ClientOperation};
use crate::execute_test_support::{asynchronous_response_bytes, test_item, test_structure};
use crate::fluent::{FluentBuildError, RegisterBuilder};

const REGISTER_OPERATION: u32 = 3;
const CERTIFICATE: u32 = 0x0042_0013;
const CRYPTOGRAPHIC_USAGE_MASK: u32 = 0x0042_002c;
const CERTIFICATE_LENGTH: u32 = 0x0042_00ad;
const CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_0028;
const CRYPTOGRAPHIC_LENGTH: u32 = 0x0042_002a;
const OBJECT_SENTINEL: &str = "FLUENT_REGISTER_SECRET_SENTINEL";

fn valid_certificate() -> ManagedObject {
    let certificate = Certificate::try_new(1, OBJECT_SENTINEL.as_bytes().to_vec())
        .expect("test Certificate fields are valid");
    ManagedObject::try_from_item(ObjectType::from_raw(1), certificate.into_item())
        .expect("Certificate tag matches its Object Type")
}

fn certificate_attributes() -> AttributeSet {
    AttributeSet::try_new([
        test_item(CRYPTOGRAPHIC_USAGE_MASK, Value::integer(3)),
        test_item(CERTIFICATE_LENGTH, Value::integer(128)),
        test_item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(4)),
        test_item(CRYPTOGRAPHIC_LENGTH, Value::integer(1024)),
    ])
    .expect("test Certificate attributes are structurally valid")
}

fn valid_builder(client: &mut Client) -> RegisterBuilder<'_> {
    client
        .request()
        .register()
        .object(valid_certificate())
        .attributes(certificate_attributes())
}

fn item(raw_tag: u32, value: Value) -> Item {
    Item::new(
        RawTag::new(raw_tag)
            .expect("test tag fits")
            .try_checked()
            .expect("test tag is valid"),
        value,
    )
    .expect("test item is valid")
}

#[test]
fn register_builder_requires_an_object_before_building_or_validation() {
    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    let builder = client.request().register();

    assert_eq!(
        builder.validate(),
        Err(FluentBuildError::MissingRegisterObject)
    );
    assert_eq!(
        builder.build_request().expect_err("object is required"),
        FluentBuildError::MissingRegisterObject
    );
}

#[test]
fn register_builder_preserves_typed_object_and_ordered_attributes() {
    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    let request = valid_builder(&mut client)
        .build_request()
        .expect("valid typed Certificate builds");

    assert_eq!(request.object_type().raw(), 1);
    assert_eq!(request.object().tag().raw(), CERTIFICATE);
    assert_eq!(
        request
            .attributes()
            .as_items()
            .iter()
            .map(|attribute| attribute.tag().raw())
            .collect::<Vec<_>>(),
        [
            CRYPTOGRAPHIC_USAGE_MASK,
            CERTIFICATE_LENGTH,
            CRYPTOGRAPHIC_ALGORITHM,
            CRYPTOGRAPHIC_LENGTH,
        ]
    );
    assert!(request.validate().is_valid());
    assert!(!format!("{request:?}").contains(OBJECT_SENTINEL));
}

#[test]
fn register_builder_accepts_explicit_unknown_object_type_and_item() {
    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    let object_type = ObjectType::from_raw(0x8000_0001);
    let object = item(
        0x0042_0301,
        Value::structure(test_structure([test_item(
            0x0042_0302,
            Value::byte_string(b"opaque-data".to_vec()),
        )])),
    );
    let request = client
        .request()
        .register()
        .object_item(object_type, object)
        .build()
        .expect("explicit generic object remains supported");

    assert_eq!(request.object_type(), object_type);
    assert_eq!(request.object().tag().raw(), 0x0042_0301);
    assert!(request.attributes().as_items().is_empty());
}

#[test]
fn register_builder_keeps_opaque_context_explicit_and_local() {
    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    let error = client
        .request()
        .register()
        .object(valid_certificate())
        .opaque_object_context(OpaqueObjectContext::ManagedCryptographicObject)
        .build_request()
        .expect_err("Opaque Object context does not apply to Certificate");

    assert_eq!(error, FluentBuildError::InvalidRegisterObject);
}

#[test]
fn register_builder_creates_typed_batch_item_without_manual_request_construction() {
    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    let item = valid_builder(&mut client)
        .to_batch_item()
        .expect("valid Register builder converts to a batch item");
    assert_eq!(item.operation(), REGISTER_OPERATION);
    assert!(matches!(item.request(), crate::execute::ClientRequest::Register(_)));
}

#[test]
fn register_builder_executes_one_typed_exchange_and_retains_result() {
    let response = asynchronous_response_bytes(REGISTER_OPERATION, 1, Some(0x20), None, None);
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let result = valid_builder(&mut client)
        .execute(&kmipkit_ttlv::codec::CodecLimits::defaults())
        .expect("the scripted response completes one exchange");

    assert_eq!(fake.borrow().exchange_count(), 1);
    assert_eq!(result.outcome().operation(), ClientOperation::Register);
    assert_eq!(result.outcome().result().status().raw(), 1);
}

#[test]
fn invalid_register_object_is_rejected_before_transport() {
    let mismatched = ManagedObject::try_from_item(
        ObjectType::from_raw(1),
        item(
            CERTIFICATE,
            Value::structure(test_structure([test_item(
                0x0042_001d,
                Value::enumeration(1),
            )])),
        ),
    );
    assert!(mismatched.is_err());

    let (mut client, fake, _) = client_for(ExchangeScript::FailBeforeWrite);
    let err = client
        .request()
        .register()
        .object_item(
            ObjectType::from_raw(1),
            item(
                0x0042_006d,
                Value::structure(test_structure([])),
            ),
        )
        .build_request()
        .expect_err("object type and object tag must match");

    assert_eq!(err, FluentBuildError::InvalidRegisterObject);
    assert_eq!(fake.borrow().exchange_count(), 0);
}

#[test]
fn register_builder_debug_does_not_retain_object_payload_in_error() {
    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    let request = valid_builder(&mut client)
        .build()
        .expect("test Certificate builds");
    let object_bytes = request.object().with_value(|value| match value {
        ValueView::Structure(structure) => structure.children()[1]
            .with_value(|value| match value {
                ValueView::ByteString(bytes) => bytes.to_vec(),
                _ => Vec::new(),
            }),
        _ => Vec::new(),
    });

    assert_eq!(object_bytes, OBJECT_SENTINEL.as_bytes());
    assert!(!format!("{request:?}").contains(OBJECT_SENTINEL));
}
