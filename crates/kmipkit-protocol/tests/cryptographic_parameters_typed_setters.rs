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

fn parameters_with_vendor_extension(hashing_algorithm: u32) -> CryptographicParameters {
    let mut structure = Structure::new();
    structure
        .try_push(item(HASHING_ALGORITHM, Value::enumeration(hashing_algorithm)))
        .expect("Hashing Algorithm fits in Cryptographic Parameters");
    structure
        .try_push(item(0x0054_0031, Value::byte_string(vec![0xca, 0xfe])))
        .expect("vendor extension fits after known Table 59 fields");
    CryptographicParameters::try_from_ttlv(structure)
        .expect("known and unknown fields remain valid")
}

fn key_information_preserves_parameters(
    fields: &kmipkit_ttlv::StructureView<'_>,
    identifier: &str,
    hashing_algorithm: u32,
) -> bool {
    let children = fields.children();
    children.len() == 2
        && children[0].tag().raw() == 0x0042_0094
        && children[0].with_value(|value| {
            matches!(value, ValueView::TextString(value) if value == identifier)
        })
        && children[1].tag().raw() == CRYPTOGRAPHIC_PARAMETERS
        && children[1].with_value(|value| match value {
            ValueView::Structure(parameters) => {
                let members = parameters.children();
                members.len() == 2
                    && members[0].tag().raw() == HASHING_ALGORITHM
                    && members[0].with_value(|value| {
                        matches!(value, ValueView::Enumeration(value) if *value == hashing_algorithm)
                    })
                    && members[1].tag().raw() == 0x0054_0031
                    && members[1].with_value(|value| {
                        matches!(value, ValueView::ByteString(value) if value == [0xca, 0xfe])
                    })
            }
            _ => false,
        })
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
        .with_typed_cryptographic_parameters(parameters_with_vendor_extension(0x8000_0033))
        .expect("typed parameters are structurally valid");
    assert!(
        encryption
            .with_raw(|structure| key_information_preserves_parameters(
                &structure,
                "wrap-key",
                0x8000_0033,
            ))
            .expect("encryption key information remains a structure")
    );

    let mac_signature = MacSignatureKeyInformation::try_new("mac-key".to_owned())
        .expect("identifier is valid")
        .with_typed_cryptographic_parameters(parameters_with_vendor_extension(0x8000_0044))
        .expect("typed parameters are structurally valid");
    assert!(
        mac_signature
            .with_raw(|structure| key_information_preserves_parameters(
                &structure,
                "mac-key",
                0x8000_0044,
            ))
            .expect("MAC/signature key information remains a structure")
    );

    let _ = KeyWrappingData::try_new(1).expect("raw wrapping method remains accepted");
}
