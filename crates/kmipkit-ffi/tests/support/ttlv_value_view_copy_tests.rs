use super::*;

fn make_value_view(value: Value) -> (*mut kmipkit_ttlv_value_t, *mut kmipkit_ttlv_value_view_t) {
    let source = make_handle(Kind::Value, HandleValue::Value(value));
    let mut view = ptr::null_mut();
    assert_eq!(kmipkit_ttlv_value_view(source, &raw mut view), SUCCESS);
    (source, view)
}

fn assert_copied_bytes(value: Value, expected: &[u8]) {
    let (source, view) = make_value_view(value);
    let mut output = vec![0; expected.len()];
    assert_eq!(
        kmipkit_ttlv_value_view_copy_bytes(
            view,
            output.as_mut_ptr(),
            u64::try_from(output.len()).expect("test byte length fits u64"),
        ),
        SUCCESS
    );
    assert_eq!(output, expected);
    kmipkit_ttlv_value_view_release(view);
    kmipkit_ttlv_value_release(source);
}

#[test]
fn copies_utf8_text_string_in_one_complete_buffer() {
    assert_copied_bytes(
        Value::text_string("namespace — café".to_owned()),
        "namespace — café".as_bytes(),
    );
}

#[test]
fn copies_byte_string_and_big_integer_as_raw_bytes() {
    assert_copied_bytes(Value::byte_string(vec![0x00, 0x80, 0xFF]), &[0x00, 0x80, 0xFF]);
    assert_copied_bytes(Value::big_integer(vec![0xFF, 0x80, 0x01]), &[0xFF, 0x80, 0x01]);
}

#[test]
fn rejects_small_capacity_without_writing_a_prefix() {
    let (source, view) = make_value_view(Value::text_string("complete".to_owned()));
    let mut output = [0xA5; 8];
    assert_eq!(
        kmipkit_ttlv_value_view_copy_bytes(view, output.as_mut_ptr(), 7),
        ERROR_INVALID_INPUT
    );
    assert_eq!(output, [0xA5; 8]);
    kmipkit_ttlv_value_view_release(view);
    kmipkit_ttlv_value_release(source);
}

#[test]
fn rejects_null_buffer_for_nonempty_bytes_without_panicking() {
    let (source, view) = make_value_view(Value::byte_string(vec![0x10]));
    assert_eq!(
        kmipkit_ttlv_value_view_copy_bytes(view, ptr::null_mut(), 1),
        ERROR_INVALID_INPUT
    );
    kmipkit_ttlv_value_view_release(view);
    kmipkit_ttlv_value_release(source);
}

#[test]
fn rejects_non_byte_backed_values_without_writing_output() {
    let (source, view) = make_value_view(Value::integer(17));
    let mut output = [0xA5; 4];
    assert_eq!(
        kmipkit_ttlv_value_view_copy_bytes(view, output.as_mut_ptr(), 4),
        ERROR_INVALID_INPUT
    );
    assert_eq!(output, [0xA5; 4]);
    kmipkit_ttlv_value_view_release(view);
    kmipkit_ttlv_value_release(source);
}

#[test]
fn accepts_null_buffer_for_empty_text_without_writing() {
    let (source, view) = make_value_view(Value::text_string(String::new()));
    assert_eq!(
        kmipkit_ttlv_value_view_copy_bytes(view, ptr::null_mut(), 0),
        SUCCESS
    );
    kmipkit_ttlv_value_view_release(view);
    kmipkit_ttlv_value_release(source);
}
