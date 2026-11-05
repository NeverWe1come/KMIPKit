//! Public API contract tests for KMIP v2.1 §4.16 Table 59 Cryptographic Parameters.
//!
//! Structure field order follows §10.1.2. Unknown member preservation and raw
//! Enumeration values are project API compatibility requirements.
//!
//! Traceability: KMIPKIT-0050-FR-001–FR-005; SC-002, SC-005; the 18
//! `KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-*` IDs in
//! `specs/050-parameters-auth-client/traceability.md`.

use kmipkit_protocol::{CryptographicParameters, CryptographicParametersBuilder};
use kmipkit_ttlv::{Item, RawTag, Structure, Tag, Value, ValueView};

const BLOCK_CIPHER_MODE: u32 = 0x0042_0011;
const CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_0028;
const HASHING_ALGORITHM: u32 = 0x0042_0038;
const PADDING_METHOD: u32 = 0x0042_005F;
const KEY_ROLE_TYPE: u32 = 0x0042_0083;
const DIGITAL_SIGNATURE_ALGORITHM: u32 = 0x0042_00AE;
const RANDOM_IV: u32 = 0x0042_00C5;
const IV_LENGTH: u32 = 0x0042_00CD;
const TAG_LENGTH: u32 = 0x0042_00CE;
const FIXED_FIELD_LENGTH: u32 = 0x0042_00CF;
const COUNTER_LENGTH: u32 = 0x0042_00D0;
const INITIAL_COUNTER_VALUE: u32 = 0x0042_00D1;
const INVOCATION_FIELD_LENGTH: u32 = 0x0042_00D2;
const SALT_LENGTH: u32 = 0x0042_0100;
const MASK_GENERATOR: u32 = 0x0042_0101;
const MASK_GENERATOR_HASHING_ALGORITHM: u32 = 0x0042_0102;
const P_SOURCE: u32 = 0x0042_0103;
const TRAILER_FIELD: u32 = 0x0042_0104;
const VENDOR_EXTENSION: u32 = 0x0054_0001;

const TABLE_59_ORDER: [u32; 18] = [
    BLOCK_CIPHER_MODE,
    PADDING_METHOD,
    HASHING_ALGORITHM,
    KEY_ROLE_TYPE,
    DIGITAL_SIGNATURE_ALGORITHM,
    CRYPTOGRAPHIC_ALGORITHM,
    RANDOM_IV,
    IV_LENGTH,
    TAG_LENGTH,
    FIXED_FIELD_LENGTH,
    INVOCATION_FIELD_LENGTH,
    COUNTER_LENGTH,
    INITIAL_COUNTER_VALUE,
    SALT_LENGTH,
    MASK_GENERATOR,
    MASK_GENERATOR_HASHING_ALGORITHM,
    P_SOURCE,
    TRAILER_FIELD,
];

fn tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("fixture tag fits KMIP tag width")
        .try_checked()
        .expect("fixture tag is allocated by KMIP 2.1")
}

fn item(raw: u32, value: Value) -> Item {
    Item::new(tag(raw), value).expect("checked tag and value make an Item")
}

fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut value = Structure::new();
    for field in items {
        value
            .try_push(field)
            .expect("fixture remains within the TTLV structure limits");
    }
    value
}

fn item_tags(value: &CryptographicParameters) -> Vec<u32> {
    value.with_ttlv(|tree| {
        tree.children()
            .iter()
            .map(|field| field.tag().raw())
            .collect()
    })
}

fn complete_builder() -> CryptographicParametersBuilder {
    CryptographicParametersBuilder::new()
        .with_trailer_field(18)
        .with_p_source(vec![0x80, 0x01, 0xFE])
        .with_mask_generator_hashing_algorithm(0x8FFF_0002)
        .with_mask_generator(0x8FFF_0001)
        .with_salt_length(14)
        .with_initial_counter_value(13)
        .with_counter_length(12)
        .with_invocation_field_length(11)
        .with_fixed_field_length(10)
        .with_tag_length(9)
        .with_iv_length(8)
        .with_random_iv(true)
        .with_cryptographic_algorithm(0x8FFF_0003)
        .with_digital_signature_algorithm(0x8FFF_0004)
        .with_key_role_type(0x8FFF_0005)
        .with_hashing_algorithm(0x8FFF_0006)
        .with_padding_method(0x8FFF_0007)
        .with_block_cipher_mode(0x8FFF_0008)
}

#[test]
fn builder_emits_all_table_59_members_in_normative_order() {
    let parameters = complete_builder()
        .build()
        .expect("all supplied Table 59 members have valid TTLV types");

    assert_eq!(item_tags(&parameters), TABLE_59_ORDER);
    assert_eq!(parameters.block_cipher_mode(), Some(0x8FFF_0008));
    assert_eq!(parameters.padding_method(), Some(0x8FFF_0007));
    assert_eq!(parameters.hashing_algorithm(), Some(0x8FFF_0006));
    assert_eq!(parameters.key_role_type(), Some(0x8FFF_0005));
    assert_eq!(parameters.digital_signature_algorithm(), Some(0x8FFF_0004));
    assert_eq!(parameters.cryptographic_algorithm(), Some(0x8FFF_0003));
    assert_eq!(parameters.random_iv(), Some(true));
    assert_eq!(parameters.iv_length(), Some(8));
    assert_eq!(parameters.tag_length(), Some(9));
    assert_eq!(parameters.fixed_field_length(), Some(10));
    assert_eq!(parameters.invocation_field_length(), Some(11));
    assert_eq!(parameters.counter_length(), Some(12));
    assert_eq!(parameters.initial_counter_value(), Some(13));
    assert_eq!(parameters.salt_length(), Some(14));
    assert_eq!(parameters.mask_generator(), Some(0x8FFF_0001));
    assert_eq!(
        parameters.mask_generator_hashing_algorithm(),
        Some(0x8FFF_0002)
    );
    assert_eq!(
        parameters.with_p_source(|bytes| bytes.to_vec()),
        Some(vec![0x80, 0x01, 0xFE])
    );
    assert_eq!(parameters.trailer_field(), Some(18));
}

#[test]
fn parser_preserves_unknown_fields_and_future_enumerations_losslessly() {
    let extension_before = item(VENDOR_EXTENSION, Value::byte_string(vec![0x80, 0xAA]));
    let extension_after = item(
        VENDOR_EXTENSION,
        Value::text_string("vendor-tail".to_owned()),
    );
    let source = structure([
        extension_before,
        item(BLOCK_CIPHER_MODE, Value::enumeration(0x8ABC_DEF0)),
        item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(0x8ABC_DEF1)),
        extension_after,
    ]);

    let parameters = CryptographicParameters::try_from_ttlv(source)
        .expect("unknown members do not affect known-field order validation");

    assert_eq!(parameters.block_cipher_mode(), Some(0x8ABC_DEF0));
    assert_eq!(parameters.cryptographic_algorithm(), Some(0x8ABC_DEF1));
    let roundtrip = parameters.into_ttlv();
    let fields = roundtrip.view().children();
    assert_eq!(
        fields
            .iter()
            .map(|field| field.tag().raw())
            .collect::<Vec<_>>(),
        [
            VENDOR_EXTENSION,
            BLOCK_CIPHER_MODE,
            CRYPTOGRAPHIC_ALGORITHM,
            VENDOR_EXTENSION
        ]
    );
    assert!(matches!(
        fields[0].with_value(|value| value),
        ValueView::ByteString(bytes) if bytes == [0x80, 0xAA]
    ));
    assert!(matches!(
        fields[3].with_value(|value| value),
        ValueView::TextString(value) if value == "vendor-tail"
    ));
}

#[test]
fn wrong_known_member_type_returns_a_payload_free_error() {
    let sentinel = "CRYPTPARAMS_SECRET_SENTINEL";
    let source = structure([item(RANDOM_IV, Value::text_string(sentinel.to_owned()))]);

    let error = CryptographicParameters::try_from_ttlv(source)
        .expect_err("Random IV must use the Boolean Item Type");

    assert!(!format!("{error:?}").contains(sentinel));
    assert!(!format!("{error}").contains(sentinel));
}

#[test]
fn parser_rejects_duplicate_known_singletons() {
    let source = structure([
        item(IV_LENGTH, Value::integer(96)),
        item(IV_LENGTH, Value::integer(128)),
    ]);

    assert!(CryptographicParameters::try_from_ttlv(source).is_err());
}

#[test]
fn parser_rejects_known_members_outside_table_59_order() {
    let source = structure([
        item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(1)),
        item(PADDING_METHOD, Value::enumeration(1)),
    ]);

    assert!(CryptographicParameters::try_from_ttlv(source).is_err());
}

#[test]
fn builder_preserves_unknown_extension_order_after_known_members() {
    let parameters = CryptographicParametersBuilder::new()
        .with_extension(item(VENDOR_EXTENSION, Value::byte_string(vec![0x01, 0xFE])))
        .with_invocation_field_length(64)
        .with_padding_method(0x8ABC_0001)
        .build()
        .expect("known fields and vendor extension are retained");

    assert_eq!(
        item_tags(&parameters),
        [PADDING_METHOD, INVOCATION_FIELD_LENGTH, VENDOR_EXTENSION]
    );
}
