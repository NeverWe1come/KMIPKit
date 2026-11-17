use super::*;

macro_rules! assert_lifecycle_request {
    ($call:ident, $variant:ident $(,)?) => {{
        let mut item = ptr::null_mut();
        assert_eq!(SUCCESS, $call(ptr::null(), &raw mut item));
        let handle = reference_handle(item, Kind::BatchItem)
            .expect("lifecycle constructor returns a valid item handle");
        let HandleValue::BatchItem(batch_item) = &handle.value else {
            panic!("the constructor returns a batch item");
        };
        assert!(matches!(batch_item.request(), ClientRequest::$variant(request)
            if request.unique_identifier().is_none()));
        kmipkit_client_batch_item_release(item);
    }};
}

#[test]
fn lifecycle_constructors_preserve_optional_identifier_omission() {
    assert_lifecycle_request!(kmipkit_client_batch_item_activate, Activate,);
    assert_lifecycle_request!(kmipkit_client_batch_item_archive, Archive,);
    assert_lifecycle_request!(kmipkit_client_batch_item_destroy, Destroy,);
    assert_lifecycle_request!(kmipkit_client_batch_item_recover, Recover,);
}

macro_rules! assert_lifecycle_request_identifier {
    ($call:ident, $variant:ident, $identifier:expr, $expected:expr $(,)?) => {{
        let mut item = ptr::null_mut();
        let identifier = $identifier;
        assert_eq!(SUCCESS, $call(&raw const identifier, &raw mut item));
        let handle = reference_handle(item, Kind::BatchItem)
            .expect("lifecycle constructor returns a valid item handle");
        let HandleValue::BatchItem(batch_item) = &handle.value else {
            panic!("the constructor returns a batch item");
        };
        let ClientRequest::$variant(request) = batch_item.request() else {
            panic!("the batch item preserves its lifecycle operation");
        };
        assert_eq!(request.unique_identifier(), Some(&$expected));
        kmipkit_client_batch_item_release(item);
    }};
}

#[test]
fn lifecycle_constructors_preserve_supplied_identifier_kind_and_value() {
    let mut activate_text = b"activate-target".to_vec();
    let activate_identifier = kmipkit_unique_identifier_v1_t {
        struct_size: u32::try_from(std::mem::size_of::<kmipkit_unique_identifier_v1_t>())
            .expect("identifier size fits ABI field"),
        abi_version: 1,
        kind: crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING,
        reserved: 0,
        text_data: activate_text.as_ptr(),
        text_length: u64::try_from(activate_text.len()).expect("text length fits ABI"),
        enumeration_value: 0,
        integer_value: 0,
    };
    let expected_activate = UniqueIdentifier::TextString("activate-target".to_owned());
    let archive_identifier = kmipkit_unique_identifier_v1_t {
        struct_size: u32::try_from(std::mem::size_of::<kmipkit_unique_identifier_v1_t>())
            .expect("identifier size fits ABI field"),
        abi_version: 1,
        kind: crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_ENUMERATION,
        reserved: 0,
        text_data: ptr::null(),
        text_length: 0,
        enumeration_value: 0x8fed_cba9,
        integer_value: 0,
    };
    let expected_archive = UniqueIdentifier::Enumeration(0x8fed_cba9);
    let destroy_identifier = kmipkit_unique_identifier_v1_t {
        struct_size: u32::try_from(std::mem::size_of::<kmipkit_unique_identifier_v1_t>())
            .expect("identifier size fits ABI field"),
        abi_version: 1,
        kind: crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_INTEGER,
        reserved: 0,
        text_data: ptr::null(),
        text_length: 0,
        enumeration_value: 0,
        integer_value: -723,
    };
    let expected_destroy = UniqueIdentifier::Integer(-723);
    let mut recover_text = b"recover-target".to_vec();
    let recover_identifier = kmipkit_unique_identifier_v1_t {
        struct_size: u32::try_from(std::mem::size_of::<kmipkit_unique_identifier_v1_t>())
            .expect("identifier size fits ABI field"),
        abi_version: 1,
        kind: crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING,
        reserved: 0,
        text_data: recover_text.as_ptr(),
        text_length: u64::try_from(recover_text.len()).expect("text length fits ABI"),
        enumeration_value: 0,
        integer_value: 0,
    };
    let expected_recover = UniqueIdentifier::TextString("recover-target".to_owned());

    assert_lifecycle_request_identifier!(
        kmipkit_client_batch_item_activate,
        Activate,
        activate_identifier,
        expected_activate
    );
    activate_text.fill(b'x');
    assert_lifecycle_request_identifier!(
        kmipkit_client_batch_item_archive,
        Archive,
        archive_identifier,
        expected_archive
    );
    assert_lifecycle_request_identifier!(
        kmipkit_client_batch_item_destroy,
        Destroy,
        destroy_identifier,
        expected_destroy
    );
    assert_lifecycle_request_identifier!(
        kmipkit_client_batch_item_recover,
        Recover,
        recover_identifier,
        expected_recover
    );
    recover_text.fill(b'x');
}

macro_rules! assert_successful_lifecycle_response {
    ($request:expr, $operation:expr, $getter:ident) => {{
        let identifier = b"lifecycle-response";
        let response_item = successful_cryptographic_item(
            $request,
            $operation,
            b"lifecycle-batch-item",
            vec![ttlv_item(0x0042_0094, 0x07, identifier)],
        );
        let mut present = 0;
        let mut output_identifier = empty_unique_identifier_output_v1();
        assert_eq!(
            SUCCESS,
            $getter(response_item, &raw mut present, &raw mut output_identifier)
        );
        assert_eq!(present, 1);
        assert_response_identifier(&output_identifier, identifier);
        kmipkit_client_batch_item_response_release(response_item);
    }};
}

macro_rules! assert_failed_lifecycle_response {
    ($request:expr, $operation:expr, $getter:ident) => {{
        let response_item =
            failed_cryptographic_item($request, $operation, b"lifecycle-batch-item");
        let mut present = 1;
        let mut output_identifier = empty_unique_identifier_output_v1();
        assert_eq!(
            SUCCESS,
            $getter(response_item, &raw mut present, &raw mut output_identifier)
        );
        assert_eq!(present, 0);
        assert_eq!(output_identifier.kind, 0);
        kmipkit_client_batch_item_response_release(response_item);
    }};
}

#[test]
fn lifecycle_response_accessors_return_identifier_only_for_success() {
    assert_successful_lifecycle_response!(
        ClientRequest::Activate(ActivateRequest::new(Some(response_fixture_uid()))),
        0x12,
        kmipkit_client_batch_item_response_activate
    );
    assert_successful_lifecycle_response!(
        ClientRequest::Archive(ArchiveRequest::new(Some(response_fixture_uid()))),
        0x15,
        kmipkit_client_batch_item_response_archive
    );
    assert_successful_lifecycle_response!(
        ClientRequest::Destroy(DestroyRequest::new(Some(response_fixture_uid()))),
        0x14,
        kmipkit_client_batch_item_response_destroy
    );
    assert_successful_lifecycle_response!(
        ClientRequest::Recover(RecoverRequest::new(Some(response_fixture_uid()))),
        0x16,
        kmipkit_client_batch_item_response_recover
    );
    assert_successful_lifecycle_response!(
        ClientRequest::CreateGroup(kmipkit_protocol::CreateGroupRequest::new(
            AttributeSet::new()
        )),
        0x3C,
        kmipkit_client_batch_item_response_create_group
    );
    assert_successful_lifecycle_response!(
        ClientRequest::CreateUser(kmipkit_protocol::CreateUserRequest::new(AttributeSet::new())),
        0x3E,
        kmipkit_client_batch_item_response_create_user
    );
    let password = kmipkit_protocol::PasswordCredentialObject::try_new(
        kmipkit_protocol::PasswordCredentialObjectFields {
            password: Some(SecretText::new("secret-marker".to_owned())),
            ..kmipkit_protocol::PasswordCredentialObjectFields::default()
        },
    )
    .expect("valid password credential");
    assert_successful_lifecycle_response!(
        ClientRequest::CreateCredential(kmipkit_protocol::CreateCredentialRequest::new(
            kmipkit_protocol::CredentialType::UsernameAndPassword,
            AttributeSet::new(),
            kmipkit_protocol::CreateCredentialObject::PasswordCredential(password),
        )),
        0x3F,
        kmipkit_client_batch_item_response_create_credential
    );
    assert_successful_lifecycle_response!(
        ClientRequest::Deactivate(kmipkit_protocol::DeactivateRequest::new(
            response_fixture_uid()
        )),
        0x40,
        kmipkit_client_batch_item_response_deactivate
    );
    assert_successful_lifecycle_response!(
        ClientRequest::Decapsulate(kmipkit_protocol::DecapsulateRequest::new(
            response_fixture_uid(),
            vec![0x12]
        )),
        0x42,
        kmipkit_client_batch_item_response_decapsulate
    );
}

#[test]
fn obliterate_and_encapsulate_response_accessors_return_data_only_for_success() {
    let response_item = successful_cryptographic_item(
        ClientRequest::Obliterate(kmipkit_protocol::ObliterateRequest::new(
            response_fixture_uid(),
        )),
        0x3D,
        b"lifecycle-batch-item",
        vec![],
    );
    let mut present = 0;
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_response_obliterate(response_item, &raw mut present)
    );
    assert_eq!(present, 1);
    kmipkit_client_batch_item_response_release(response_item);

    let identifier = b"encapsulate-response";
    let ciphertext = b"encapsulate-ciphertext";
    let response_item = successful_cryptographic_item(
        ClientRequest::Encapsulate(kmipkit_protocol::EncapsulateRequest::new(
            response_fixture_uid(),
        )),
        0x41,
        b"lifecycle-batch-item",
        vec![
            ttlv_item(0x0042_0094, 0x07, identifier),
            ttlv_item(0x0042_00C2, 0x08, ciphertext),
        ],
    );
    let mut present = 0;
    let mut output_identifier = empty_unique_identifier_output_v1();
    let mut data_present = 0;
    let mut data = kmipkit_byte_span_v1_t {
        data: ptr::null(),
        length: 0,
    };
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_response_encapsulate(
            response_item,
            &raw mut present,
            &raw mut output_identifier,
            &raw mut data_present,
            &raw mut data,
        )
    );
    assert_eq!(present, 1);
    assert_response_identifier(&output_identifier, identifier);
    assert_eq!(data_present, 1);
    assert_eq!(
        unsafe { std::slice::from_raw_parts(data.data, usize::try_from(data.length).unwrap()) },
        ciphertext
    );
    kmipkit_client_batch_item_response_release(response_item);
}

#[test]
fn lifecycle_response_accessors_clear_identifier_for_failure() {
    assert_failed_lifecycle_response!(
        ClientRequest::Recover(RecoverRequest::new(Some(response_fixture_uid()))),
        0x16,
        kmipkit_client_batch_item_response_recover
    );
    assert_failed_lifecycle_response!(
        ClientRequest::CreateGroup(kmipkit_protocol::CreateGroupRequest::new(
            AttributeSet::new(),
        )),
        0x3C,
        kmipkit_client_batch_item_response_create_group
    );
    assert_failed_lifecycle_response!(
        ClientRequest::CreateUser(kmipkit_protocol::CreateUserRequest::new(AttributeSet::new())),
        0x3E,
        kmipkit_client_batch_item_response_create_user
    );
    let password = kmipkit_protocol::PasswordCredentialObject::try_new(
        kmipkit_protocol::PasswordCredentialObjectFields {
            password: Some(SecretText::new("secret-marker".to_owned())),
            ..kmipkit_protocol::PasswordCredentialObjectFields::default()
        },
    )
    .expect("valid password credential");
    assert_failed_lifecycle_response!(
        ClientRequest::CreateCredential(kmipkit_protocol::CreateCredentialRequest::new(
            kmipkit_protocol::CredentialType::UsernameAndPassword,
            AttributeSet::new(),
            kmipkit_protocol::CreateCredentialObject::PasswordCredential(password),
        )),
        0x3F,
        kmipkit_client_batch_item_response_create_credential
    );
    assert_failed_lifecycle_response!(
        ClientRequest::Deactivate(kmipkit_protocol::DeactivateRequest::new(
            response_fixture_uid(),
        )),
        0x40,
        kmipkit_client_batch_item_response_deactivate
    );
    assert_failed_lifecycle_response!(
        ClientRequest::Decapsulate(kmipkit_protocol::DecapsulateRequest::new(
            response_fixture_uid(),
            vec![0x12],
        )),
        0x42,
        kmipkit_client_batch_item_response_decapsulate
    );

    let obliterate_failure = failed_cryptographic_item(
        ClientRequest::Obliterate(kmipkit_protocol::ObliterateRequest::new(
            response_fixture_uid(),
        )),
        0x3D,
        b"lifecycle-batch-item",
    );
    let mut obliterate_present = 1;
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_response_obliterate(
            obliterate_failure,
            &raw mut obliterate_present,
        )
    );
    assert_eq!(obliterate_present, 0);
    kmipkit_client_batch_item_response_release(obliterate_failure);

    let encap_failure = failed_cryptographic_item(
        ClientRequest::Encapsulate(kmipkit_protocol::EncapsulateRequest::new(
            response_fixture_uid(),
        )),
        0x41,
        b"lifecycle-batch-item",
    );
    let mut encap_present = 1;
    let mut output_identifier = empty_unique_identifier_output_v1();
    let mut data_present = 1;
    let mut data = kmipkit_byte_span_v1_t {
        data: ptr::null(),
        length: 0,
    };
    assert_eq!(
        SUCCESS,
        kmipkit_client_batch_item_response_encapsulate(
            encap_failure,
            &raw mut encap_present,
            &raw mut output_identifier,
            &raw mut data_present,
            &raw mut data,
        )
    );
    assert_eq!(encap_present, 0);
    assert_eq!(data_present, 0);
    kmipkit_client_batch_item_response_release(encap_failure);
}
