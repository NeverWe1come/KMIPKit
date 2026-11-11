//! Fluent Register tests derived from KMIP v2.1 §6.1.43, Tables 291–293,
//! and KMIPKIT-0100-FR-001 through FR-006.

use std::cell::RefCell;
use std::rc::Rc;

use kmipkit_protocol::{
    AttributeSet, Certificate, DeviceCredentialObject, DeviceCredentialObjectFields,
    HashedPasswordCredentialObject, HashedPasswordCredentialObjectFields, ManagedObject,
    ObjectType, OneTimePasswordCredentialObject, OneTimePasswordCredentialObjectFields,
    OpaqueObjectContext, PasswordCredentialObject, PasswordCredentialObjectFields, ProtocolVersion,
    SecretBytes, SecretText, ValidateKmipRequest,
};
use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::{RequestDeliveryState, RequestOptions, WireEncoding};
use kmipkit_ttlv::{Item, RawTag, Structure, Value, ValueView};

use crate::asynchronous_execution_test_support::{client_for, client_for_with_options_observer};
use crate::execute::{Client, ClientBatchOutcome, ClientOperation};
use crate::execute_test_support::{
    asynchronous_response_bytes, asynchronous_response_bytes_with_version, test_item,
    test_structure,
};
use crate::fluent::{FluentAttributePosition, FluentBuildError, RegisterBuilder};

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

fn credential_objects() -> [ManagedObject; 4] {
    let password = PasswordCredentialObject::try_new(PasswordCredentialObjectFields {
        password: Some(SecretText::new("register credential sentinel".to_owned())),
        ..PasswordCredentialObjectFields::default()
    })
    .expect("KMIP 3.0 Password Credential is valid");
    let device = DeviceCredentialObject::try_new(DeviceCredentialObjectFields {
        device_identifier: Some(SecretText::new("device-id".to_owned())),
        ..DeviceCredentialObjectFields::default()
    })
    .expect("KMIP 3.0 Device Credential is valid");
    let otp = OneTimePasswordCredentialObject::try_new(OneTimePasswordCredentialObjectFields {
        otp_algorithm: Some(1),
        ..OneTimePasswordCredentialObjectFields::default()
    })
    .expect("KMIP 3.0 OTP Credential is valid");
    let hashed_password =
        HashedPasswordCredentialObject::try_new(HashedPasswordCredentialObjectFields {
            hashed_username_password: Some(SecretBytes::new(vec![1, 2, 3])),
            hashed_password_username: Some(SecretBytes::new(vec![4, 5, 6])),
            ..HashedPasswordCredentialObjectFields::default()
        })
        .expect("KMIP 3.0 Hashed Password Credential is valid");

    [
        (13, password.into_item()),
        (14, device.into_item()),
        (15, otp.into_item()),
        (16, hashed_password.into_item()),
    ]
    .map(|(object_type, item)| {
        ManagedObject::try_from_item(ObjectType::from_raw(object_type), item)
            .expect("typed credential tag matches its KMIP 3.0 Object Type")
    })
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
        0x0042_0040,
        Value::structure(test_structure([test_item(
            0x0042_001e,
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
    assert_eq!(request.object().tag().raw(), 0x0042_0040);
    assert!(request.attributes().as_items().is_empty());
}

#[test]
fn register_builder_accepts_kmip30_typed_system_credential_objects() {
    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    let expected = [
        (13, 0x0042_01A1),
        (14, 0x0042_01A6),
        (15, 0x0042_01A7),
        (16, 0x0042_01AF),
    ];
    for ((object_type, item_tag), object) in expected.into_iter().zip(credential_objects()) {
        let request = client
            .request()
            .register()
            .object(object)
            .attributes(AttributeSet::new())
            .build_request()
            .expect("Register accepts KMIP 3.0 system credentials from Section 2");

        assert_eq!(request.object_type().raw(), object_type);
        assert_eq!(request.object().tag().raw(), item_tag);
        assert!(request.validate().is_valid());
    }
}

#[test]
fn register_builder_executes_a_credential_with_explicit_kmip30_version() {
    let response = asynchronous_response_bytes_with_version(
        ProtocolVersion::V3_0,
        REGISTER_OPERATION,
        0,
        None,
        None,
        Some(test_structure([test_item(
            0x0042_0094,
            Value::text_string("credential-id".to_owned()),
        )])),
    );
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let password = credential_objects()
        .into_iter()
        .next()
        .expect("first credential");
    let result = client
        .request()
        .register()
        .object(password)
        .attributes(AttributeSet::new())
        .with_options(RequestOptions::default().with_protocol_version(ProtocolVersion::V3_0))
        .execute(&kmipkit_ttlv::codec::CodecLimits::defaults())
        .expect("the explicitly selected KMIP 3.0 route accepts credential tags");

    assert_eq!(result.outcome().operation(), ClientOperation::Register);
    assert_eq!(fake.borrow().exchange_count(), 1);
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
    assert!(matches!(
        item.request(),
        crate::execute::ClientRequest::Register(_)
    ));
}

#[test]
fn register_builder_executes_one_typed_exchange_and_retains_failure_result() {
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
fn register_builder_exposes_the_server_assigned_identifier_on_success() {
    const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
    let response = asynchronous_response_bytes(
        REGISTER_OPERATION,
        0,
        None,
        None,
        Some(test_structure([test_item(
            UNIQUE_IDENTIFIER,
            Value::text_string("registered-certificate-id".to_owned()),
        )])),
    );
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let result = valid_builder(&mut client)
        .execute(&kmipkit_ttlv::codec::CodecLimits::defaults())
        .expect("the scripted Success completes one Register exchange");

    let ClientBatchOutcome::Register(register) = result.outcome() else {
        panic!("Register Success remains a typed Register response");
    };
    assert_eq!(register.result().status().raw(), 0);
    assert!(register.unique_identifier().is_some());
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn register_builder_preserves_pending_correlation_without_polling() {
    const CORRELATION: &[u8] = &[0x00, 0xff, 0x80, 0x01];
    let response = asynchronous_response_bytes(
        REGISTER_OPERATION,
        2,
        None,
        Some(CORRELATION),
        Some(test_structure([])),
    );
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let result = valid_builder(&mut client)
        .with_asynchronous_indicator(1)
        .execute(&kmipkit_ttlv::codec::CodecLimits::defaults())
        .expect("the current client returns Pending without polling");

    let ClientBatchOutcome::Pending(pending) = result.outcome() else {
        panic!("Register Pending remains a visible Pending outcome");
    };
    assert_eq!(pending.operation(), ClientOperation::Register);
    assert_eq!(pending.asynchronous_correlation_value(), CORRELATION);
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn register_builder_preserves_masks_and_placed_vendor_attributes() {
    const NAME: u32 = 0x0042_0053;
    const NAME_TYPE: u32 = 0x0042_0097;
    const NAME_VALUE: u32 = 0x0042_0098;
    const VENDOR_ATTRIBUTE: u32 = 0x0042_0008;
    const PROTECTION_STORAGE_MASKS: u32 = 0x0042_015f;
    let name = item(
        NAME,
        Value::structure(test_structure([
            test_item(NAME_TYPE, Value::enumeration(1)),
            test_item(NAME_VALUE, Value::text_string("registered-cert".to_owned())),
        ])),
    );
    let vendor_value = item(NAME_VALUE, Value::text_string("opaque-label".to_owned()));
    let masks = Structure::new();
    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    let request = valid_builder(&mut client)
        .attribute_at(FluentAttributePosition::ObjectAttributes { index: 1 }, name)
        .vendor_attribute("example.invalid", "label", &vendor_value)
        .protection_storage_masks(masks)
        .build()
        .expect("typed Register validation accepts these optional values");

    assert_eq!(
        request
            .attributes()
            .as_items()
            .iter()
            .map(|attribute| attribute.tag().raw())
            .collect::<Vec<_>>(),
        [
            CRYPTOGRAPHIC_USAGE_MASK,
            NAME,
            CERTIFICATE_LENGTH,
            CRYPTOGRAPHIC_ALGORITHM,
            CRYPTOGRAPHIC_LENGTH,
            VENDOR_ATTRIBUTE,
        ]
    );
    let payload = request
        .into_ttlv_payload()
        .expect("valid Register request creates the normal Table 291 payload");
    assert_eq!(
        payload
            .view()
            .children()
            .last()
            .map(|item| item.tag().raw()),
        Some(PROTECTION_STORAGE_MASKS)
    );
}

#[test]
fn register_builder_preserves_explicit_extension_tags_without_vendor_conversion() {
    let (mut client, fake, _) = client_for(ExchangeScript::FailBeforeWrite);
    let request = client
        .request()
        .register()
        .object(valid_certificate())
        .attributes(certificate_attributes())
        .attribute_item(item(0x0054_0000, Value::boolean(true)))
        .build_request()
        .expect("explicit extension-tag Items remain generic TTLV inputs");

    let extension = request
        .attributes()
        .as_items()
        .iter()
        .find(|attribute| attribute.tag().raw() == 0x0054_0000)
        .expect("the extension tag remains present");
    assert!(extension.with_value(|value| matches!(value, ValueView::Boolean(true))));
    assert_eq!(fake.borrow().exchange_count(), 0);
}

#[test]
fn register_builder_rejects_known_non_attribute_tags_before_transport() {
    const OPERATION_TAG: u32 = 0x0042_005c;
    let (mut client, fake, _) = client_for(ExchangeScript::FailBeforeWrite);
    let builder =
        valid_builder(&mut client).attribute_item(item(OPERATION_TAG, Value::enumeration(3)));

    assert_eq!(
        builder.validate(),
        Err(FluentBuildError::InvalidAttributeValue)
    );
    let error = builder
        .execute(&kmipkit_ttlv::codec::CodecLimits::defaults())
        .expect_err("Operation is an assigned KMIP tag but not an Object Attribute");

    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert!(!format!("{error:?} {error}").contains(OBJECT_SENTINEL));
    assert_eq!(fake.borrow().exchange_count(), 0);
}

#[test]
fn fluent_object_builders_reject_known_non_attribute_tags() {
    const OPERATION_TAG: u32 = 0x0042_005c;
    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    let operation = || item(OPERATION_TAG, Value::enumeration(3));

    assert_eq!(
        client
            .request()
            .create_symmetric_key()
            .attribute_item(operation())
            .build_request()
            .err(),
        Some(FluentBuildError::InvalidAttributeValue)
    );
    assert_eq!(
        client
            .request()
            .create_key_pair()
            .common_attributes(AttributeSet::try_new([operation()]).expect("single Item is valid"))
            .build_request()
            .err(),
        Some(FluentBuildError::InvalidAttributeValue)
    );
    assert_eq!(
        client
            .request()
            .create_split_key()
            .attribute_item(operation())
            .build_request()
            .err(),
        Some(FluentBuildError::InvalidAttributeValue)
    );
}

#[test]
fn unassigned_standard_tag_cannot_become_an_expert_attribute_item() {
    assert!(
        RawTag::new(0x0042_01ff)
            .expect("the value fits in the TTLV tag field")
            .try_checked()
            .is_err(),
        "reserved or unassigned KMIP tags cannot be wrapped in an Item"
    );
}

#[test]
fn register_builder_rejects_duplicate_and_out_of_range_attribute_positions() {
    let name = item(0x0042_0053, Value::structure(test_structure([])));
    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    let duplicate = valid_builder(&mut client)
        .attribute_at(FluentAttributePosition::ObjectAttributes { index: 1 }, name)
        .attribute_at(
            FluentAttributePosition::ObjectAttributes { index: 1 },
            item(0x0042_0053, Value::structure(test_structure([]))),
        );
    assert_eq!(
        duplicate.validate(),
        Err(FluentBuildError::DuplicateAttributePosition)
    );

    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    let out_of_range = valid_builder(&mut client).attribute_at(
        FluentAttributePosition::ObjectAttributes { index: 5 },
        item(0x0042_0053, Value::structure(test_structure([]))),
    );
    assert_eq!(
        out_of_range.validate(),
        Err(FluentBuildError::AttributePositionOutOfBounds)
    );
}

#[test]
fn register_builder_passes_exchange_options_to_the_existing_client() {
    let response = asynchronous_response_bytes(REGISTER_OPERATION, 1, Some(0x20), None, None);
    let observed_options = Rc::new(RefCell::new(Vec::new()));
    let (mut client, fake, _) = client_for_with_options_observer(
        ExchangeScript::Success {
            response,
            request_write_chunks: Vec::new(),
        },
        Rc::clone(&observed_options),
    );
    let options = RequestOptions::default().with_wire_encoding(WireEncoding::Ttlv);
    valid_builder(&mut client)
        .with_options(options)
        .execute(&kmipkit_ttlv::codec::CodecLimits::defaults())
        .expect("explicit options are forwarded to the existing exchange");

    assert_eq!(fake.borrow().exchange_count(), 1);
    assert_eq!(&*observed_options.borrow(), &[options]);
}

#[test]
fn invalid_register_object_is_rejected_before_transport() {
    let (mut client, fake, _) = client_for(ExchangeScript::FailBeforeWrite);
    let error = client
        .request()
        .register()
        .object_item(
            ObjectType::from_raw(1),
            item(0x0042_006d, Value::structure(test_structure([]))),
        )
        .execute(&kmipkit_ttlv::codec::CodecLimits::defaults())
        .expect_err("object type and object tag must match before transport");

    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert!(!format!("{error:?} {error}").contains(OBJECT_SENTINEL));
    assert_eq!(fake.borrow().exchange_count(), 0);
}

#[test]
fn register_builder_debug_does_not_retain_object_payload_in_error() {
    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);
    let request = valid_builder(&mut client)
        .build()
        .expect("test Certificate builds");
    let object_bytes = request.object().with_value(|value| match value {
        ValueView::Structure(structure) => {
            structure.children()[1].with_value(|value| match value {
                ValueView::ByteString(bytes) => bytes.to_vec(),
                _ => Vec::new(),
            })
        }
        _ => Vec::new(),
    });

    assert_eq!(object_bytes, OBJECT_SENTINEL.as_bytes());
    assert!(!format!("{request:?}").contains(OBJECT_SENTINEL));
}
