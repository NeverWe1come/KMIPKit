//! Unit tests verifying Fase 2 64/64 fluent request builder parity in Rust.

use kmipkit_protocol::{
    EndpointRole, KeyCompressionType, KeyFormatType, KeyWrapType, ManagedObject, ObjectType,
    OpaqueObject, ProtocolVersion, ResultStatus, SecretBytes, SplitKeyReference,
};
use kmipkit_test_support::ExchangeScript;
use kmipkit_ttlv::{Item, KmipTag, Structure, Tag, Value};

use crate::asynchronous_execution_test_support::client_for;
use crate::execute_test_support::{asynchronous_response_bytes, test_structure};
use crate::fluent::FluentBuildError;

fn item(tag: KmipTag, value: Value) -> Item {
    Item::new(Tag::from(tag), value).expect("valid TTLV test item")
}

#[test]
fn fluent_stage2_new_builders_validate_build_and_map_to_expected_operations() {
    let ok = asynchronous_response_bytes(0x0000_003b, 0, None, None, Some(test_structure([])));
    let (mut client, _, _) = client_for(ExchangeScript::Success {
        response: ok,
        request_write_chunks: Vec::new(),
    });

    // 1. Certify (0x06)
    let cert_item = client
        .request()
        .certify()
        .unique_identifier("pub-1")
        .certificate_request_type(1)
        .certificate_request_value(SecretBytes::new(vec![0x30, 0x03]))
        .to_batch_item()
        .expect("certify builds");
    assert_eq!(cert_item.operation(), 0x0000_0006);

    // 2. Validate (0x17)
    let val_item = client
        .request()
        .validate()
        .unique_identifier("cert-1")
        .validity_date(1_700_000_000)
        .to_batch_item()
        .expect("validate builds");
    assert_eq!(val_item.operation(), 0x0000_0017);

    // 3-6. Async ops: Cancel (0x19), Poll (0x1A), Process (0x3A), QueryAsyncRequests (0x39)
    let cancel_item = client
        .request()
        .cancel(SecretBytes::new(vec![1, 2, 3, 4]))
        .to_batch_item();
    assert_eq!(cancel_item.operation(), 0x0000_0019);

    let poll_item = client
        .request()
        .poll(SecretBytes::new(vec![1, 2, 3, 4]))
        .to_batch_item();
    assert_eq!(poll_item.operation(), 0x0000_001a);

    let process_item = client
        .request()
        .process(SecretBytes::new(vec![1, 2, 3, 4]))
        .to_batch_item();
    assert_eq!(process_item.operation(), 0x0000_003a);

    let qasync_item = client
        .request()
        .query_async_requests()
        .correlation_value(SecretBytes::new(vec![1, 2, 3, 4]))
        .operation(0x0000_0001)
        .to_batch_item();
    assert_eq!(qasync_item.operation(), 0x0000_0039);

    // 7. JoinSplitKey (0x29)
    let err = client
        .request()
        .join_split_key()
        .build()
        .expect_err("missing object type");
    assert_eq!(err, FluentBuildError::MissingJoinSplitKeyObjectType);

    let join_item = client
        .request()
        .join_split_key()
        .object_type(ObjectType::from_raw(2))
        .reference(SplitKeyReference::new("part-1".into()).with_split_key_threshold(2))
        .split_key_identifier("part-2", Some(2))
        .to_batch_item()
        .expect("join_split_key builds");
    assert_eq!(join_item.operation(), 0x0000_0029);

    // 8. Import (0x2A)
    let err = client
        .request()
        .import_object()
        .build()
        .expect_err("missing uid");
    assert_eq!(err, FluentBuildError::MissingImportUniqueIdentifier);

    let import_item = client
        .request()
        .import_object()
        .unique_identifier("imp-1")
        .managed_object(ManagedObject::OpaqueObject(
            OpaqueObject::try_new(1, b"opaque".to_vec()).expect("opaque"),
        ))
        .replace_existing(true)
        .key_wrap_type(KeyWrapType::from_raw(1))
        .to_batch_item()
        .expect("import_object builds");
    assert_eq!(import_item.operation(), 0x0000_002a);

    // 9. Export (0x2B)
    let export_item = client
        .request()
        .export()
        .unique_identifier("exp-1")
        .key_format_type(KeyFormatType::from_raw(1))
        .key_wrap_type(KeyWrapType::from_raw(1))
        .key_compression_type(KeyCompressionType::from_raw(1))
        .to_batch_item()
        .expect("export builds");
    assert_eq!(export_item.operation(), 0x0000_002b);
}

#[test]
fn fluent_stage2_session_and_admin_builders_validate_and_build() {
    let ok = asynchronous_response_bytes(0x0000_003b, 0, None, None, Some(test_structure([])));
    let (mut client, _, _) = client_for(ExchangeScript::Success {
        response: ok,
        request_write_chunks: Vec::new(),
    });

    // 10. Log (0x2C)
    let log_item = client
        .request()
        .log("audit-event")
        .to_batch_item()
        .expect("log builds");
    assert_eq!(log_item.operation(), 0x0000_002c);

    // 11-12. Login (0x2D) & DelegatedLogin (0x2F)
    let login_item = client
        .request()
        .login()
        .lease_time_interval(60)
        .request_count(5)
        .to_batch_item();
    assert_eq!(login_item.operation(), 0x0000_002d);

    let err = client
        .request()
        .delegated_login()
        .build()
        .expect_err("missing rights");
    assert_eq!(err, FluentBuildError::MissingDelegatedLoginRights);

    let del_item = client
        .request()
        .delegated_login()
        .rights(test_structure([]))
        .lease_time_date_time(1_700_000_100)
        .to_batch_item()
        .expect("delegated_login builds");
    assert_eq!(del_item.operation(), 0x0000_002f);

    // 13. SetEndpointRole (0x32)
    let role_item = client
        .request()
        .set_endpoint_role(EndpointRole::CLIENT)
        .vendor_identification("KMIPKit")
        .to_batch_item();
    assert_eq!(role_item.operation(), 0x0000_0032);

    // 14. PKCS#11 (0x33)
    let err = client
        .request()
        .pkcs11()
        .build()
        .expect_err("missing function");
    assert_eq!(err, FluentBuildError::MissingPkcs11Function);

    let mut fn_struct = Structure::new();
    fn_struct
        .try_push(item(KmipTag::PKCS11Function, Value::enumeration(1)))
        .expect("push fn");
    let pkcs11_item = client
        .request()
        .pkcs11()
        .function(fn_struct)
        .to_batch_item()
        .expect("pkcs11 builds");
    assert_eq!(pkcs11_item.operation(), 0x0000_0033);

    // 15. Interop (0x34)
    let interop_item = client
        .request()
        .interop(1, "interop-id")
        .to_batch_item()
        .expect("interop builds");
    assert_eq!(interop_item.operation(), 0x0000_0034);

    // 16. ReProvision (0x35)
    let reprov_item = client
        .request()
        .reprovision()
        .to_batch_item()
        .expect("reprovision builds");
    assert_eq!(reprov_item.operation(), 0x0000_0035);

    // 17. SetDefaults (0x36)
    let defaults_item = client
        .request()
        .set_defaults()
        .defaults_information(test_structure([]))
        .to_batch_item();
    assert_eq!(defaults_item.operation(), 0x0000_0036);

    // 18. SetConstraints (0x37)
    let err = client
        .request()
        .set_constraints()
        .build()
        .expect_err("missing constraints");
    assert_eq!(err, FluentBuildError::MissingSetConstraints);

    let constraints_item = client
        .request()
        .set_constraints()
        .constraints(test_structure([]))
        .to_batch_item()
        .expect("set_constraints builds");
    assert_eq!(constraints_item.operation(), 0x0000_0037);
}

#[test]
fn fluent_stage2_executes_typed_async_and_role_exchanges() {
    let cancel_payload = test_structure([
        item(KmipTag::AsynchronousCorrelationValue, Value::byte_string(vec![0xaa, 0xbb])),
        item(KmipTag::CancellationResult, Value::enumeration(1)),
    ]);
    let cancel_ok = asynchronous_response_bytes(0x0000_0019, 0, None, None, Some(cancel_payload));
    let (mut client, _, _) = client_for(ExchangeScript::Success {
        response: cancel_ok,
        request_write_chunks: Vec::new(),
    });
    let res = client
        .request()
        .cancel(SecretBytes::new(vec![0xaa, 0xbb]))
        .with_protocol_version(ProtocolVersion::V2_1)
        .execute()
        .expect("cancel executes");
    assert_eq!(res.status(), ResultStatus::from_raw(0));

    let role_payload = test_structure([item(KmipTag::EndpointRole, Value::enumeration(1))]);
    let role_ok = asynchronous_response_bytes(0x0000_0032, 0, None, None, Some(role_payload));
    let (mut role_client, _, _) = client_for(ExchangeScript::Success {
        response: role_ok,
        request_write_chunks: Vec::new(),
    });
    let role_resp = role_client
        .request()
        .set_endpoint_role(EndpointRole::CLIENT)
        .execute()
        .expect("set_endpoint_role executes");
    assert_eq!(role_resp.result().status(), ResultStatus::from_raw(0));
}
