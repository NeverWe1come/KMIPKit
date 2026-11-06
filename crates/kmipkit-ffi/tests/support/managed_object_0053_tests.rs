use super::*;

const CERTIFICATE: u32 = 0x0042_0013;
const CERTIFICATE_REQUEST: u32 = 0x0042_0018;
const OPAQUE_OBJECT: u32 = 0x0042_005b;
const PGP_KEY: u32 = 0x0042_00bd;
const PRIVATE_KEY: u32 = 0x0042_0064;
const PUBLIC_KEY: u32 = 0x0042_006d;
const SECRET_DATA: u32 = 0x0042_0085;
const SPLIT_KEY: u32 = 0x0042_0089;
const SYMMETRIC_KEY: u32 = 0x0042_008f;
const KEY_BLOCK: u32 = 0x0042_0040;
const KEY_VALUE: u32 = 0x0042_0045;
const KEY_MATERIAL: u32 = 0x0042_0043;
const KEY_WRAPPING_DATA: u32 = 0x0042_0046;

#[link(name = "kmipkit_managed_object_parity_consumer", kind = "static")]
unsafe extern "C" {
    fn kmipkit_0053_managed_object_from_item(
        object_type: u32,
        item: *mut kmipkit_ttlv_item_t,
        out_item: *mut *mut kmipkit_ttlv_item_t,
    ) -> i32;
    fn kmipkit_0053_key_block_from_item(
        item: *mut kmipkit_ttlv_item_t,
        out_item: *mut *mut kmipkit_ttlv_item_t,
    ) -> i32;
    fn kmipkit_0053_key_value_from_item(
        format_raw: u32,
        item: *mut kmipkit_ttlv_item_t,
        out_item: *mut *mut kmipkit_ttlv_item_t,
    ) -> i32;
    fn kmipkit_0053_key_material_from_item(
        format_raw: u32,
        item: *mut kmipkit_ttlv_item_t,
        out_item: *mut *mut kmipkit_ttlv_item_t,
    ) -> i32;
    fn kmipkit_0053_key_wrapping_data_from_item(
        item: *mut kmipkit_ttlv_item_t,
        out_item: *mut *mut kmipkit_ttlv_item_t,
    ) -> i32;
}

fn tag(raw: u32) -> RawTag {
    RawTag::new(raw)
        .expect("fixture tag fits")
        .try_checked()
        .expect("fixture tag is allocated")
}

fn key_block() -> KeyBlock {
    KeyBlock::try_new(KeyFormatType::from_raw(1)).expect("Raw Key Format Type is valid")
}

fn all_objects() -> [(u32, Item); 9] {
    [
        (
            1,
            Certificate::try_new(0x8000_0001, vec![0xc1])
                .expect("certificate")
                .into_item(),
        ),
        (
            10,
            CertificateRequest::try_new(0x8000_0002, vec![0xc2])
                .expect("certificate request")
                .into_item(),
        ),
        (
            8,
            OpaqueObject::try_new(0x8000_0003, vec![0xc3])
                .expect("opaque object")
                .into_item(),
        ),
        (9, PgpKey::try_new(7, key_block()).expect("PGP key").into_item()),
        (
            4,
            PrivateKey::try_new(key_block())
                .expect("private key")
                .into_item(),
        ),
        (
            3,
            PublicKey::try_new(key_block())
                .expect("public key")
                .into_item(),
        ),
        (
            7,
            SecretData::try_new(0x8000_0004, key_block())
                .expect("secret data")
                .into_item(),
        ),
        (
            5,
            SplitKey::try_new(key_block(), 2, SplitKeyMethod::from_raw(1), 5, 3, None)
                .expect("split key")
                .into_item(),
        ),
        (
            2,
            SymmetricKey::try_new(key_block())
                .expect("symmetric key")
                .into_item(),
        ),
    ]
}

fn assert_copied_item(
    source: Item,
    convert: impl FnOnce(
        *mut kmipkit_ttlv_item_t,
        *mut *mut kmipkit_ttlv_item_t,
    ) -> i32,
    expected_tag: u32,
) {
    let source_handle = make_handle(Kind::Item, HandleValue::Item(source));
    let mut output = ptr::null_mut();
    let status = convert(source_handle, &raw mut output);
    assert_eq!(status, SUCCESS);
    let input = reference_handle(source_handle, Kind::Item).expect("source remains caller-owned");
    let result = reference_handle(output, Kind::Item).expect("result is an owned generic Item");
    let HandleValue::Item(input) = &input.value else {
        panic!("input is a generic Item")
    };
    let HandleValue::Item(result) = &result.value else {
        panic!("output is a generic Item")
    };
    assert_eq!(input.tag().raw(), expected_tag);
    assert_eq!(result.tag().raw(), expected_tag);
    assert_eq!(input.item_type(), result.item_type());
    kmipkit_ttlv_item_release(output);
    kmipkit_ttlv_item_release(source_handle);
}

#[test]
fn compiled_c_consumer_reads_all_nine_known_object_types() {
    let expected_tags = [
        CERTIFICATE,
        CERTIFICATE_REQUEST,
        OPAQUE_OBJECT,
        PGP_KEY,
        PRIVATE_KEY,
        PUBLIC_KEY,
        SECRET_DATA,
        SPLIT_KEY,
        SYMMETRIC_KEY,
    ];
    for ((object_type, item), expected_tag) in all_objects().into_iter().zip(expected_tags) {
        assert_copied_item(
            item,
            |source, output| {
                // SAFETY: Both handles are live test-owned handles and the output slot is valid.
                unsafe { kmipkit_0053_managed_object_from_item(object_type, source, output) }
            },
            expected_tag,
        );
    }
}

#[test]
fn compiled_c_consumer_roundtrips_typed_key_components_and_material() {
    let material_item = KeyMaterial::byte_string(1, b"secret-canary".to_vec())
        .expect("Raw format uses byte-string key material")
        .into_item();
    let key_value_item = KeyValue::try_new(
        KeyMaterial::byte_string(1, b"secret-canary".to_vec())
            .expect("Raw format uses byte-string key material"),
        None,
    )
    .expect("valid Key Value")
    .into_item();
    let key_value = KeyValue::try_new(
        KeyMaterial::byte_string(1, b"secret-canary".to_vec())
            .expect("Raw format uses byte-string key material"),
        None,
    )
    .expect("valid Key Value for Key Block");
    let key_block = KeyBlock::try_new(KeyFormatType::from_raw(1))
        .expect("Raw Key Format Type is valid")
        .with_key_value(KeyBlockValue::Plaintext(key_value));
    let key_block_item = key_block.into_item().expect("valid Key Block");

    assert_copied_item(
        material_item,
        |source, output| {
            // SAFETY: Both handles are live test-owned handles and the output slot is valid.
            unsafe { kmipkit_0053_key_material_from_item(1, source, output) }
        },
        KEY_MATERIAL,
    );
    assert_copied_item(
        key_value_item,
        |source, output| {
            // SAFETY: Both handles are live test-owned handles and the output slot is valid.
            unsafe { kmipkit_0053_key_value_from_item(1, source, output) }
        },
        KEY_VALUE,
    );
    assert_copied_item(
        key_block_item,
        |source, output| {
            // SAFETY: Both handles are live test-owned handles and the output slot is valid.
            unsafe { kmipkit_0053_key_block_from_item(source, output) }
        },
        KEY_BLOCK,
    );
}

#[test]
fn compiled_c_consumer_reads_wrapping_data_and_rejects_wrong_types() {
    let wrapping = KeyWrappingData::try_new(1)
        .expect("open wrapping method")
        .with_encryption_key_information(
            EncryptionKeyInformation::try_new("wrapping-key".to_owned())
                .expect("non-empty identifier"),
        )
        .into_item()
        .expect("valid wrapping data");
    assert_copied_item(
        wrapping,
        |source, output| {
            // SAFETY: Both handles are live test-owned handles and the output slot is valid.
            unsafe { kmipkit_0053_key_wrapping_data_from_item(source, output) }
        },
        KEY_WRAPPING_DATA,
    );

    let wrong = Item::new(tag(KEY_MATERIAL), Value::text_string("not bytes".to_owned()))
        .expect("valid but wrong typed material");
    let source = make_handle(Kind::Item, HandleValue::Item(wrong));
    let mut output = ptr::null_mut();
    // SAFETY: Both handles are live test-owned handles and the output slot is valid.
    let status = unsafe { kmipkit_0053_key_material_from_item(1, source, &raw mut output) };
    assert_eq!(status, ERROR_INVALID_INPUT);
    assert!(output.is_null());
    assert!(reference_handle(source, Kind::Item).is_ok());
    kmipkit_ttlv_item_release(source);
}

#[test]
fn managed_object_unknown_fallback_and_secret_debug_are_safe() {
    let mut structure = Structure::new();
    structure
        .try_push(
            Item::new(tag(0x0054_0001), Value::byte_string(b"secret-canary".to_vec()))
                .expect("valid vendor extension"),
        )
        .expect("extension fits in structure");
    let item = Item::new(tag(0x0054_0002), Value::structure(structure))
        .expect("valid vendor object");
    let source = make_handle(Kind::Item, HandleValue::Item(item));
    let mut output = ptr::null_mut();
    // SAFETY: Both handles are live test-owned handles and the output slot is valid.
    let status = unsafe {
        kmipkit_0053_managed_object_from_item(0x8000_0001, source, &raw mut output)
    };
    assert_eq!(status, SUCCESS);
    let result = reference_handle(output, Kind::Item).expect("unknown object stays generic");
    let HandleValue::Item(item) = &result.value else {
        panic!("output is an Item")
    };
    assert_eq!(item.tag().raw(), 0x0054_0002);
    assert!(!format!("{item:?}").contains("secret-canary"));
    kmipkit_ttlv_item_release(output);
    kmipkit_ttlv_item_release(source);
}
