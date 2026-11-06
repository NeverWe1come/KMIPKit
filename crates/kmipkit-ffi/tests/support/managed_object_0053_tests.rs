use super::*;
use kmipkit_protocol::{
    Certificate, CertificateRequest, EncryptionKeyInformation, KeyBlockValue, KeyMaterial,
    OpaqueObject, PgpKey, PrivateKey, PublicKey, SecretData, SplitKey, SymmetricKey,
};

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
const FIELD_OWNER_MANAGED_OBJECT: u32 = 1;
const FIELD_OWNER_KEY_BLOCK: u32 = 2;
const FIELD_OWNER_KEY_MATERIAL: u32 = 4;

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
    fn kmipkit_0053_managed_object_from_fields(
        object_type: u32,
        fields: *mut *mut kmipkit_ttlv_item_t,
        field_count: u64,
        out_item: *mut *mut kmipkit_ttlv_item_t,
    ) -> i32;
    fn kmipkit_0053_key_material_from_fields(
        format_raw: u32,
        fields: *mut *mut kmipkit_ttlv_item_t,
        field_count: u64,
        out_item: *mut *mut kmipkit_ttlv_item_t,
    ) -> i32;
    fn kmipkit_0053_key_block_from_fields(
        fields: *mut *mut kmipkit_ttlv_item_t,
        field_count: u64,
        out_item: *mut *mut kmipkit_ttlv_item_t,
    ) -> i32;
    fn kmipkit_0053_key_value_from_fields(
        fields: *mut *mut kmipkit_ttlv_item_t,
        field_count: u64,
        out_item: *mut *mut kmipkit_ttlv_item_t,
    ) -> i32;
    fn kmipkit_0053_key_material_from_fields_default(
        fields: *mut *mut kmipkit_ttlv_item_t,
        field_count: u64,
        out_item: *mut *mut kmipkit_ttlv_item_t,
    ) -> i32;
    fn kmipkit_0053_key_wrapping_data_from_fields(
        fields: *mut *mut kmipkit_ttlv_item_t,
        field_count: u64,
        out_item: *mut *mut kmipkit_ttlv_item_t,
    ) -> i32;
    fn kmipkit_0053_certificate_from_fields(
        fields: *mut *mut kmipkit_ttlv_item_t,
        field_count: u64,
        out_item: *mut *mut kmipkit_ttlv_item_t,
    ) -> i32;
    fn kmipkit_0053_symmetric_key_from_fields(
        fields: *mut *mut kmipkit_ttlv_item_t,
        field_count: u64,
        out_item: *mut *mut kmipkit_ttlv_item_t,
    ) -> i32;
    fn kmipkit_0053_public_key_from_fields(
        fields: *mut *mut kmipkit_ttlv_item_t,
        field_count: u64,
        out_item: *mut *mut kmipkit_ttlv_item_t,
    ) -> i32;
    fn kmipkit_0053_private_key_from_fields(
        fields: *mut *mut kmipkit_ttlv_item_t,
        field_count: u64,
        out_item: *mut *mut kmipkit_ttlv_item_t,
    ) -> i32;
    fn kmipkit_0053_split_key_from_fields(
        fields: *mut *mut kmipkit_ttlv_item_t,
        field_count: u64,
        out_item: *mut *mut kmipkit_ttlv_item_t,
    ) -> i32;
    fn kmipkit_0053_secret_data_from_fields(
        fields: *mut *mut kmipkit_ttlv_item_t,
        field_count: u64,
        out_item: *mut *mut kmipkit_ttlv_item_t,
    ) -> i32;
    fn kmipkit_0053_opaque_object_from_fields(
        fields: *mut *mut kmipkit_ttlv_item_t,
        field_count: u64,
        out_item: *mut *mut kmipkit_ttlv_item_t,
    ) -> i32;
    fn kmipkit_0053_pgp_key_from_fields(
        fields: *mut *mut kmipkit_ttlv_item_t,
        field_count: u64,
        out_item: *mut *mut kmipkit_ttlv_item_t,
    ) -> i32;
    fn kmipkit_0053_certificate_request_from_fields(
        fields: *mut *mut kmipkit_ttlv_item_t,
        field_count: u64,
        out_item: *mut *mut kmipkit_ttlv_item_t,
    ) -> i32;
    fn kmipkit_0053_certificate_roundtrip(
        fields: *mut *mut kmipkit_ttlv_item_t,
        field_count: u64,
    ) -> i32;
    fn kmipkit_0053_managed_field_reader_roundtrip(
        fields: *mut *mut kmipkit_ttlv_item_t,
        field_count: u64,
    ) -> i32;
    fn kmipkit_0053_managed_field_reader_check(
        owner_kind: u32,
        context_raw: u32,
        item: *mut kmipkit_ttlv_item_t,
        field_tag: u32,
        read_kind: u32,
        expected_item_type: u8,
        expected_enumeration: u32,
        expected_integer: i32,
        expected_bytes: *const u8,
        expected_byte_length: u64,
        expected_status: i32,
    ) -> i32;
    fn kmipkit_0053_managed_field_reader_rejects_short_buffer(
        owner_kind: u32,
        context_raw: u32,
        item: *mut kmipkit_ttlv_item_t,
        field_tag: u32,
        expected_byte_length: u64,
    ) -> i32;
    fn kmipkit_0053_managed_nested_nodes_roundtrip(
        key_block: *mut kmipkit_ttlv_item_t,
        wrapped_key_block: *mut kmipkit_ttlv_item_t,
    ) -> i32;
}

fn tag(raw: u32) -> kmipkit_ttlv::Tag {
    RawTag::new(raw)
        .expect("fixture tag fits")
        .try_checked()
        .expect("fixture tag is allocated")
}

fn key_block() -> KeyBlock {
    KeyBlock::try_new(KeyFormatType::from_raw(1)).expect("Raw Key Format Type is valid")
}

fn transparent_material_item(format_raw: u32, fields: impl IntoIterator<Item = Item>) -> Item {
    let mut structure = Structure::new();
    for field in fields {
        structure.try_push(field).expect("transparent field fits");
    }
    KeyMaterial::structure(format_raw, structure)
        .expect("transparent material matches its model")
        .into_item()
}

fn structure_item_with_fields(raw_tag: u32, fields: impl IntoIterator<Item = Item>) -> Item {
    let mut structure = Structure::new();
    for field in fields {
        structure.try_push(field).expect("nested field fits");
    }
    Item::new(tag(raw_tag), Value::structure(structure)).expect("nested structure Item")
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
        (
            9,
            PgpKey::try_new(7, key_block())
                .expect("PGP key")
                .into_item(),
        ),
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
    convert: impl FnOnce(*mut kmipkit_ttlv_item_t, *mut *mut kmipkit_ttlv_item_t) -> i32,
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

fn field_handle(raw_tag: u32, value: Value) -> *mut kmipkit_ttlv_item_t {
    make_handle(
        Kind::Item,
        HandleValue::Item(Item::new(tag(raw_tag), value).expect("valid fixture Item")),
    )
}

fn assert_compiled_c_managed_field_reader(
    item: Item,
    owner_kind: u32,
    context_raw: u32,
    field_tag: u32,
    read_kind: u32,
    expected_item_type: u8,
    expected_enumeration: u32,
    expected_integer: i32,
    expected_bytes: &[u8],
    expected_status: i32,
) {
    let item_handle = make_handle(Kind::Item, HandleValue::Item(item));
    let expected_bytes_ptr = if expected_bytes.is_empty() {
        ptr::null()
    } else {
        expected_bytes.as_ptr()
    };
    // SAFETY: The Item handle remains live for the C call and expected_bytes is a valid slice.
    let status = unsafe {
        kmipkit_0053_managed_field_reader_check(
            owner_kind,
            context_raw,
            item_handle,
            field_tag,
            read_kind,
            expected_item_type,
            expected_enumeration,
            expected_integer,
            expected_bytes_ptr,
            expected_bytes.len() as u64,
            expected_status,
        )
    };
    assert_eq!(
        status, 0,
        "owner={owner_kind} context={context_raw} field={field_tag:#x} read_kind={read_kind}"
    );
    kmipkit_ttlv_item_release(item_handle);
}

type FieldConstructor =
    unsafe extern "C" fn(*mut *mut kmipkit_ttlv_item_t, u64, *mut *mut kmipkit_ttlv_item_t) -> i32;

fn assert_compiled_c_field_constructor(
    constructor: FieldConstructor,
    mut fields: Vec<*mut kmipkit_ttlv_item_t>,
    expected_tag: u32,
) {
    let mut output_handle = ptr::null_mut();
    // SAFETY: Every field is a live Item handle and both the pointer array and output slot are valid.
    let status = unsafe {
        constructor(
            fields.as_mut_ptr(),
            fields.len() as u64,
            &raw mut output_handle,
        )
    };
    assert_eq!(status, SUCCESS);
    let output = reference_handle(output_handle, Kind::Item).expect("typed structure output");
    let HandleValue::Item(item) = &output.value else {
        panic!("typed structure output is an Item")
    };
    assert_eq!(item.tag().raw(), expected_tag);
    assert!(
        fields
            .iter()
            .all(|field| reference_handle(*field, Kind::Item).is_ok())
    );
    kmipkit_ttlv_item_release(output_handle);
    for field in fields {
        kmipkit_ttlv_item_release(field);
    }
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
    let object_fields = [
        (0x0042_001d, 3, 5, 0x8000_0001, 0),
        (0x0042_0019, 3, 5, 0x8000_0002, 0),
        (0x0042_0059, 3, 5, 0x8000_0003, 0),
        (0x0042_00be, 4, 2, 0, 7),
        (KEY_BLOCK, 1, 0, 1, 0),
        (KEY_BLOCK, 1, 0, 1, 0),
        (0x0042_0086, 3, 5, 0x8000_0004, 0),
        (0x0042_008b, 4, 2, 0, 5),
        (KEY_BLOCK, 1, 0, 1, 0),
    ];
    for (
        ((object_type, item), expected_tag),
        (field_tag, read_kind, item_type, enum_value, integer),
    ) in all_objects()
        .into_iter()
        .zip(expected_tags)
        .zip(object_fields)
    {
        assert_compiled_c_managed_field_reader(
            item.try_clone().expect("preserve object for reader test"),
            FIELD_OWNER_MANAGED_OBJECT,
            object_type,
            field_tag,
            read_kind,
            item_type,
            enum_value,
            integer,
            &[],
            SUCCESS,
        );
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
fn compiled_c_consumer_reads_transparent_fields_for_assigned_key_formats() {
    let cases = [
        (
            7,
            0x0042_003f,
            5,
            vec![
                Item::new(tag(0x0042_003f), Value::byte_string(b"sym".to_vec()))
                    .expect("symmetric key field"),
            ],
            b"sym".as_slice(),
        ),
        (
            8,
            0x0042_009f,
            5,
            vec![
                Item::new(tag(0x0042_005e), Value::big_integer(vec![1])).expect("DSA P"),
                Item::new(tag(0x0042_0071), Value::big_integer(vec![2])).expect("DSA Q"),
                Item::new(tag(0x0042_0037), Value::big_integer(vec![3])).expect("DSA G"),
                Item::new(tag(0x0042_009f), Value::big_integer(vec![4])).expect("DSA X"),
            ],
            &[4][..],
        ),
        (
            9,
            0x0042_00a0,
            5,
            vec![
                Item::new(tag(0x0042_005e), Value::big_integer(vec![1])).expect("DSA P"),
                Item::new(tag(0x0042_0071), Value::big_integer(vec![2])).expect("DSA Q"),
                Item::new(tag(0x0042_0037), Value::big_integer(vec![3])).expect("DSA G"),
                Item::new(tag(0x0042_00a0), Value::big_integer(vec![5])).expect("DSA Y"),
            ],
            &[5][..],
        ),
        (
            10,
            0x0042_0063,
            5,
            vec![
                Item::new(tag(0x0042_0052), Value::big_integer(vec![6])).expect("RSA modulus"),
                Item::new(tag(0x0042_0063), Value::big_integer(vec![7]))
                    .expect("RSA private exponent"),
            ],
            &[7][..],
        ),
        (
            11,
            0x0042_006c,
            5,
            vec![
                Item::new(tag(0x0042_0052), Value::big_integer(vec![8])).expect("RSA modulus"),
                Item::new(tag(0x0042_006c), Value::big_integer(vec![9]))
                    .expect("RSA public exponent"),
            ],
            &[9][..],
        ),
        (
            12,
            0x0042_009f,
            5,
            vec![
                Item::new(tag(0x0042_005e), Value::big_integer(vec![1])).expect("DH P"),
                Item::new(tag(0x0042_0037), Value::big_integer(vec![2])).expect("DH G"),
                Item::new(tag(0x0042_009f), Value::big_integer(vec![3])).expect("DH X"),
            ],
            &[3][..],
        ),
        (
            13,
            0x0042_00a0,
            5,
            vec![
                Item::new(tag(0x0042_005e), Value::big_integer(vec![1])).expect("DH P"),
                Item::new(tag(0x0042_0037), Value::big_integer(vec![2])).expect("DH G"),
                Item::new(tag(0x0042_00a0), Value::big_integer(vec![4])).expect("DH Y"),
            ],
            &[4][..],
        ),
        (
            0x14,
            0x0042_0075,
            3,
            vec![
                Item::new(tag(0x0042_0075), Value::enumeration(0x8000_0014))
                    .expect("open curve enumeration"),
                Item::new(tag(0x0042_002e), Value::big_integer(vec![5])).expect("EC D"),
            ],
            &[][..],
        ),
        (
            0x15,
            0x0042_0072,
            5,
            vec![
                Item::new(tag(0x0042_0075), Value::enumeration(0x8000_0015))
                    .expect("open curve enumeration"),
                Item::new(tag(0x0042_0072), Value::byte_string(b"point".to_vec()))
                    .expect("EC Q String"),
            ],
            b"point".as_slice(),
        ),
    ];

    for (format_raw, field_tag, read_kind, fields, expected_bytes) in cases {
        let material = transparent_material_item(format_raw, fields);
        let (item_type, expected_enumeration) = match (format_raw, read_kind) {
            (0x14, 3) => (5, 0x8000_0014),
            (7 | 0x15, 5) => (8, 0),
            (_, 5) => (4, 0),
            _ => (0, 0),
        };
        assert_compiled_c_managed_field_reader(
            material
                .try_clone()
                .expect("preserve material for mismatch checks"),
            FIELD_OWNER_KEY_MATERIAL,
            format_raw,
            field_tag,
            read_kind,
            item_type,
            expected_enumeration,
            0,
            expected_bytes,
            SUCCESS,
        );
        if format_raw == 11 {
            assert_compiled_c_managed_field_reader(
                material.try_clone().expect("preserve RSA material"),
                FIELD_OWNER_KEY_MATERIAL,
                8,
                field_tag,
                read_kind,
                item_type,
                expected_enumeration,
                0,
                expected_bytes,
                -1,
            );
            assert_compiled_c_managed_field_reader(
                material.try_clone().expect("preserve RSA material"),
                FIELD_OWNER_KEY_MATERIAL,
                format_raw,
                field_tag,
                3,
                item_type,
                expected_enumeration,
                0,
                &[],
                -1,
            );
        }
        if !expected_bytes.is_empty() {
            let item_handle = make_handle(Kind::Item, HandleValue::Item(material));
            // SAFETY: The live Item handle is borrowed only for the C reader call.
            let status = unsafe {
                kmipkit_0053_managed_field_reader_rejects_short_buffer(
                    FIELD_OWNER_KEY_MATERIAL,
                    format_raw,
                    item_handle,
                    field_tag,
                    expected_bytes.len() as u64,
                )
            };
            assert_eq!(status, 0);
            kmipkit_ttlv_item_release(item_handle);
        }
    }

    let key_block = KeyBlock::try_new(KeyFormatType::from_raw(1))
        .expect("Key Block builder")
        .try_into_item()
        .expect("minimal Key Block Item");
    assert_compiled_c_managed_field_reader(
        key_block,
        FIELD_OWNER_KEY_BLOCK,
        0,
        KEY_WRAPPING_DATA,
        1,
        0,
        0,
        0,
        &[],
        SUCCESS,
    );
}

#[test]
fn compiled_c_consumer_builds_each_named_object_from_typed_fields() {
    let key_block_field = || {
        make_handle(
            Kind::Item,
            HandleValue::Item(key_block().try_into_item().expect("Key Block Item")),
        )
    };
    let cases: [(FieldConstructor, u32, Vec<*mut kmipkit_ttlv_item_t>); 9] = [
        (
            kmipkit_0053_certificate_from_fields,
            CERTIFICATE,
            vec![
                field_handle(0x0042_001e, Value::byte_string(b"cert".to_vec())),
                field_handle(0x0042_001d, Value::enumeration(0x8000_0042)),
            ],
        ),
        (
            kmipkit_0053_symmetric_key_from_fields,
            SYMMETRIC_KEY,
            vec![key_block_field()],
        ),
        (
            kmipkit_0053_public_key_from_fields,
            PUBLIC_KEY,
            vec![key_block_field()],
        ),
        (
            kmipkit_0053_private_key_from_fields,
            PRIVATE_KEY,
            vec![key_block_field()],
        ),
        (
            kmipkit_0053_split_key_from_fields,
            SPLIT_KEY,
            vec![
                field_handle(0x0042_008c, Value::integer(2)),
                field_handle(0x0042_0044, Value::integer(1)),
                field_handle(0x0042_008a, Value::enumeration(1)),
                field_handle(0x0042_008b, Value::integer(2)),
                key_block_field(),
            ],
        ),
        (
            kmipkit_0053_secret_data_from_fields,
            SECRET_DATA,
            vec![
                key_block_field(),
                field_handle(0x0042_0086, Value::enumeration(3)),
            ],
        ),
        (
            kmipkit_0053_opaque_object_from_fields,
            OPAQUE_OBJECT,
            vec![
                field_handle(0x0042_005a, Value::byte_string(b"opaque".to_vec())),
                field_handle(0x0042_0059, Value::enumeration(9)),
            ],
        ),
        (
            kmipkit_0053_pgp_key_from_fields,
            PGP_KEY,
            vec![
                key_block_field(),
                field_handle(0x0042_00be, Value::integer(7)),
            ],
        ),
        (
            kmipkit_0053_certificate_request_from_fields,
            CERTIFICATE_REQUEST,
            vec![
                field_handle(0x0042_0140, Value::byte_string(b"request".to_vec())),
                field_handle(0x0042_0019, Value::enumeration(4)),
            ],
        ),
    ];
    for (constructor, expected_tag, mut fields) in cases {
        assert_compiled_c_field_constructor(constructor, std::mem::take(&mut fields), expected_tag);
    }
}

#[test]
fn compiled_c_consumer_reads_named_certificate_fields_by_ttlv_type() {
    let mut fields = vec![
        field_handle(0x0042_001e, Value::byte_string(b"cert".to_vec())),
        field_handle(0x0042_001d, Value::enumeration(0x8000_0042)),
    ];
    // SAFETY: The C consumer reads the live borrowed field handles and releases all views and its owned output.
    let status =
        unsafe { kmipkit_0053_certificate_roundtrip(fields.as_mut_ptr(), fields.len() as u64) };
    assert_eq!(status, SUCCESS);
    assert!(
        fields
            .iter()
            .all(|field| reference_handle(*field, Kind::Item).is_ok())
    );
    for field in fields {
        kmipkit_ttlv_item_release(field);
    }
}

#[test]
fn compiled_c_consumer_reads_typed_managed_fields_by_owner_and_tag() {
    let mut fields = vec![
        field_handle(0x0042_001e, Value::byte_string(b"cert".to_vec())),
        field_handle(0x0042_001d, Value::enumeration(0x8000_0042)),
    ];
    // SAFETY: The C consumer reads the borrowed fields, constructs/releases an owned result, and copies bytes into its local buffer.
    let status = unsafe {
        kmipkit_0053_managed_field_reader_roundtrip(fields.as_mut_ptr(), fields.len() as u64)
    };
    assert_eq!(status, SUCCESS);
    assert!(
        fields
            .iter()
            .all(|field| reference_handle(*field, Kind::Item).is_ok())
    );
    for field in fields {
        kmipkit_ttlv_item_release(field);
    }

    let key_block = KeyBlock::try_new(KeyFormatType::from_raw(11))
        .expect("open Key Format Type")
        .try_into_item()
        .expect("Key Block Item");
    assert_compiled_c_managed_field_reader(
        key_block,
        FIELD_OWNER_KEY_BLOCK,
        0,
        0x0042_0042,
        3,
        5,
        11,
        0,
        &[],
        SUCCESS,
    );

    let key_value = KeyValue::try_new(
        KeyMaterial::byte_string(1, b"key".to_vec()).expect("raw Key Material"),
        None,
    )
    .expect("Key Value")
    .into_item();
    assert_compiled_c_managed_field_reader(key_value, 3, 1, KEY_MATERIAL, 2, 8, 0, 0, &[], SUCCESS);

    let wrapping = KeyWrappingData::try_new(0x8000_0001)
        .expect("open wrapping method")
        .with_encryption_key_information(
            EncryptionKeyInformation::try_new("wrapping-key".to_owned()).expect("key information"),
        )
        .into_item()
        .expect("Key Wrapping Data Item");
    assert_compiled_c_managed_field_reader(
        wrapping,
        5,
        0,
        0x0042_009e,
        3,
        5,
        0x8000_0001,
        0,
        &[],
        SUCCESS,
    );
}

#[test]
fn compiled_c_consumer_navigates_owned_nested_key_nodes_and_vendor_fields() {
    let key_material = KeyMaterial::byte_string(1, b"key-data".to_vec())
        .expect("raw Key Material")
        .into_item();
    let attributes = structure_item_with_fields(
        0x0042_0125,
        [
            Item::new(tag(0x0054_0031), Value::byte_string(b"vendor-one".to_vec()))
                .expect("first vendor attribute"),
            Item::new(tag(0x0054_0031), Value::byte_string(b"vendor-two".to_vec()))
                .expect("repeated vendor attribute"),
        ],
    );
    let key_value = structure_item_with_fields(0x0042_0045, [key_material, attributes]);
    let key_block = structure_item_with_fields(
        0x0042_0040,
        [
            Item::new(tag(0x0042_0042), Value::enumeration(1)).expect("Key Format Type"),
            key_value,
        ],
    );
    let key_block = KeyBlock::try_from_item(key_block)
        .expect("valid plaintext Key Block")
        .try_into_item()
        .expect("Key Block Item");

    let key_information = |outer_tag: u32, unique_id: &[u8]| {
        structure_item_with_fields(
            outer_tag,
            [
                Item::new(
                    tag(0x0042_0094),
                    Value::text_string(String::from_utf8(unique_id.to_vec()).expect("UTF-8 id")),
                )
                .expect("Unique Identifier"),
                Item::new(tag(0x0054_0031), Value::byte_string(b"vendor-one".to_vec()))
                    .expect("first vendor child"),
                Item::new(tag(0x0054_0031), Value::byte_string(b"vendor-two".to_vec()))
                    .expect("repeated vendor child"),
            ],
        )
    };
    let wrapping_data = structure_item_with_fields(
        0x0042_0046,
        [
            Item::new(tag(0x0042_009e), Value::enumeration(0x8000_0020))
                .expect("open Wrapping Method"),
            key_information(0x0042_0036, b"encryption-id"),
            key_information(0x0042_004e, b"mac-id"),
        ],
    );
    let wrapping_data = KeyWrappingData::try_from_item(wrapping_data)
        .expect("valid wrapping data")
        .into_item()
        .expect("Key Wrapping Data Item");

    let wrapped_key_block = structure_item_with_fields(
        0x0042_0040,
        [
            Item::new(tag(0x0042_0042), Value::enumeration(1)).expect("Key Format Type"),
            Item::new(tag(0x0042_0045), Value::byte_string(b"wrapped".to_vec()))
                .expect("wrapped Key Value"),
            wrapping_data
                .try_clone()
                .expect("preserve wrapping data for wrapped Key Block"),
        ],
    );
    let wrapped_key_block = KeyBlock::try_from_item(wrapped_key_block)
        .expect("valid wrapped Key Block")
        .try_into_item()
        .expect("wrapped Key Block Item");

    let key_block_handle = make_handle(Kind::Item, HandleValue::Item(key_block));
    let wrapped_block_handle = make_handle(Kind::Item, HandleValue::Item(wrapped_key_block));
    // SAFETY: All parent handles remain live during the C traversal. The C consumer releases each returned owned child Item.
    let status = unsafe {
        kmipkit_0053_managed_nested_nodes_roundtrip(key_block_handle, wrapped_block_handle)
    };
    assert_eq!(status, SUCCESS);
    for handle in [key_block_handle, wrapped_block_handle] {
        assert!(reference_handle(handle, Kind::Item).is_ok());
        kmipkit_ttlv_item_release(handle);
    }
}

#[test]
fn compiled_c_consumer_builds_key_block_value_and_wrapping_fields() {
    assert_compiled_c_field_constructor(
        kmipkit_0053_key_value_from_fields,
        vec![field_handle(
            KEY_MATERIAL,
            Value::byte_string(b"secret-canary".to_vec()),
        )],
        KEY_VALUE,
    );
    assert_compiled_c_field_constructor(
        kmipkit_0053_key_material_from_fields_default,
        vec![field_handle(
            KEY_MATERIAL,
            Value::byte_string(b"secret-canary".to_vec()),
        )],
        KEY_MATERIAL,
    );
    let key_value = KeyValue::try_new(
        KeyMaterial::byte_string(1, b"secret-canary".to_vec()).expect("raw key material"),
        None,
    )
    .expect("Key Value")
    .into_item();
    assert_compiled_c_field_constructor(
        kmipkit_0053_key_block_from_fields,
        vec![
            make_handle(Kind::Item, HandleValue::Item(key_value)),
            field_handle(0x0042_0042, Value::enumeration(1)),
        ],
        KEY_BLOCK,
    );
    let encryption_info = EncryptionKeyInformation::try_new("wrapping-key".to_owned())
        .expect("encryption key information")
        .into_item();
    assert_compiled_c_field_constructor(
        kmipkit_0053_key_wrapping_data_from_fields,
        vec![
            make_handle(Kind::Item, HandleValue::Item(encryption_info)),
            field_handle(0x0042_009e, Value::enumeration(1)),
        ],
        KEY_WRAPPING_DATA,
    );
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
    let key_block_item = key_block.try_into_item().expect("valid Key Block");

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

    let wrong = Item::new(
        tag(KEY_MATERIAL),
        Value::text_string("not bytes".to_owned()),
    )
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
            Item::new(
                tag(0x0054_0001),
                Value::byte_string(b"secret-canary".to_vec()),
            )
            .expect("valid vendor extension"),
        )
        .expect("extension fits in structure");
    let item =
        Item::new(tag(0x0054_0002), Value::structure(structure)).expect("valid vendor object");
    let source = make_handle(Kind::Item, HandleValue::Item(item));
    let mut output = ptr::null_mut();
    // SAFETY: Both handles are live test-owned handles and the output slot is valid.
    let status =
        unsafe { kmipkit_0053_managed_object_from_item(0x8000_0001, source, &raw mut output) };
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

#[test]
fn compiled_c_consumer_builds_certificate_in_schema_order_and_reads_field_values() {
    let certificate_value = Item::new(
        tag(0x0042_001e),
        Value::byte_string(b"certificate-canary".to_vec()),
    )
    .expect("certificate value");
    let unknown_first = Item::new(tag(0x0054_0010), Value::byte_string(b"vendor-one".to_vec()))
        .expect("vendor field");
    let certificate_type =
        Item::new(tag(0x0042_001d), Value::enumeration(0x8000_0042)).expect("certificate type");
    let unknown_repeat = Item::new(tag(0x0054_0010), Value::byte_string(b"vendor-two".to_vec()))
        .expect("repeated vendor field");
    let mut source_handles = [
        make_handle(Kind::Item, HandleValue::Item(certificate_value)),
        make_handle(Kind::Item, HandleValue::Item(unknown_first)),
        make_handle(Kind::Item, HandleValue::Item(certificate_type)),
        make_handle(Kind::Item, HandleValue::Item(unknown_repeat)),
    ];
    let mut output_handle = ptr::null_mut();
    // SAFETY: All input handles are live, the pointer array spans four entries, and output is writable.
    let status = unsafe {
        kmipkit_0053_managed_object_from_fields(
            1,
            source_handles.as_mut_ptr(),
            source_handles.len() as u64,
            &raw mut output_handle,
        )
    };
    assert_eq!(status, SUCCESS);
    let output = reference_handle(output_handle, Kind::Item).expect("built certificate");
    let HandleValue::Item(item) = &output.value else {
        panic!("output is an Item")
    };
    item.with_value(|value| {
        let ValueView::Structure(structure) = value else {
            panic!("certificate is a Structure")
        };
        assert_eq!(
            structure
                .children()
                .iter()
                .map(|field| field.tag().raw())
                .collect::<Vec<_>>(),
            [0x0042_001d, 0x0054_0010, 0x0042_001e, 0x0054_0010]
        );
        assert_eq!(
            structure.children()[0].with_value(|value| match value {
                ValueView::Enumeration(raw) => *raw,
                _ => 0,
            }),
            0x8000_0042
        );
        assert!(structure.children()[2].with_value(|value| match value {
            ValueView::ByteString(bytes) => bytes == b"certificate-canary",
            _ => false,
        }));
    });
    assert!(
        source_handles
            .iter()
            .all(|source| reference_handle(*source, Kind::Item).is_ok())
    );
    kmipkit_ttlv_item_release(output_handle);
    for source in source_handles {
        kmipkit_ttlv_item_release(source);
    }
}

#[test]
fn compiled_c_consumer_uses_oasis_hexadecimal_transparent_ec_format_values() {
    for format_raw in [0x14, 0x15] {
        let mut output = ptr::null_mut();
        // SAFETY: A null field array is valid with a zero count; output is a writable slot.
        let status = unsafe {
            kmipkit_0053_key_material_from_fields(format_raw, ptr::null_mut(), 0, &raw mut output)
        };
        assert_eq!(status, ERROR_INVALID_INPUT);
        assert!(output.is_null());
    }

    for reserved_format in [14, 15] {
        let mut output = ptr::null_mut();
        // SAFETY: A null field array is valid with a zero count; output is a writable slot.
        let status = unsafe {
            kmipkit_0053_key_material_from_fields(
                reserved_format,
                ptr::null_mut(),
                0,
                &raw mut output,
            )
        };
        assert_eq!(status, SUCCESS);
        assert!(!output.is_null());
        kmipkit_ttlv_item_release(output);
    }
}

#[test]
fn ffi_builder_dispatches_ec_formats_at_oasis_hex_values_and_preserves_reserved_values() {
    let ec_cases = [
        (
            0x14,
            vec![
                Item::new(tag(0x0042_0075), Value::enumeration(0x8000_0014))
                    .expect("EC recommended curve"),
                Item::new(tag(0x0042_002e), Value::big_integer(vec![0x21]))
                    .expect("EC private scalar"),
            ],
        ),
        (
            0x15,
            vec![
                Item::new(tag(0x0042_0075), Value::enumeration(0x8000_0015))
                    .expect("EC recommended curve"),
                Item::new(tag(0x0042_0072), Value::byte_string(vec![0x04, 0x22]))
                    .expect("EC public point"),
            ],
        ),
    ];

    for (format, fields) in ec_cases {
        let mut handles: Vec<*mut kmipkit_ttlv_item_t> = fields
            .into_iter()
            .map(|item| make_handle(Kind::Item, HandleValue::Item(item)))
            .collect();
        let mut output = ptr::null_mut();
        // SAFETY: Every field handle is live and the output slot is writable.
        // SAFETY: Every field handle is live and the output slot is writable.
        let status = unsafe {
            kmipkit_0053_key_material_from_fields(
                format,
                handles.as_mut_ptr(),
                handles.len() as u64,
                &raw mut output,
            )
        };
        assert_eq!(status, SUCCESS, "assigned EC format {format:#x}");
        assert!(!output.is_null());
        kmipkit_ttlv_item_release(output);
        for handle in handles {
            kmipkit_ttlv_item_release(handle);
        }
    }

    for reserved_format in [14, 15] {
        let vendor_item = Item::new(tag(0x54ff_01), Value::byte_string(vec![0x31, 0x32]))
            .expect("vendor extension item");
        let vendor_handle = make_handle(Kind::Item, HandleValue::Item(vendor_item));
        let mut handles = [vendor_handle];
        let mut output = ptr::null_mut();
        // SAFETY: The extension handle is live and the output slot is writable.
        // SAFETY: The vendor field handle is live and the output slot is writable.
        let status = unsafe {
            kmipkit_0053_key_material_from_fields(
                reserved_format,
                handles.as_mut_ptr(),
                handles.len() as u64,
                &raw mut output,
            )
        };
        assert_eq!(
            status, SUCCESS,
            "reserved format {reserved_format:#x} remains a generic structure"
        );
        assert!(!output.is_null());
        kmipkit_ttlv_item_release(output);
        kmipkit_ttlv_item_release(vendor_handle);
    }
}

#[test]
fn compiled_c_consumer_builds_transparent_rsa_public_material_in_schema_order() {
    let exponent =
        Item::new(tag(0x0042_006c), Value::big_integer(vec![1, 0, 1])).expect("public exponent");
    let modulus =
        Item::new(tag(0x0042_0052), Value::big_integer(vec![0x80, 0x01])).expect("modulus");
    let mut source_handles = [
        make_handle(Kind::Item, HandleValue::Item(exponent)),
        make_handle(Kind::Item, HandleValue::Item(modulus)),
    ];
    let mut output_handle = ptr::null_mut();
    // SAFETY: Both input handles are live and the pointer array and output slot are valid.
    let status = unsafe {
        kmipkit_0053_key_material_from_fields(
            11,
            source_handles.as_mut_ptr(),
            source_handles.len() as u64,
            &raw mut output_handle,
        )
    };
    assert_eq!(status, SUCCESS);
    let output = reference_handle(output_handle, Kind::Item).expect("typed RSA material");
    let HandleValue::Item(item) = &output.value else {
        panic!("output is an Item")
    };
    item.with_value(|value| {
        let ValueView::Structure(structure) = value else {
            panic!("RSA material is a Structure")
        };
        assert_eq!(
            structure
                .children()
                .iter()
                .map(|field| field.tag().raw())
                .collect::<Vec<_>>(),
            [0x0042_0052, 0x0042_006c]
        );
    });
    kmipkit_ttlv_item_release(output_handle);
    for source in source_handles {
        kmipkit_ttlv_item_release(source);
    }
}
