use super::*;

const CREDENTIAL_TYPE_TAG: u32 = 0x0042_0024;
const CREDENTIAL_VALUE_TAG: u32 = 0x0042_0025;

fn credential(raw_type: u32) -> Credential {
    let mut tree = Structure::new();
    tree.try_push(
        Item::new(
            RawTag::new(CREDENTIAL_TYPE_TAG)
                .expect("Credential Type fits the TTLV tag width")
                .try_checked()
                .expect("Credential Type is assigned by KMIP 2.1"),
            Value::enumeration(raw_type),
        )
        .expect("Credential Type item is valid"),
    )
    .expect("Credential Type fits in the Structure");
    tree.try_push(
        Item::new(
            RawTag::new(CREDENTIAL_VALUE_TAG)
                .expect("Credential Value fits the TTLV tag width")
                .try_checked()
                .expect("Credential Value is assigned by KMIP 2.1"),
            Value::structure(Structure::new()),
        )
        .expect("Credential Value item is valid"),
    )
    .expect("Credential Value fits in the Structure");
    Credential::try_from_ttlv(tree).expect("the generic Credential envelope is valid")
}

fn credential_handle(raw_type: u32) -> *mut kmipkit_credential_t {
    make_handle(
        Kind::Credential,
        HandleValue::Credential(credential(raw_type)),
    )
}

#[test]
fn authentication_consumes_ordered_credentials_and_keeps_views_owner_backed() {
    let first = credential_handle(2);
    let second = credential_handle(0xF123_4567);
    let mut inputs = [first, second];
    let mut authentication = ptr::null_mut();
    assert_eq!(
        kmipkit_authentication_create(
            inputs.as_mut_ptr(),
            inputs.len() as u64,
            &raw mut authentication
        ),
        SUCCESS
    );
    assert!(!authentication.is_null());
    assert!(reference_handle(first, Kind::Credential).is_err());
    assert!(reference_handle(second, Kind::Credential).is_err());

    let mut count = u64::MAX;
    assert_eq!(
        kmipkit_authentication_credential_count(authentication, &raw mut count),
        SUCCESS
    );
    assert_eq!(count, 2);

    let mut first_view = ptr::null_mut();
    let mut second_view = ptr::null_mut();
    assert_eq!(
        kmipkit_authentication_credential_at(authentication, 0, &raw mut first_view),
        SUCCESS
    );
    assert_eq!(
        kmipkit_authentication_credential_at(authentication, 1, &raw mut second_view),
        SUCCESS
    );
    for (view, expected) in [(first_view, 2), (second_view, 0xF123_4567)] {
        let mut typed = 0;
        let mut raw = 0;
        assert_eq!(kmipkit_credential_view_type(view, &raw mut typed), SUCCESS);
        assert_eq!(
            kmipkit_credential_view_type_raw(view, &raw mut raw),
            SUCCESS
        );
        assert_eq!(typed, expected);
        assert_eq!(raw, expected);

        let mut ttlv_view = ptr::null_mut();
        assert_eq!(
            kmipkit_credential_view_ttlv_view(view, &raw mut ttlv_view),
            SUCCESS
        );
        let mut item_count = 0;
        assert_eq!(
            kmipkit_ttlv_structure_view_item_count(ttlv_view, &raw mut item_count),
            SUCCESS
        );
        assert_eq!(item_count, 2);
        kmipkit_ttlv_structure_view_release(ttlv_view);
    }

    kmipkit_authentication_release(authentication);
    let mut still_live = 0;
    assert_eq!(
        kmipkit_credential_view_type_raw(first_view, &raw mut still_live),
        SUCCESS
    );
    assert_eq!(still_live, 2);
    kmipkit_credential_view_release(first_view);
    kmipkit_credential_view_release(second_view);
}

#[test]
fn authentication_create_rejects_shared_or_duplicate_inputs_without_partial_consumption() {
    let shared = credential_handle(1);
    let untouched = credential_handle(2);
    let mut retained_view = ptr::null_mut();
    assert_eq!(
        kmipkit_credential_ttlv_view(shared, &raw mut retained_view),
        SUCCESS
    );
    let mut shared_inputs = [shared, untouched];
    let mut authentication = ptr::null_mut();
    assert_eq!(
        kmipkit_authentication_create(
            shared_inputs.as_mut_ptr(),
            shared_inputs.len() as u64,
            &raw mut authentication
        ),
        ERROR_SHARED_OWNER
    );
    assert!(authentication.is_null());
    assert!(reference_handle(shared, Kind::Credential).is_ok());
    assert!(reference_handle(untouched, Kind::Credential).is_ok());

    kmipkit_ttlv_structure_view_release(retained_view);
    let mut duplicate_inputs = [shared, shared];
    assert_eq!(
        kmipkit_authentication_create(
            duplicate_inputs.as_mut_ptr(),
            duplicate_inputs.len() as u64,
            &raw mut authentication
        ),
        ERROR_INVALID_INPUT
    );
    assert!(reference_handle(shared, Kind::Credential).is_ok());
    assert!(reference_handle(untouched, Kind::Credential).is_ok());
    kmipkit_credential_release(shared);
    kmipkit_credential_release(untouched);
}

#[test]
fn authentication_from_ttlv_rejects_empty_and_preserves_validated_ttlv() {
    let empty = make_handle(Kind::Structure, HandleValue::Structure(Structure::new()));
    let mut authentication = ptr::null_mut();
    assert_eq!(
        kmipkit_authentication_from_ttlv(empty, &raw mut authentication),
        ERROR_INVALID_INPUT
    );
    assert!(authentication.is_null());

    let source = Authentication::new(vec![credential(0x8000_0009), credential(1)])
        .expect("Authentication has two credentials")
        .into_ttlv();
    let source = make_handle(Kind::Structure, HandleValue::Structure(source));
    assert_eq!(
        kmipkit_authentication_from_ttlv(source, &raw mut authentication),
        SUCCESS
    );
    let mut count = 0;
    assert_eq!(
        kmipkit_authentication_credential_count(authentication, &raw mut count),
        SUCCESS
    );
    assert_eq!(count, 2);
    kmipkit_authentication_release(authentication);
}
