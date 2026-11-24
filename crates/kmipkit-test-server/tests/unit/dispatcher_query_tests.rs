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
