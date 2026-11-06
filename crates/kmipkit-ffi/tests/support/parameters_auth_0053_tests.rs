use super::*;

unsafe extern "C" {
    fn kmipkit_parameters_authentication_c_consumer_parameters() -> i32;
    fn kmipkit_parameters_authentication_c_consumer_attach_authentication(
        batch: *mut kmipkit_client_batch_t,
        authentication: *mut *mut kmipkit_authentication_t,
    ) -> i32;
}

#[test]
fn compiled_c_consumer_builds_all_table_59_parameters_in_source_order() {
    // SAFETY: This no-argument consumer owns and releases every handle it creates.
    let status = unsafe { kmipkit_parameters_authentication_c_consumer_parameters() };
    assert_eq!(status, SUCCESS);
}

#[test]
fn compiled_c_consumer_can_attach_authentication_to_a_batch() {
    let credential = Credential::try_from_ttlv({
        let mut fields = Structure::new();
        fields
            .try_push(Item::new(
                RawTag::new(0x0042_0024)
                    .expect("tag fits")
                    .try_checked()
                    .expect("tag is assigned"),
                Value::enumeration(0x8000_0001),
            ).expect("credential type item is valid"))
            .expect("credential type fits");
        fields
            .try_push(Item::new(
                RawTag::new(0x0042_0025)
                    .expect("tag fits")
                    .try_checked()
                    .expect("tag is assigned"),
                Value::structure(Structure::new()),
            ).expect("credential value item is valid"))
            .expect("credential value fits");
        fields
    })
    .expect("credential envelope is valid");
    let authentication = Authentication::new(vec![credential])
        .expect("one credential forms non-empty Authentication");
    let mut authentication = make_handle(
        Kind::Authentication,
        HandleValue::Authentication(authentication),
    );
    let mut batch = ptr::null_mut();
    assert_eq!(kmipkit_client_batch_create(&raw mut batch), SUCCESS);

    // SAFETY: The C wrapper receives live owned handles and consumes Authentication only on success.
    let status = unsafe {
        kmipkit_parameters_authentication_c_consumer_attach_authentication(
            batch,
            &raw mut authentication,
        )
    };
    assert_eq!(status, SUCCESS);
    assert!(authentication.is_null());
    assert!(reference_handle(authentication, Kind::Authentication).is_err());
    release(batch, Kind::ClientBatch);
}
