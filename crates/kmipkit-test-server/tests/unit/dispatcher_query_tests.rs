use super::*;

#[test]
fn operations_and_masks_follow_the_requested_version() {
    let mut payload = Structure::new();
    payload
        .try_push(ttlv_item(TAG_QUERY_FUNCTION, Value::enumeration(1)))
        .expect("query function");
    for (major, minor, ping, deactivate, masks) in [
        (2, 0, false, false, true),
        (2, 1, true, false, true),
        (3, 0, true, true, false),
    ] {
        let fields = query_response(major, minor, Some(&payload));
        let operations: Vec<_> = fields
            .iter()
            .filter(|field| field.tag().raw() == TAG_OPERATION)
            .filter_map(extract_enumeration)
            .collect();
        assert!(operations.contains(&QUERY_OPERATION));
        assert_eq!(operations.contains(&PING_OPERATION), ping);
        assert_eq!(operations.contains(&DEACTIVATE_OPERATION), deactivate);
        assert_eq!(
            fields
                .iter()
                .any(|field| field.tag().raw() == TAG_PROTECTION_STORAGE_MASKS),
            masks
        );
    }
}

#[test]
fn queried_objects_and_explicit_masks_are_versioned() {
    assert!(query_response(2, 1, None).is_empty());

    let mut payload = Structure::new();
    for function in [2, 14] {
        payload
            .try_push(ttlv_item(TAG_QUERY_FUNCTION, Value::enumeration(function)))
            .expect("query function");
    }
    for (major, minor, expected) in [(2, 1, &[2, 3, 4][..]), (3, 0, &[2, 3, 4, 7][..])] {
        let fields = query_response(major, minor, Some(&payload));
        let object_types: Vec<_> = fields
            .iter()
            .filter(|field| field.tag().raw() == TAG_OBJECT_TYPE)
            .filter_map(extract_enumeration)
            .collect();
        assert_eq!(object_types, expected);
        assert_eq!(
            fields
                .iter()
                .filter(|field| field.tag().raw() == TAG_PROTECTION_STORAGE_MASKS)
                .count(),
            1
        );
    }
}
