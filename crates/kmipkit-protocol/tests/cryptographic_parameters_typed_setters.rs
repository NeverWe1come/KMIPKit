//! KMIPKIT-0050 typed Cryptographic Parameters operation integration contracts.

use kmipkit_protocol::{
    AttributeSet, CryptographicParameters, CryptographicParametersBuilder, DecryptRequest,
    DerivationMethod, DerivationParameters, DeriveKeyRequest, EncryptRequest,
    EncryptionKeyInformation, KeyWrappingData, MacRequest, MacSignatureKeyInformation,
    MacVerifyRequest, ObjectType, OperationData, SecretBytes, SignRequest, SignatureVerifyRequest,
    UniqueIdentifier,
};
use kmipkit_ttlv::{Item, RawTag, Structure, Value, ValueView};

const CRYPTOGRAPHIC_PARAMETERS: u32 = 0x0042_002B;
const HASHING_ALGORITHM: u32 = 0x0042_0038;
const CRYPTOGRAPHIC_LENGTH: u32 = 0x0042_002A;
const CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_0028;

fn parameters() -> CryptographicParameters {
    CryptographicParametersBuilder::new()
        .with_hashing_algorithm(6)
        .build()
        .expect("Hashing Algorithm is valid without algorithm-specific requirements")
}

fn typed_parameters_are_present(payload: &Structure) -> bool {
    payload.view().children().iter().any(|field| {
        field.tag().raw() == CRYPTOGRAPHIC_PARAMETERS
            && field.with_value(|value| match value {
                ValueView::Structure(parameters) => parameters
                    .children()
                    .iter()
                    .any(|member| member.tag().raw() == HASHING_ALGORITHM),
                _ => false,
            })
    })
}

fn item(raw: u32, value: Value) -> Item {
    Item::new(
        RawTag::new(raw)
            .expect("test tag fits the TTLV tag width")
            .try_checked()
            .expect("test tag is catalog allocated"),
        value,
    )
    .expect("test item is valid")
}

#[test]
fn typed_parameters_attach_to_every_existing_operation_family() {
    let encrypt = EncryptRequest::new(
        None,
        Some(OperationData::ByteString(SecretBytes::new(vec![1]))),
    )
    .with_typed_cryptographic_parameters(parameters())
    .to_ttlv_payload()
    .expect("Encrypt request is valid");
    assert!(typed_parameters_are_present(&encrypt));

    let decrypt = DecryptRequest::new(
        None,
        Some(OperationData::ByteString(SecretBytes::new(vec![1]))),
    )
    .with_typed_cryptographic_parameters(parameters())
    .to_ttlv_payload()
    .expect("Decrypt request is valid");
    assert!(typed_parameters_are_present(&decrypt));

    let sign = SignRequest::new()
        .with_digested_data(SecretBytes::new(vec![1]))
        .with_typed_cryptographic_parameters(parameters())
        .to_ttlv_payload()
        .expect("Sign request is valid");
    assert!(typed_parameters_are_present(&sign));

    let signature_verify = SignatureVerifyRequest::new()
        .with_signature_data(SecretBytes::new(vec![1]))
        .with_typed_cryptographic_parameters(parameters())
        .to_ttlv_payload()
        .expect("Signature Verify request is valid");
    assert!(typed_parameters_are_present(&signature_verify));

    let mac = MacRequest::new()
        .with_data(OperationData::ByteString(SecretBytes::new(vec![1])))
        .with_typed_cryptographic_parameters(parameters())
        .to_ttlv_payload()
        .expect("MAC request is valid");
    assert!(typed_parameters_are_present(&mac));

    let mac_verify = MacVerifyRequest::new()
        .with_mac_data(SecretBytes::new(vec![1]))
        .with_typed_cryptographic_parameters(parameters())
        .to_ttlv_payload()
        .expect("MAC Verify request is valid");
    assert!(typed_parameters_are_present(&mac_verify));

    let derivation_parameters =
        DerivationParameters::new().with_typed_cryptographic_parameters(parameters());
    let attributes = AttributeSet::try_new([
        item(CRYPTOGRAPHIC_LENGTH, Value::integer(256)),
        item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3)),
    ])
    .expect("symmetric-key output attributes have valid types");
    let derive_key = DeriveKeyRequest::new(
        ObjectType::from_raw(2),
        vec![UniqueIdentifier::TextString("source".to_owned())],
        DerivationMethod::from_raw(2),
        derivation_parameters,
        attributes,
    )
    .into_ttlv_payload()
    .expect("Derive Key accepts caller-selected hash parameters");
    assert!(derive_key.view().children().iter().any(|field| {
        field.tag().raw() == 0x0042_0032
            && field.with_value(|value| match value {
                ValueView::Structure(parameters) => parameters.children().iter().any(|member| {
                    member.tag().raw() == CRYPTOGRAPHIC_PARAMETERS
                        && member.with_value(|value| match value {
                            ValueView::Structure(parameters) => parameters
                                .children()
                                .iter()
                                .any(|field| field.tag().raw() == HASHING_ALGORITHM),
                            _ => false,
                        })
                }),
                _ => false,
            })
    }));
}

#[test]
fn typed_parameters_attach_to_key_wrapping_information_structures() {
    let encryption = EncryptionKeyInformation::try_new("wrap-key".to_owned())
        .expect("identifier is valid")
        .with_typed_cryptographic_parameters(parameters())
        .expect("typed parameters are structurally valid");
    assert!(
        encryption
            .with_raw(|structure| structure
                .children()
                .iter()
                .any(|field| field.tag().raw() == CRYPTOGRAPHIC_PARAMETERS))
            .expect("encryption key information remains a structure")
    );

    let mac_signature = MacSignatureKeyInformation::try_new("mac-key".to_owned())
        .expect("identifier is valid")
        .with_typed_cryptographic_parameters(parameters())
        .expect("typed parameters are structurally valid");
    assert!(
        mac_signature
            .with_raw(|structure| structure
                .children()
                .iter()
                .any(|field| field.tag().raw() == CRYPTOGRAPHIC_PARAMETERS))
            .expect("MAC/signature key information remains a structure")
    );

    let _ = KeyWrappingData::try_new(1).expect("raw wrapping method remains accepted");
}

#[test]
// OASIS KMIP 2.1 §4.16, Table 59 defines the ordered optional members.
fn every_table_59_builder_setter_and_typed_reader_roundtrips() {
    let parameters = CryptographicParametersBuilder::new()
        .with_block_cipher_mode(6)
        .with_padding_method(2)
        .with_hashing_algorithm(6)
        .with_key_role_type(3)
        .with_digital_signature_algorithm(1)
        .with_cryptographic_algorithm(3)
        .with_random_iv(true)
        .with_iv_length(96)
        .with_tag_length(128)
        .with_fixed_field_length(32)
        .with_invocation_field_length(64)
        .with_counter_length(32)
        .with_initial_counter_value(1)
        .with_salt_length(16)
        .with_mask_generator(1)
        .with_mask_generator_hashing_algorithm(6)
        .with_p_source(vec![0xA5, 0x5A])
        .with_trailer_field(1)
        .with_extension(item(0x0054_0001, Value::text_string("vendor".to_owned())))
        .build()
        .expect("all optional Table 59 fields form a valid structure");

    assert_eq!(parameters.block_cipher_mode(), Some(6));
    assert_eq!(parameters.padding_method(), Some(2));
    assert_eq!(parameters.hashing_algorithm(), Some(6));
    assert_eq!(parameters.key_role_type(), Some(3));
    assert_eq!(parameters.digital_signature_algorithm(), Some(1));
    assert_eq!(parameters.cryptographic_algorithm(), Some(3));
    assert_eq!(parameters.random_iv(), Some(true));
    assert_eq!(parameters.iv_length(), Some(96));
    assert_eq!(parameters.tag_length(), Some(128));
    assert_eq!(parameters.fixed_field_length(), Some(32));
    assert_eq!(parameters.invocation_field_length(), Some(64));
    assert_eq!(parameters.counter_length(), Some(32));
    assert_eq!(parameters.initial_counter_value(), Some(1));
    assert_eq!(parameters.salt_length(), Some(16));
    assert_eq!(parameters.mask_generator(), Some(1));
    assert_eq!(parameters.mask_generator_hashing_algorithm(), Some(6));
    assert_eq!(
        parameters.with_p_source(<[u8]>::to_vec),
        Some(vec![0xA5, 0x5A])
    );
    assert_eq!(parameters.trailer_field(), Some(1));
    assert!(format!("{parameters:?}").contains("member_count"));
    assert!(parameters.with_ttlv(|raw| {
        raw.children()
            .iter()
            .any(|field| field.tag().raw() == 0x0054_0001)
    }));
}
