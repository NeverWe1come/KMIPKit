use super::*;

const NONCE_ID_TAG: u32 = 0x0042_00C9;
const NONCE_VALUE_TAG: u32 = 0x0042_00CA;
const NONCE_ID: &[u8] = &[0x10, 0x00, 0xFF];
const NONCE_VALUE: &[u8] = &[0x80, 0x01, 0x7F, 0x00];

fn nonce_structure() -> Structure {
    let mut structure = Structure::new();
    for (tag, bytes) in [(NONCE_ID_TAG, NONCE_ID), (NONCE_VALUE_TAG, NONCE_VALUE)] {
        structure
            .try_push(
                Item::new(
                    RawTag::new(tag)
                        .expect("Nonce tags fit the TTLV tag width")
                        .try_checked()
                        .expect("Nonce tags are assigned by KMIP 2.1"),
                    Value::byte_string(bytes.to_vec()),
                )
                .expect("the Nonce fixture item is valid"),
            )
            .expect("the Nonce fixture stays within TTLV limits");
    }
    structure
}

fn assert_nonce_view(view: *mut kmipkit_ttlv_structure_view_t) {
    let mut count = 0;
    assert_eq!(
        kmipkit_ttlv_structure_view_item_count(view, &raw mut count),
        SUCCESS
    );
    assert_eq!(count, 2);
    for (index, (expected_tag, expected_bytes)) in
        [(NONCE_ID_TAG, NONCE_ID), (NONCE_VALUE_TAG, NONCE_VALUE)]
            .into_iter()
            .enumerate()
    {
        let mut item = ptr::null_mut();
        assert_eq!(
            kmipkit_ttlv_structure_view_item_at(view, index as u64, &raw mut item),
            SUCCESS
        );
        let mut tag = ptr::null_mut();
        assert_eq!(kmipkit_ttlv_item_view_tag(item, &raw mut tag), SUCCESS);
        let mut actual_tag = 0;
        assert_eq!(kmipkit_ttlv_tag_value(tag, &raw mut actual_tag), SUCCESS);
        assert_eq!(actual_tag, expected_tag);

        let mut value = ptr::null_mut();
        assert_eq!(kmipkit_ttlv_item_view_value(item, &raw mut value), SUCCESS);
        let mut length = 0;
        assert_eq!(
            kmipkit_ttlv_value_view_byte_length(value, &raw mut length),
            SUCCESS
        );
        assert_eq!(length, expected_bytes.len() as u64);
        for (byte_index, expected) in expected_bytes.iter().copied().enumerate() {
            let mut actual = 0;
            assert_eq!(
                kmipkit_ttlv_value_view_byte_at(value, byte_index as u64, &raw mut actual),
                SUCCESS
            );
            assert_eq!(actual, expected);
        }
        kmipkit_ttlv_value_view_release(value);
        kmipkit_tag_release(tag);
        kmipkit_ttlv_item_view_release(item);
    }
}

#[test]
fn nonce_abi_retains_exact_server_bytes_and_transfers_the_original_structure() {
    let source = make_handle(Kind::Structure, HandleValue::Structure(nonce_structure()));
    let mut nonce: *mut kmipkit_nonce_t = ptr::null_mut();
    assert_eq!(kmipkit_nonce_from_ttlv(source, &raw mut nonce), SUCCESS);

    let mut view = ptr::null_mut();
    assert_eq!(kmipkit_nonce_ttlv_view(nonce, &raw mut view), SUCCESS);
    kmipkit_nonce_release(nonce);
    assert_nonce_view(view);
    kmipkit_ttlv_structure_view_release(view);

    let source = make_handle(Kind::Structure, HandleValue::Structure(nonce_structure()));
    let mut nonce: *mut kmipkit_nonce_t = ptr::null_mut();
    assert_eq!(kmipkit_nonce_from_ttlv(source, &raw mut nonce), SUCCESS);
    let mut restored: *mut kmipkit_ttlv_structure_t = ptr::null_mut();
    assert_eq!(kmipkit_nonce_into_ttlv(nonce, &raw mut restored), SUCCESS);
    assert_eq!(
        reference_handle(restored, Kind::Structure)
            .expect("the moved TTLV structure remains owned")
            .kind,
        Kind::Structure
    );
    let mut restored_view = ptr::null_mut();
    assert_eq!(
        kmipkit_ttlv_structure_view(restored, &raw mut restored_view),
        SUCCESS
    );
    assert_nonce_view(restored_view);
    kmipkit_ttlv_structure_view_release(restored_view);
    kmipkit_ttlv_structure_release(restored);
}

#[test]
fn nonce_abi_preflights_outputs_and_reports_malformed_shapes_without_payloads() {
    let source = make_handle(Kind::Structure, HandleValue::Structure(nonce_structure()));
    assert_eq!(
        kmipkit_nonce_from_ttlv(source, ptr::null_mut()),
        ERROR_INVALID_INPUT
    );
    assert!(reference_handle(source, Kind::Structure).is_ok());

    let malformed = make_handle(Kind::Structure, HandleValue::Structure(Structure::new()));
    let mut nonce: *mut kmipkit_nonce_t = ptr::null_mut();
    assert_eq!(
        kmipkit_nonce_from_ttlv(malformed, &raw mut nonce),
        ERROR_INVALID_INPUT
    );
    assert!(nonce.is_null());
}
