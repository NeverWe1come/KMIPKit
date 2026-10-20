use super::*;

const QUERY_OPERATION: u32 = 0x0000_0018;

fn query_status(
    functions: *const u32,
    function_count: u64,
    has_object_groups: u8,
    object_groups: *const kmipkit_byte_span_v1_t,
    object_group_count: u64,
) -> i32 {
    let mut item = ptr::dangling_mut();
    let status = kmipkit_client_batch_item_query(
        functions,
        function_count,
        has_object_groups,
        object_groups,
        object_group_count,
        &raw mut item,
    );
    if status == SUCCESS {
        assert!(!item.is_null());
        kmipkit_client_batch_item_release(item);
    } else {
        assert!(item.is_null());
    }
    status
}

fn query_status_with_limits(
    limits: *mut kmipkit_codec_limits_t,
    functions: *const u32,
    function_count: u64,
    has_object_groups: u8,
    object_groups: *const kmipkit_byte_span_v1_t,
    object_group_count: u64,
) -> i32 {
    let mut item = ptr::dangling_mut();
    let status = kmipkit_client_batch_item_query_with_limits(
        limits,
        functions,
        function_count,
        has_object_groups,
        object_groups,
        object_group_count,
        &raw mut item,
    );
    if status == SUCCESS {
        assert!(!item.is_null());
        kmipkit_client_batch_item_release(item);
    } else {
        assert!(item.is_null());
    }
    status
}

fn custom_query_status(
    max_message_bytes: u64,
    max_structure_depth: u64,
    max_elements: u64,
    functions: *const u32,
    function_count: u64,
    has_object_groups: u8,
    object_groups: *const kmipkit_byte_span_v1_t,
    object_group_count: u64,
) -> i32 {
    let mut limits = ptr::null_mut();
    assert_eq!(
        kmipkit_codec_limits_create(
            max_message_bytes,
            max_structure_depth,
            max_elements,
            &raw mut limits,
        ),
        SUCCESS
    );
    let status = query_status_with_limits(
        limits,
        functions,
        function_count,
        has_object_groups,
        object_groups,
        object_group_count,
    );
    kmipkit_codec_limits_release(limits);
    status
}

#[test]
fn ffi_query_builder_applies_codec_limits_before_copying_and_accepts_custom_limits() {
    let functions = vec![1_u32; 100_001];
    let group = b"group";
    let groups = vec![byte_span(group); 100_001];
    assert_eq!(
        query_status(
            functions.as_ptr(),
            functions.len() as u64,
            1,
            groups.as_ptr(),
            groups.len() as u64,
        ),
        ERROR_RESOURCE_LIMIT,
        "the compatibility constructor enforces default limits"
    );
    assert_eq!(
        custom_query_status(
            8 * 1024 * 1024,
            3,
            400_000,
            functions.as_ptr(),
            functions.len() as u64,
            1,
            groups.as_ptr(),
            groups.len() as u64,
        ),
        ERROR_RESOURCE_LIMIT,
        "Object Groups require four levels in the complete Request Message"
    );
    assert_eq!(
        custom_query_status(
            8 * 1024 * 1024,
            2,
            400_000,
            functions.as_ptr(),
            functions.len() as u64,
            0,
            ptr::null(),
            0,
        ),
        ERROR_RESOURCE_LIMIT,
        "a Query without Object Groups requires three message structure levels"
    );
    let mut defaults = ptr::null_mut();
    assert_eq!(kmipkit_codec_limits_defaults(&raw mut defaults), SUCCESS);
    assert_eq!(
        query_status_with_limits(
            defaults,
            functions.as_ptr(),
            functions.len() as u64,
            1,
            groups.as_ptr(),
            groups.len() as u64,
        ),
        ERROR_RESOURCE_LIMIT
    );
    kmipkit_codec_limits_release(defaults);

    let mut custom = ptr::null_mut();
    assert_eq!(
        kmipkit_codec_limits_create(8 * 1024 * 1024, 64, 400_000, &raw mut custom),
        SUCCESS
    );
    assert_eq!(
        query_status_with_limits(
            custom,
            functions.as_ptr(),
            functions.len() as u64,
            1,
            groups.as_ptr(),
            groups.len() as u64,
        ),
        SUCCESS
    );
    kmipkit_codec_limits_release(custom);
}

#[test]
fn ffi_query_preflight_rejects_object_groups_over_the_element_budget() {
    let function = QUERY_OPERATION;
    let group = b"group";
    let groups = [byte_span(group)];

    assert_eq!(
        custom_query_status(1024, 4, 3, &function, 1, 1, groups.as_ptr(), 1,),
        ERROR_RESOURCE_LIMIT,
        "the function fits, but the Object Groups structure and member exceed the budget"
    );
}

#[test]
fn ffi_query_preflight_rejects_encoded_size_after_object_groups_are_added() {
    let function = QUERY_OPERATION;
    let group = b"group";
    let groups = [byte_span(group)];

    assert_eq!(
        custom_query_status(40, 4, 100, &function, 1, 1, groups.as_ptr(), 1,),
        ERROR_RESOURCE_LIMIT,
        "the Query functions fit, but the complete payload with Object Groups does not"
    );
}

#[test]
fn ffi_query_builder_accepts_present_empty_object_groups_at_exact_ttlv_size() {
    let function = QUERY_OPERATION;

    assert_eq!(
        custom_query_status(32, 4, 3, &function, 1, 1, ptr::null(), 0),
        SUCCESS,
        "Request Payload, Query Function, and empty Object Groups occupy 32 TTLV bytes"
    );
}

#[test]
fn ffi_query_builder_rejects_text_string_length_above_ttlv_item_length_limit() {
    let function = QUERY_OPERATION;
    let oversized_group = kmipkit_byte_span_v1_t {
        data: ptr::null(),
        length: u64::from(u32::MAX) + 1,
    };

    assert_eq!(
        custom_query_status(
            u64::MAX,
            64,
            100,
            &function,
            1,
            1,
            &raw const oversized_group,
            1,
        ),
        ERROR_RESOURCE_LIMIT,
        "Text String Item Length is a four-byte unsigned TTLV field"
    );
}

fn successful_response_item(
    request: ClientRequest,
    operation: u32,
    batch_item_id: &[u8],
    response_fields: Vec<Vec<u8>>,
) -> *mut kmipkit_client_batch_item_response_t {
    let response = execute_single_abi_response(
        request,
        batch_item_id,
        successful_cryptographic_response(operation, batch_item_id, response_fields),
    )
    .unwrap_or_else(|error| {
        panic!("successful response for operation {operation} failed validation: {error:?}")
    });
    let response_handle = make_handle(
        Kind::ClientBatchResponse,
        HandleValue::ClientBatchResponse(Arc::new(response)),
    );
    let mut item = ptr::null_mut();
    assert_eq!(
        kmipkit_client_batch_response_item_at(response_handle, 0, &raw mut item),
        SUCCESS
    );
    kmipkit_client_batch_response_release(response_handle);
    item
}

#[test]
fn ffi_ping_and_query_constructors_cover_empty_and_counted_inputs() {
    let mut ping = ptr::null_mut();
    assert_eq!(kmipkit_client_batch_item_ping(&raw mut ping), SUCCESS);
    assert!(!ping.is_null());
    kmipkit_client_batch_item_release(ping);

    assert_eq!(
        query_status(ptr::null(), 0, 0, ptr::null(), 0),
        ERROR_INVALID_INPUT
    );
    assert_eq!(
        query_status(ptr::null(), 0, 1, ptr::null(), 0),
        ERROR_INVALID_INPUT
    );

    let functions = [1, 0x8000_0001, 1];
    let first_group = b"group-a";
    let second_group = b"group-a";
    let groups = [byte_span(first_group), byte_span(second_group)];
    assert_eq!(
        query_status(
            functions.as_ptr(),
            functions.len() as u64,
            1,
            groups.as_ptr(),
            groups.len() as u64,
        ),
        SUCCESS
    );
}

#[test]
fn ffi_query_constructor_accepts_absent_object_groups() {
    let function = QUERY_OPERATION;

    assert_eq!(
        query_status(&function, 1, 0, ptr::null(), 0),
        SUCCESS,
        "an absent optional Object Groups field is valid"
    );
}

#[test]
fn ffi_query_preflight_rejects_parent_structure_length_overflow_before_copying() {
    let length = u64::from(u32::MAX) - 7;
    let spans = [kmipkit_byte_span_v1_t {
        data: ptr::null(),
        length,
    }];
    assert_eq!(
        custom_query_status(
            u64::MAX,
            64,
            u64::MAX,
            [1].as_ptr(),
            1,
            1,
            spans.as_ptr(),
            1,
        ),
        ERROR_RESOURCE_LIMIT,
        "Object Groups parent Item Length must be checked before dereferencing the span"
    );
}

#[test]
fn ffi_query_constructor_rejects_invalid_flags_pointers_and_limits() {
    let functions = [1_u32];
    let dangling_functions = ptr::NonNull::<u32>::dangling().as_ptr();
    let dangling_groups = ptr::NonNull::<kmipkit_byte_span_v1_t>::dangling().as_ptr();

    assert_eq!(
        query_status(ptr::null(), 1, 0, ptr::null(), 0),
        ERROR_INVALID_INPUT
    );
    assert_eq!(
        query_status(functions.as_ptr(), 1, 0, dangling_groups, 1,),
        ERROR_INVALID_INPUT,
        "Object Groups cannot be supplied when the presence flag is clear"
    );
    assert_eq!(
        query_status(functions.as_ptr(), 1, 2, ptr::null(), 0),
        ERROR_INVALID_INPUT,
        "presence flags are fixed-width booleans"
    );
    assert_eq!(
        query_status(functions.as_ptr(), 1, 1, ptr::null(), 1),
        ERROR_INVALID_INPUT
    );
    assert_eq!(
        query_status(dangling_functions, u64::MAX, 0, ptr::null(), 0,),
        ERROR_RESOURCE_LIMIT
    );
    assert_eq!(
        query_status(functions.as_ptr(), 1, 1, dangling_groups, u64::MAX,),
        ERROR_RESOURCE_LIMIT
    );

    let missing_bytes = [kmipkit_byte_span_v1_t {
        data: ptr::null(),
        length: 1,
    }];
    assert_eq!(
        query_status(functions.as_ptr(), 1, 1, missing_bytes.as_ptr(), 1),
        ERROR_INVALID_INPUT
    );
}

#[test]
fn ffi_query_constructor_rejects_malformed_object_group_utf8_after_preflight() {
    let function = QUERY_OPERATION;
    let malformed_group = [b'g', 0xc3, 0x28];
    let groups = [byte_span(&malformed_group)];

    assert_eq!(
        custom_query_status(1024, 4, 100, &function, 1, 1, groups.as_ptr(), 1,),
        ERROR_INVALID_INPUT,
        "object group bytes must be valid UTF-8 after passing size preflight"
    );
}

#[test]
fn ffi_query_constructor_accepts_large_ordered_function_and_group_arrays() {
    let values = vec![1_u32; 100_001];
    let group = b"group";
    let group_span = byte_span(group);
    let groups = vec![group_span; 100_001];

    assert_eq!(
        custom_query_status(
            8 * 1024 * 1024,
            64,
            400_000,
            values.as_ptr(),
            values.len() as u64,
            1,
            groups.as_ptr(),
            groups.len() as u64,
        ),
        SUCCESS,
        "the configurable codec element limit, not a fixed adapter count, governs Query"
    );
}

#[test]
fn ffi_query_constructor_accepts_object_group_larger_than_default_message_limit() {
    let functions = [1_u32];
    let group = vec![b'g'; 16 * 1024 * 1024 + 1];
    let group_span = [byte_span(&group)];

    assert_eq!(
        custom_query_status(
            32 * 1024 * 1024,
            64,
            100_000,
            functions.as_ptr(),
            1,
            1,
            group_span.as_ptr(),
            1,
        ),
        SUCCESS,
        "Query input construction remains configurable; execution applies CodecLimits"
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn ffi_ping_response_presence_and_query_ttlv_copy_are_typed_and_lossless() {
    let ping = successful_response_item(
        ClientRequest::ping(),
        0x0000_003b,
        b"ping-result",
        Vec::new(),
    );
    let query = successful_response_item(
        ClientRequest::query(QueryRequest::new([QueryFunction::from_raw(1)])),
        QUERY_OPERATION,
        b"query-result",
        vec![
            ttlv_enumeration(0x0042_005c, 0x0000_003b),
            ttlv_structure(0x0042_015f, []),
            ttlv_structure(0x0042_0012, [ttlv_integer(0x0042_0057, 7)]),
        ],
    );

    let mut present = u8::MAX;
    assert_eq!(
        kmipkit_client_batch_item_response_ping(ping, &raw mut present),
        SUCCESS
    );
    assert_eq!(present, 1);

    present = u8::MAX;
    assert_eq!(
        kmipkit_client_batch_item_response_ping(query, &raw mut present),
        SUCCESS
    );
    assert_eq!(present, 0);

    let mut query_ttlv = ptr::dangling_mut();
    present = u8::MAX;
    assert_eq!(
        kmipkit_client_batch_item_response_query_ttlv(query, &raw mut present, &raw mut query_ttlv,),
        SUCCESS
    );
    assert_eq!(present, 1);
    assert!(!query_ttlv.is_null());

    let mut view = ptr::null_mut();
    assert_eq!(
        kmipkit_ttlv_structure_view(query_ttlv, &raw mut view),
        SUCCESS
    );
    let mut item_count = u64::MAX;
    assert_eq!(
        kmipkit_ttlv_structure_view_item_count(view, &raw mut item_count),
        SUCCESS
    );
    assert_eq!(
        item_count, 3,
        "known, required, and unknown Query fields are all copied"
    );
    kmipkit_ttlv_structure_view_release(view);
    kmipkit_ttlv_structure_release(query_ttlv);

    query_ttlv = ptr::dangling_mut();
    present = u8::MAX;
    assert_eq!(
        kmipkit_client_batch_item_response_query_ttlv(ping, &raw mut present, &raw mut query_ttlv,),
        SUCCESS
    );
    assert_eq!(present, 0);
    assert!(query_ttlv.is_null());

    present = u8::MAX;
    query_ttlv = ptr::dangling_mut();
    assert_eq!(
        kmipkit_client_batch_item_response_query_ttlv(
            ptr::null_mut(),
            &raw mut present,
            &raw mut query_ttlv,
        ),
        ERROR_INVALID_INPUT
    );
    assert_eq!(present, 0);
    assert!(query_ttlv.is_null());

    let response_handle = reference_handle(query, Kind::ClientBatchItemResponse)
        .expect("the Query result handle remains registered");
    let HandleValue::ClientBatchItemResponse { response, .. } = &response_handle.value else {
        panic!("the Query result handle stores a batch response and item index");
    };
    let invalid_index = make_handle(
        Kind::ClientBatchItemResponse,
        HandleValue::ClientBatchItemResponse {
            response: Arc::clone(response),
            index: usize::MAX,
        },
    );
    present = u8::MAX;
    assert_eq!(
        kmipkit_client_batch_item_response_ping(invalid_index, &raw mut present),
        ERROR_INTERNAL
    );
    assert_eq!(present, 0);

    present = u8::MAX;
    query_ttlv = ptr::dangling_mut();
    assert_eq!(
        kmipkit_client_batch_item_response_query_ttlv(
            invalid_index,
            &raw mut present,
            &raw mut query_ttlv,
        ),
        ERROR_INTERNAL
    );
    assert_eq!(present, 0);
    assert!(query_ttlv.is_null());

    kmipkit_client_batch_item_response_release(invalid_index);
    kmipkit_client_batch_item_response_release(query);
    kmipkit_client_batch_item_response_release(ping);
}
