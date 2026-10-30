#![allow(unsafe_code)]
#![cfg(feature = "coverage-c-consumer")]

use std::ffi::{CString, c_char};
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::net::{Shutdown, TcpListener};
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;

use kmipkit_ffi::kmipkit_codec_limits_t;
use kmipkit_test_support::EphemeralPki;
use kmipkit_ttlv::{Item, ValueView};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::server::WebPkiClientVerifier;
use rustls::{RootCertStore, ServerConfig, ServerConnection, StreamOwned};

const MAX_TEST_TTLV_VALUE_BYTES: usize = 8_388_608;
static TEMP_DIRECTORY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[link(name = "kmipkit_extension_registry_c_consumer", kind = "static")]
unsafe extern "C" {
    fn kmipkit_extension_registry_c_consumer_main(argc: i32, argv: *mut *mut c_char) -> i32;
    fn shared_schema_redaction_fixture_matches_fixture(
        diagnostic_marker: *const u8,
        diagnostic_marker_length: u64,
        expected_category: *const c_char,
    ) -> i32;
    fn shared_registry_fixture_matches_cleanup_and_isolation(
        alpha_name: *const u8,
        alpha_name_length: u64,
        beta_name: *const u8,
        beta_name_length: u64,
        expected_alpha_count: u64,
        expected_beta_count: u64,
        expected_registry_state_is_shared: u8,
        close_configurations_action: *const c_char,
        release_registry_views_action: *const c_char,
    ) -> i32;
}

#[link(name = "kmipkit_client_abi_c_consumer", kind = "static")]
unsafe extern "C" {
    fn kmipkit_client_abi_c_consumer_main(argc: i32, argv: *mut *mut c_char) -> i32;
    fn kmipkit_client_abi_c_consumer_async_output_errors_clear() -> i32;
    fn shared_ttlv_fixture_preserves_open_values_and_order(
        raw_tags: *const u32,
        enumeration_value: u32,
        byte_string: *const u8,
        byte_string_length: u64,
    ) -> i32;
    fn codec_limits_shared_fixture_matches_defaults_ranges_and_errors(
        expected_default_message_bytes: u64,
        expected_default_structure_depth: u64,
        expected_default_elements: u64,
        configured_message_bytes: u64,
        configured_structure_depth: u64,
        configured_elements: u64,
        invalid_structure_depth: u64,
    ) -> i32;
    fn kmipkit_client_abi_c_consumer_execute_tls(
        argc: i32,
        argv: *mut *mut c_char,
        limits: *mut kmipkit_codec_limits_t,
    ) -> i32;
    fn kmipkit_client_abi_c_consumer_execute_error_ttlv_tls(
        argc: i32,
        argv: *mut *mut c_char,
        limits: *mut kmipkit_codec_limits_t,
    ) -> i32;
    fn kmipkit_client_abi_c_consumer_execute_pkcs11_tls(
        argc: i32,
        argv: *mut *mut c_char,
        limits: *mut kmipkit_codec_limits_t,
    ) -> i32;
}

#[test]
fn c_abi_consumer_preserves_shared_ttlv_fixture_values_and_order() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/api/shared-capability-parity.json"
    ))
    .expect("the shared API fixture is valid JSON");
    let case = fixture["cases"]
        .as_array()
        .expect("fixture cases are an array")
        .iter()
        .find(|case| case["id"] == "ttlv.open-values-and-child-order.v1")
        .expect("the generic TTLV fixture case exists");
    let input_children = case["input"]["structure"]["children"]
        .as_array()
        .expect("generic TTLV input children are an array");
    let expected_tags = case["expectedOutput"]["childTagsInOrder"]
        .as_array()
        .expect("generic TTLV expected child tags are an array")
        .iter()
        .map(|tag| {
            u32::from_str_radix(
                tag.as_str()
                    .expect("generic TTLV child tag is a string")
                    .strip_prefix("0x")
                    .expect("generic TTLV child tag has a hexadecimal prefix"),
                16,
            )
            .expect("generic TTLV child tag is valid hexadecimal")
        })
        .collect::<Vec<_>>();
    let input_tags = input_children
        .iter()
        .map(|child| {
            u32::from_str_radix(
                child["tag"]
                    .as_str()
                    .expect("generic TTLV input tag is a string")
                    .strip_prefix("0x")
                    .expect("generic TTLV input tag has a hexadecimal prefix"),
                16,
            )
            .expect("generic TTLV input tag is valid hexadecimal")
        })
        .collect::<Vec<_>>();
    let enumeration_value = u32::try_from(
        input_children[0]["value"]
            .as_u64()
            .expect("generic TTLV enumeration input is an unsigned integer"),
    )
    .expect("generic TTLV enumeration fits the public fixed-width type");
    let expected_enumeration = u32::try_from(
        case["expectedOutput"]["enumerationValue"]
            .as_u64()
            .expect("generic TTLV expected enumeration is an unsigned integer"),
    )
    .expect("generic TTLV expected enumeration fits the public fixed-width type");
    let input_byte_string = input_children[1]["valueHex"]
        .as_str()
        .expect("generic TTLV byte-string input is hexadecimal");
    let byte_string = decode_fixture_hex(input_byte_string);
    let expected_byte_string = decode_fixture_hex(
        case["expectedOutput"]["byteStringHex"]
            .as_str()
            .expect("generic TTLV expected byte string is hexadecimal"),
    );

    assert_eq!(input_tags, expected_tags, "fixture input order is expected");
    assert_eq!(
        enumeration_value, expected_enumeration,
        "fixture enumeration input and output agree"
    );
    assert_eq!(
        byte_string, expected_byte_string,
        "fixture byte values agree"
    );
    assert_eq!(
        case["expectedOutput"]["unknownValuesPreserved"], true,
        "the fixture requires unknown values to be preserved"
    );

    // SAFETY: the fixture-backed tag and byte slices remain live for the
    // synchronous call; the C consumer creates, inspects, and releases its own handles.
    let result = unsafe {
        shared_ttlv_fixture_preserves_open_values_and_order(
            input_tags.as_ptr(),
            enumeration_value,
            byte_string.as_ptr(),
            u64::try_from(byte_string.len()).expect("fixture byte length fits in u64"),
        )
    };

    assert_eq!(
        result, 0,
        "the C ABI must preserve fixture values and child order"
    );
}

fn decode_fixture_hex(value: &str) -> Vec<u8> {
    let (pairs, remainder) = value.as_bytes().as_chunks::<2>();
    assert!(remainder.is_empty(), "fixture hexadecimal has whole bytes");
    pairs
        .iter()
        .map(|digits| {
            let pair = std::str::from_utf8(digits).expect("fixture hex digits are ASCII");
            u8::from_str_radix(pair, 16).expect("fixture byte is valid hexadecimal")
        })
        .collect()
}

#[test]
fn c_abi_consumer_matches_the_shared_codec_limits_fixture() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/api/shared-capability-parity.json"
    ))
    .expect("the shared API fixture is valid JSON");
    let case = fixture["cases"]
        .as_array()
        .expect("fixture cases are an array")
        .iter()
        .find(|case| case["id"] == "codec-limits.defaults-and-ranges.v1")
        .expect("the shared CodecLimits fixture case exists");
    let defaults = &case["expectedOutput"]["defaults"];
    let configured = &case["input"]["configured"];
    let invalid_depth = case["input"]["invalidDepth"]
        .as_u64()
        .expect("invalid fixture depth is unsigned");
    let error_category = case["expectedOutput"]["invalidDepth"]["errorCategory"]
        .as_str()
        .expect("invalid-depth fixture error category is text");
    assert_eq!(error_category, "resource_limit");

    // SAFETY: all values come from the parsed fixture and the C consumer creates
    // and releases each limits handle during this synchronous call.
    let result = unsafe {
        codec_limits_shared_fixture_matches_defaults_ranges_and_errors(
            defaults["maxMessageBytes"]
                .as_u64()
                .expect("default message limit is unsigned"),
            defaults["maxStructureDepth"]
                .as_u64()
                .expect("default depth is unsigned"),
            defaults["maxElements"]
                .as_u64()
                .expect("default element count is unsigned"),
            configured["maxMessageBytes"]
                .as_u64()
                .expect("configured message limit is unsigned"),
            configured["maxStructureDepth"]
                .as_u64()
                .expect("configured depth is unsigned"),
            configured["maxElements"]
                .as_u64()
                .expect("configured element count is unsigned"),
            invalid_depth,
        )
    };

    assert_eq!(result, 1, "the C ABI must match the CodecLimits fixture");
}

#[test]
fn c_abi_consumer_redacts_shared_fixture_diagnostic_marker() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/api/shared-capability-parity.json"
    ))
    .expect("the shared API fixture is valid JSON");
    let case = fixture["cases"]
        .as_array()
        .expect("fixture cases are an array")
        .iter()
        .find(|case| case["id"] == "errors.schema-redaction.v1")
        .expect("the shared schema-redaction fixture case exists");
    let marker = CString::new(
        case["input"]["diagnosticMarker"]
            .as_str()
            .expect("fixture diagnostic marker is text"),
    )
    .expect("fixture diagnostic marker contains no NUL byte");
    let expected = &case["expectedOutput"];
    let category = CString::new(
        expected["errorCategory"]
            .as_str()
            .expect("fixture error category is text"),
    )
    .expect("fixture error category contains no NUL byte");
    assert_eq!(case["errorCategory"], expected["errorCategory"]);
    assert_eq!(expected["diagnosticOmitsInputMarker"], true);
    assert_eq!(expected["diagnosticContainsPayload"], false);

    // SAFETY: the fixture marker and category strings remain live through the
    // synchronous C call; the compiled C consumer receives the marker as input.
    let result = unsafe {
        shared_schema_redaction_fixture_matches_fixture(
            marker.as_bytes().as_ptr(),
            u64::try_from(marker.as_bytes().len()).expect("marker length fits in u64"),
            category.as_ptr(),
        )
    };

    assert_eq!(
        result, 0,
        "the C ABI must return only the fixture's stable error category"
    );
}

#[test]
fn c_abi_consumer_matches_shared_registry_cleanup_fixture() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/api/shared-capability-parity.json"
    ))
    .expect("the shared API fixture is valid JSON");
    let case = fixture["cases"]
        .as_array()
        .expect("fixture cases are an array")
        .iter()
        .find(|case| case["id"] == "client-configuration.registry-isolation.v1")
        .expect("the shared registry cleanup fixture case exists");
    let configurations = case["input"]["configurations"]
        .as_array()
        .expect("fixture configurations are an array");
    let names = configurations
        .iter()
        .map(|configuration| {
            configuration["extensionIdentities"][0]
                .as_str()
                .expect("fixture extension identity is text")
        })
        .collect::<Vec<_>>();
    let expected = &case["expectedOutput"];
    let counts = expected["configurationDefinitionCounts"]
        .as_array()
        .expect("fixture definition counts are an array");
    let identities = expected["configurationIdentities"]
        .as_array()
        .expect("fixture configuration identities are an array");
    let cleanup = case["cleanup"]
        .as_array()
        .expect("fixture cleanup actions are an array");
    let close_configurations_action = CString::new(
        cleanup[0]
            .as_str()
            .expect("fixture configuration cleanup action is text"),
    )
    .expect("fixture cleanup action contains no NUL byte");
    let release_registry_views_action = CString::new(
        cleanup[1]
            .as_str()
            .expect("fixture registry-view cleanup action is text"),
    )
    .expect("fixture cleanup action contains no NUL byte");
    assert_eq!(names, ["alpha", "beta"]);
    assert_eq!(identities[0][0], names[0]);
    assert_eq!(identities[1][0], names[1]);
    assert_eq!(expected["registryStateIsShared"], false);
    assert_eq!(
        close_configurations_action.to_str().unwrap(),
        "close-configuration-owned-native-handles-once"
    );
    assert_eq!(
        release_registry_views_action.to_str().unwrap(),
        "release-returned-registry-views-once"
    );

    // SAFETY: fixture byte spans and C strings stay live for the synchronous C
    // consumer call; the consumer releases all handles it constructs.
    let result = unsafe {
        shared_registry_fixture_matches_cleanup_and_isolation(
            names[0].as_ptr(),
            u64::try_from(names[0].len()).expect("fixture identity length fits in u64"),
            names[1].as_ptr(),
            u64::try_from(names[1].len()).expect("fixture identity length fits in u64"),
            counts[0].as_u64().expect("fixture alpha count is unsigned"),
            counts[1].as_u64().expect("fixture beta count is unsigned"),
            u8::from(
                expected["registryStateIsShared"]
                    .as_bool()
                    .expect("fixture sharing value is Boolean"),
            ),
            close_configurations_action.as_ptr(),
            release_registry_views_action.as_ptr(),
        )
    };

    assert_eq!(
        result, 0,
        "the C ABI must match registry fixture values and cleanup"
    );
}

#[link(name = "kmipkit_ffi")]
unsafe extern "C" {
    fn kmipkit_codec_limits_defaults(out_limits: *mut *mut kmipkit_codec_limits_t) -> i32;
    fn kmipkit_codec_limits_create(
        max_message_bytes: u64,
        max_structure_depth: u64,
        max_elements: u64,
        out_limits: *mut *mut kmipkit_codec_limits_t,
    ) -> i32;
    fn kmipkit_codec_limits_release(limits: *mut kmipkit_codec_limits_t);
    fn kmipkit_last_error_get(
        out_status: *mut i32,
        out_category: *mut u32,
        out_cause: *mut u32,
        out_delivery: *mut u32,
    ) -> i32;
}

#[test]
fn existing_c_consumer_exercises_the_exported_abi_in_the_coverage_process() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/extensions/cases.json")
        .canonicalize()
        .expect("shared C consumer fixture exists");
    let executable = CString::new("kmipkit_extension_registry_consumer")
        .expect("test executable name has no NUL byte");
    let fixture = CString::new(fixture.to_str().expect("fixture path is UTF-8"))
        .expect("fixture path has no NUL byte");
    let pki = EphemeralPki::generate().expect("ephemeral test PKI is generated");
    let client_chain = pki.client_identity().certificate_chain_der();
    let paths = TemporaryCredentialFiles::create([
        client_chain[0],
        client_chain[1],
        pki.client_identity().private_key_der(),
        pki.authority_certificate_der(),
    ]);
    let certificate = CString::new(paths.paths[0].to_string_lossy().as_bytes())
        .expect("temporary path has no NUL byte");
    let authority = CString::new(paths.paths[1].to_string_lossy().as_bytes())
        .expect("temporary path has no NUL byte");
    let private_key = CString::new(paths.paths[2].to_string_lossy().as_bytes())
        .expect("temporary path has no NUL byte");
    let trust_root = CString::new(paths.paths[3].to_string_lossy().as_bytes())
        .expect("temporary path has no NUL byte");
    let listener =
        TcpListener::bind(("127.0.0.1", 0)).expect("an ephemeral local TCP port is available");
    let port = listener
        .local_addr()
        .expect("listener has a local address")
        .port();
    let port = CString::new(port.to_string()).expect("port has no NUL byte");
    let server_config = mtls_server_config(&pki);
    let retry_server_config = Arc::clone(&server_config);
    let peer = thread::spawn(move || {
        controlled_mtls_peer(&listener, server_config, retry_server_config);
    });
    let mut arguments = [
        executable.as_ptr().cast_mut(),
        fixture.as_ptr().cast_mut(),
        certificate.as_ptr().cast_mut(),
        authority.as_ptr().cast_mut(),
        private_key.as_ptr().cast_mut(),
        trust_root.as_ptr().cast_mut(),
        port.as_ptr().cast_mut(),
    ];
    let argument_count =
        i32::try_from(arguments.len()).expect("the C consumer argument count fits in i32");
    let mut limits = std::ptr::null_mut();

    // SAFETY: `limits` is a valid writable output slot for the duration of this call.
    let status = unsafe { kmipkit_codec_limits_defaults(&raw mut limits) };
    assert_eq!(status, 0, "codec limits defaults must succeed");
    assert!(
        !limits.is_null(),
        "codec limits defaults must return an owned handle"
    );

    // SAFETY: a successful defaults call returned this live owned handle, which is released once.
    unsafe {
        kmipkit_codec_limits_release(limits);
    }

    // SAFETY: both C strings and the mutable argument array remain live for the entire call.
    let result = unsafe {
        kmipkit_extension_registry_c_consumer_main(argument_count, arguments.as_mut_ptr())
    };

    assert_eq!(result, 0, "the existing C ABI behavior suite must pass");
    peer.join().expect("the loopback TLS peer completes");
}

#[test]
fn generated_public_client_abi_consumer_compiles_and_runs() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/api/shared-capability-parity.json"
    ))
    .expect("the shared API fixture is valid JSON");
    let case = fixture["cases"]
        .as_array()
        .expect("fixture cases are an array")
        .iter()
        .find(|case| case["id"] == "api-levels.shared.v1")
        .expect("the API-level fixture case exists");
    let expected_output = &case["expectedOutput"];
    assert_eq!(case["capability"], "api_level_discovery");
    assert_eq!(expected_output["networkCalls"], 0);
    assert_eq!(
        expected_output["distinctFromKmipOperation"],
        "Discover Versions"
    );
    assert_eq!(
        expected_output["operationMappingCompleteness"],
        "not-asserted"
    );
    let expected_levels = expected_output["apiLevels"]
        .as_array()
        .expect("fixture API levels are an array")
        .iter()
        .map(|level| {
            CString::new(level.as_str().expect("API level IDs are strings"))
                .expect("API level IDs have no NUL byte")
        })
        .collect::<Vec<_>>();
    assert_eq!(expected_levels.len(), 3);
    let executable = CString::new("kmipkit_public_client_consumer")
        .expect("test executable name has no NUL byte");
    let mut arguments = vec![executable.as_ptr().cast_mut()];
    arguments.extend(
        expected_levels
            .iter()
            .map(|level| level.as_ptr().cast_mut()),
    );
    let argument_count = i32::try_from(arguments.len()).expect("argument count fits in i32");
    // SAFETY: the compiled consumer includes the generated public header, receives
    // one live argv string, and releases every created handle before returning.
    let result =
        unsafe { kmipkit_client_abi_c_consumer_main(argument_count, arguments.as_mut_ptr()) };

    assert_eq!(
        result, 0,
        "generated public C client ABI consumer must pass its implementation checks"
    );
}

#[test]
fn generated_public_async_client_abi_consumer_compiles_and_runs() {
    // SAFETY: the C consumer uses null handles to exercise documented invalid-input
    // paths and verifies that every output is cleared before returning.
    let result = unsafe { kmipkit_client_abi_c_consumer_async_output_errors_clear() };

    assert_eq!(
        result, 1,
        "the compiled C consumer must call every async ABI export and observe cleared outputs"
    );
}

#[test]
fn compiled_c_pkcs11_consumer_matches_shared_response_fixture_over_mtls() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/api/pkcs11-parity.json"
    ))
    .expect("the shared PKCS#11 parity fixture is valid JSON");
    let expected = &fixture["expectedOutput"];
    let mut correlation_cases = vec!["nonempty".to_owned()];
    correlation_cases.extend(
        expected["correlationValuePresenceCases"]
            .as_object()
            .expect("fixture correlation presence cases are an object")
            .keys()
            .cloned(),
    );
    for correlation_case in correlation_cases {
        compiled_c_pkcs11_consumer_matches_correlation_case(&correlation_case);
    }
}

#[allow(clippy::too_many_lines)]
fn compiled_c_pkcs11_consumer_matches_correlation_case(correlation_case: &str) {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/api/pkcs11-parity.json"
    ))
    .expect("the shared PKCS#11 parity fixture is valid JSON");
    let expected = &fixture["expectedOutput"];
    let expected_tags = expected["responseTags"]
        .as_array()
        .expect("fixture response tags are an array")
        .iter()
        .map(|tag| {
            u32::from_str_radix(
                tag.as_str()
                    .expect("fixture response tags are strings")
                    .strip_prefix("0x")
                    .expect("fixture response tags have a hexadecimal prefix"),
                16,
            )
            .expect("fixture response tag is hexadecimal")
        })
        .collect::<Vec<_>>();
    let expected_return_code = u32::from_str_radix(
        expected["unknownReturnCode"]
            .as_str()
            .expect("fixture return code is text")
            .strip_prefix("0x")
            .expect("fixture return code has a hexadecimal prefix"),
        16,
    )
    .expect("fixture return code is hexadecimal");
    let expected_vendor_tag = u32::from_str_radix(
        expected["vendorExtension"]["tag"]
            .as_str()
            .expect("fixture vendor tag is text")
            .strip_prefix("0x")
            .expect("fixture vendor tag has a hexadecimal prefix"),
        16,
    )
    .expect("fixture vendor tag is hexadecimal");
    let expected_vendor_value = expected["vendorExtension"]["integer"]
        .as_i64()
        .and_then(|value| i32::try_from(value).ok())
        .expect("fixture vendor integer fits the public C integer type");
    let typed = &expected["typedResponse"];
    let function_tag = fixture_hex_u32(&typed["function"]["childTag"]);
    let function_value = i32::try_from(
        typed["function"]["childValue"]
            .as_i64()
            .expect("fixture function value is an integer"),
    )
    .expect("fixture function value fits the public C integer type");
    let (expected_correlation_present, correlation_hex) =
        fixture_pkcs11_correlation_value(expected, correlation_case);
    let correlation_tag = fixture_hex_u32(&typed["correlationValue"]["tag"]);
    let output_tag = fixture_hex_u32(&typed["outputParameters"]["childTag"]);
    let output_hex = typed["outputParameters"]["childHex"]
        .as_str()
        .expect("fixture output parameters are hexadecimal");
    let return_code_tag = fixture_hex_u32(&typed["returnCode"]["childTag"]);
    assert_eq!(expected_tags.len(), 5);
    assert_eq!(expected["preserveUnknownValuesAndOrder"], true);
    assert_eq!(expected["preserveOptionalAbsence"], true);
    assert_eq!(expected["automaticRetry"], false);
    assert_eq!(expected["automaticFollowUp"], false);

    let pki = EphemeralPki::generate().expect("ephemeral test PKI is generated");
    let client_chain = pki.client_identity().certificate_chain_der();
    let paths = TemporaryCredentialFiles::create([
        client_chain[0],
        client_chain[1],
        pki.client_identity().private_key_der(),
        pki.authority_certificate_der(),
    ]);
    let listener =
        TcpListener::bind(("127.0.0.1", 0)).expect("an ephemeral local TCP port is available");
    let port = listener
        .local_addr()
        .expect("listener has a local address")
        .port();
    let response_bytes = pkcs11_success_response_message(expected, correlation_case);
    let server_config = mtls_server_config(&pki);
    let exchange_count = Arc::new(AtomicU64::new(0));
    let peer_exchange_count = Arc::clone(&exchange_count);
    let peer = thread::spawn(move || {
        let (stream, _) = listener
            .accept()
            .expect("the compiled C client connects to the loopback peer");
        peer_exchange_count.fetch_add(1, Ordering::Relaxed);
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .expect("server read timeout is configured");
        stream
            .set_write_timeout(Some(std::time::Duration::from_secs(5)))
            .expect("server write timeout is configured");
        let connection = ServerConnection::new(server_config)
            .expect("the TLS 1.3 server connection is configured");
        let mut tls = StreamOwned::new(connection, stream);
        let request = read_ttlv_frame(&mut tls).expect("bounded PKCS#11 request arrives");
        assert!(
            c_request_matches_pkcs11_fixture(&request),
            "C request contains one ordered PKCS#11 operation with the fixture inputs"
        );
        tls.write_all(&response_bytes)
            .expect("the fixture-backed KMIP response is written");
        tls.flush().expect("the TLS response is flushed");
        let _ = tls.get_ref().shutdown(Shutdown::Both);
    });

    let executable =
        CString::new("kmipkit_pkcs11_c_consumer").expect("test executable name has no NUL byte");
    let certificate = CString::new(paths.paths[0].to_string_lossy().as_bytes())
        .expect("temporary certificate path has no NUL byte");
    let authority = CString::new(paths.paths[1].to_string_lossy().as_bytes())
        .expect("temporary authority path has no NUL byte");
    let private_key = CString::new(paths.paths[2].to_string_lossy().as_bytes())
        .expect("temporary key path has no NUL byte");
    let trust_root = CString::new(paths.paths[3].to_string_lossy().as_bytes())
        .expect("temporary trust root path has no NUL byte");
    let port = CString::new(port.to_string()).expect("port has no NUL byte");
    let expected_tag_arguments = expected_tags
        .iter()
        .map(|tag| CString::new(format!("0x{tag:08X}")))
        .collect::<Result<Vec<_>, _>>()
        .expect("fixture tags have no NUL bytes");
    let return_code = CString::new(format!("0x{expected_return_code:08X}"))
        .expect("fixture return code has no NUL byte");
    let vendor_tag = CString::new(format!("0x{expected_vendor_tag:08X}"))
        .expect("fixture vendor tag has no NUL byte");
    let vendor_value = CString::new(expected_vendor_value.to_string())
        .expect("fixture vendor value has no NUL byte");
    let function_tag = CString::new(format!("0x{function_tag:08X}"))
        .expect("fixture function tag has no NUL byte");
    let function_value =
        CString::new(function_value.to_string()).expect("fixture function value has no NUL byte");
    let correlation_hex =
        CString::new(correlation_hex).expect("fixture correlation hexadecimal has no NUL byte");
    let correlation_tag = CString::new(format!("0x{correlation_tag:08X}"))
        .expect("fixture correlation tag has no NUL byte");
    let correlation_present = CString::new(if expected_correlation_present {
        "1"
    } else {
        "0"
    })
    .expect("correlation presence flag has no NUL byte");
    let output_tag =
        CString::new(format!("0x{output_tag:08X}")).expect("fixture output tag has no NUL byte");
    let output_hex = CString::new(output_hex).expect("fixture output hexadecimal has no NUL byte");
    let return_code_tag = CString::new(format!("0x{return_code_tag:08X}"))
        .expect("fixture return-code tag has no NUL byte");
    let mut arguments = vec![
        executable.as_ptr().cast_mut(),
        certificate.as_ptr().cast_mut(),
        authority.as_ptr().cast_mut(),
        private_key.as_ptr().cast_mut(),
        trust_root.as_ptr().cast_mut(),
        port.as_ptr().cast_mut(),
    ];
    arguments.extend(
        expected_tag_arguments
            .iter()
            .map(|value| value.as_ptr().cast_mut()),
    );
    arguments.push(return_code.as_ptr().cast_mut());
    arguments.push(vendor_tag.as_ptr().cast_mut());
    arguments.push(vendor_value.as_ptr().cast_mut());
    arguments.push(function_tag.as_ptr().cast_mut());
    arguments.push(function_value.as_ptr().cast_mut());
    arguments.push(correlation_hex.as_ptr().cast_mut());
    arguments.push(output_tag.as_ptr().cast_mut());
    arguments.push(output_hex.as_ptr().cast_mut());
    arguments.push(return_code_tag.as_ptr().cast_mut());
    arguments.push(correlation_tag.as_ptr().cast_mut());
    arguments.push(correlation_present.as_ptr().cast_mut());
    let argument_count = i32::try_from(arguments.len()).expect("C argv count fits in i32");
    let mut limits = std::ptr::null_mut();
    // SAFETY: `limits` is a writable output slot held through the synchronous C call.
    let status = unsafe { kmipkit_codec_limits_defaults(&raw mut limits) };
    assert_eq!(status, 0);
    assert!(!limits.is_null());
    // SAFETY: all argv strings and the codec-limits handle remain live for the synchronous C call.
    let result = unsafe {
        kmipkit_client_abi_c_consumer_execute_pkcs11_tls(
            argument_count,
            arguments.as_mut_ptr(),
            limits,
        )
    };
    // SAFETY: the test owns this limits handle and the C consumer only borrows it.
    unsafe { kmipkit_codec_limits_release(limits) };
    peer.join().expect("the loopback mTLS peer completes");
    assert_eq!(
        result, 0,
        "the compiled public C consumer verifies the {correlation_case} correlation fixture"
    );
    assert_eq!(exchange_count.load(Ordering::Relaxed), 1);
}

#[test]
// Keep PKI, argv, and the synchronous C ABI call together for this mTLS harness.
#[allow(clippy::too_many_lines)]
fn generated_public_c_consumer_maps_rekey_key_pair_request_over_mtls() {
    let pki = EphemeralPki::generate().expect("ephemeral test PKI is generated");
    let client_chain = pki.client_identity().certificate_chain_der();
    let paths = TemporaryCredentialFiles::create([
        client_chain[0],
        client_chain[1],
        pki.client_identity().private_key_der(),
        pki.authority_certificate_der(),
    ]);
    let listener =
        TcpListener::bind(("127.0.0.1", 0)).expect("an ephemeral local TCP port is available");
    let port = listener
        .local_addr()
        .expect("listener has a local address")
        .port();
    let server_config = mtls_server_config(&pki);
    let exchange_count = Arc::new(AtomicU64::new(0));
    let peer_exchange_count = Arc::clone(&exchange_count);
    let peer = thread::spawn(move || {
        let (stream, _) = listener
            .accept()
            .expect("the compiled C client connects to the loopback peer");
        peer_exchange_count.fetch_add(1, Ordering::Relaxed);
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .expect("server read timeout is configured");
        stream
            .set_write_timeout(Some(std::time::Duration::from_secs(5)))
            .expect("server write timeout is configured");
        let connection = ServerConnection::new(server_config)
            .expect("the TLS 1.3 server connection is configured");
        let mut tls = StreamOwned::new(connection, stream);
        let request = read_ttlv_frame(&mut tls).expect("bounded Re-key Key Pair request arrives");
        let request_issue = c_request_rekey_key_pair_issue(&request);
        assert_eq!(
            request_issue, "",
            "C ABI request mapping check failed: {request_issue}"
        );
        tls.write_all(&rekey_key_pair_success_response())
            .expect("the Re-key Key Pair response is written");
        tls.flush().expect("the TLS response is flushed");
        let _ = tls.get_ref().shutdown(Shutdown::Both);
    });

    let executable = CString::new("kmipkit_rekey_key_pair_c_consumer")
        .expect("test executable name has no NUL byte");
    let certificate = CString::new(paths.paths[0].to_string_lossy().as_bytes())
        .expect("temporary certificate path has no NUL byte");
    let authority = CString::new(paths.paths[1].to_string_lossy().as_bytes())
        .expect("temporary authority path has no NUL byte");
    let private_key = CString::new(paths.paths[2].to_string_lossy().as_bytes())
        .expect("temporary key path has no NUL byte");
    let trust_root = CString::new(paths.paths[3].to_string_lossy().as_bytes())
        .expect("temporary trust root path has no NUL byte");
    let port = CString::new(port.to_string()).expect("port has no NUL byte");
    let mode = CString::new("rekey-key-pair").expect("mode has no NUL byte");
    let mut arguments = vec![
        executable.as_ptr().cast_mut(),
        certificate.as_ptr().cast_mut(),
        authority.as_ptr().cast_mut(),
        private_key.as_ptr().cast_mut(),
        trust_root.as_ptr().cast_mut(),
        port.as_ptr().cast_mut(),
    ];
    arguments.push(mode.as_ptr().cast_mut());
    let argument_count = i32::try_from(arguments.len()).expect("C argv count fits in i32");
    let mut limits = std::ptr::null_mut();
    // SAFETY: `limits` is a writable output slot held through the synchronous C call.
    let status =
        unsafe { kmipkit_codec_limits_create(16 * 1024 * 1024, 64, 400_000, &raw mut limits) };
    assert_eq!(status, 0, "configured codec limits must succeed");
    assert!(!limits.is_null(), "configured codec limits return a handle");
    // SAFETY: argv values and the codec-limits handle remain live for the synchronous C call.
    let result = unsafe {
        kmipkit_client_abi_c_consumer_execute_tls(argument_count, arguments.as_mut_ptr(), limits)
    };
    // SAFETY: the test owns this limits handle and the C consumer only borrows it.
    unsafe { kmipkit_codec_limits_release(limits) };
    peer.join()
        .expect("the Re-key Key Pair loopback mTLS peer completes");
    assert_eq!(
        result, 0,
        "the compiled C consumer completes the typed request"
    );
    assert_eq!(exchange_count.load(Ordering::Relaxed), 1);
}

#[test]
// OASIS KMIP v2.1 §6.1.10, Tables 193–195; §5.1, Table 157.
// Keep the TLS peer, certificate files, argv, and C ABI call in one runtime test.
#[allow(clippy::too_many_lines)]
fn generated_public_client_abi_consumer_maps_split_key_fixture_success_and_pending_over_mtls() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/api/shared-capability-parity.json"
    ))
    .expect("the shared API fixture is valid JSON");
    let case = fixture["cases"]
        .as_array()
        .expect("fixture cases are an array")
        .iter()
        .find(|case| case["id"] == "creation.split-key.request-response.v1")
        .expect("Create Split Key fixture case exists");
    let input = &case["input"];
    let expected = &case["expectedOutput"];
    let pending_expected = &expected["pendingMetadata"];
    assert_eq!(input["operation"], "Create Split Key");
    assert_eq!(input["attributesPresent"], true);
    assert_eq!(input["protectionStorageMasks"]["present"], true);
    assert_eq!(expected["primeFieldSizePresent"], false);
    assert_eq!(expected["automaticFollowUp"], false);

    let response_identifiers = expected["responseUniqueIdentifiersInWireOrder"]
        .as_array()
        .expect("fixture response identifiers are an array")
        .iter()
        .map(|identifier| match identifier["wireType"].as_str() {
            Some("TextString") => ttlv_item(
                0x0042_0094,
                0x07,
                identifier["text"]
                    .as_str()
                    .expect("TextString fixture identifier has text")
                    .as_bytes(),
            ),
            Some("Enumeration") => enumeration(
                0x0042_0094,
                u32::try_from(
                    identifier["enumeration"]
                        .as_u64()
                        .expect("Enumeration fixture identifier is U32"),
                )
                .expect("fixture identifier fits Enumeration"),
            ),
            Some("Integer") => integer(
                0x0042_0094,
                i32::try_from(
                    identifier["integer"]
                        .as_i64()
                        .expect("Integer fixture identifier is signed"),
                )
                .expect("fixture identifier fits Integer"),
            ),
            other => panic!("unsupported fixture identifier type {other:?}"),
        })
        .collect::<Vec<_>>();
    let pending_response_identifiers = expected["pendingResponseUniqueIdentifiersInWireOrder"]
        .as_array()
        .expect("fixture Pending response identifiers are an array")
        .iter()
        .map(|identifier| match identifier["wireType"].as_str() {
            Some("TextString") => ttlv_item(
                0x0042_0094,
                0x07,
                identifier["text"]
                    .as_str()
                    .expect("TextString fixture identifier has text")
                    .as_bytes(),
            ),
            Some("Enumeration") => enumeration(
                0x0042_0094,
                u32::try_from(
                    identifier["enumeration"]
                        .as_u64()
                        .expect("Enumeration fixture identifier is U32"),
                )
                .expect("fixture identifier fits Enumeration"),
            ),
            Some("Integer") => integer(
                0x0042_0094,
                i32::try_from(
                    identifier["integer"]
                        .as_i64()
                        .expect("Integer fixture identifier is signed"),
                )
                .expect("fixture identifier fits Integer"),
            ),
            other => panic!("unsupported fixture Pending identifier type {other:?}"),
        })
        .collect::<Vec<_>>();
    let success_payload = structure(0x0042_007c, response_identifiers);
    let success_item = structure(
        0x0042_000f,
        [
            enumeration(0x0042_005c, 40),
            ttlv_item(0x0042_0093, 0x08, b"split-success-red"),
            enumeration(
                0x0042_007f,
                u32::try_from(
                    expected["resultStatus"]
                        .as_u64()
                        .expect("fixture success status is U32"),
                )
                .expect("status fits U32"),
            ),
            success_payload,
        ],
    );
    let correlation = decode_fixture_hex(
        pending_expected["correlationValueHex"]
            .as_str()
            .expect("fixture pending correlation is hexadecimal"),
    );
    let pending_item = structure(
        0x0042_000f,
        [
            enumeration(
                0x0042_005c,
                u32::try_from(
                    pending_expected["sourceOperation"]
                        .as_u64()
                        .expect("fixture operation is U32"),
                )
                .expect("operation fits U32"),
            ),
            ttlv_item(0x0042_0093, 0x08, b"split-pending"),
            enumeration(
                0x0042_007f,
                u32::try_from(
                    pending_expected["status"]
                        .as_u64()
                        .expect("fixture pending status is U32"),
                )
                .expect("status fits U32"),
            ),
            ttlv_item(0x0042_0006, 0x08, &correlation),
            structure(0x0042_007c, pending_response_identifiers),
        ],
    );
    let version = structure(
        0x0042_0069,
        [integer(0x0042_006a, 2), integer(0x0042_006b, 1)],
    );
    let header = structure(
        0x0042_007a,
        [version, date_time(0x0042_0092, 1), integer(0x0042_000d, 2)],
    );
    let response = structure(0x0042_007b, [header, success_item, pending_item]);

    let object_type = u32::try_from(
        input["objectType"]
            .as_u64()
            .expect("fixture Object Type is U32"),
    )
    .expect("Object Type fits U32");
    let unique_identifier = i32::try_from(
        input["uniqueIdentifier"]["integer"]
            .as_i64()
            .expect("fixture Unique Identifier is Integer"),
    )
    .expect("Unique Identifier fits Integer");
    let split_key_parts = i32::try_from(
        input["splitKeyParts"]
            .as_i64()
            .expect("fixture parts are Integer"),
    )
    .expect("parts fit Integer");
    let split_key_threshold = i32::try_from(
        input["splitKeyThreshold"]
            .as_i64()
            .expect("fixture threshold is Integer"),
    )
    .expect("threshold fits Integer");
    let split_key_method = u32::try_from(
        input["splitKeyMethod"]
            .as_u64()
            .expect("fixture method is Enumeration"),
    )
    .expect("method fits Enumeration");
    let expected_request_payload = structure(
        0x0042_0079,
        [
            enumeration(0x0042_0057, object_type),
            integer(0x0042_0094, unique_identifier),
            integer(0x0042_008b, split_key_parts),
            integer(0x0042_008c, split_key_threshold),
            enumeration(0x0042_008a, split_key_method),
            structure(0x0042_0125, []),
            structure(0x0042_015f, []),
        ],
    );

    let pki = EphemeralPki::generate().expect("ephemeral test PKI is generated");
    let client_chain = pki.client_identity().certificate_chain_der();
    let paths = TemporaryCredentialFiles::create([
        client_chain[0],
        client_chain[1],
        pki.client_identity().private_key_der(),
        pki.authority_certificate_der(),
    ]);
    let listener =
        TcpListener::bind(("127.0.0.1", 0)).expect("an ephemeral local TCP port is available");
    let port = listener
        .local_addr()
        .expect("listener has a local address")
        .port();
    let server_config = mtls_server_config(&pki);
    let exchange_count = Arc::new(AtomicU64::new(0));
    let peer_exchange_count = Arc::clone(&exchange_count);
    let peer = thread::spawn(move || {
        listener
            .set_nonblocking(true)
            .expect("fixture peer can bound its accept wait");
        let accept_deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        let (stream, _) = loop {
            match listener.accept() {
                Ok(connection) => break connection,
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    assert!(
                        std::time::Instant::now() < accept_deadline,
                        "the compiled C consumer connects to the fixture peer before timeout"
                    );
                    thread::sleep(std::time::Duration::from_millis(10));
                }
                Err(error) => panic!("fixture peer accept failed: {error}"),
            }
        };
        stream
            .set_nonblocking(false)
            .expect("accepted fixture connection uses blocking reads");
        peer_exchange_count.fetch_add(1, Ordering::Relaxed);
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .expect("server read timeout is configured");
        stream
            .set_write_timeout(Some(std::time::Duration::from_secs(5)))
            .expect("server write timeout is configured");
        let connection = ServerConnection::new(server_config)
            .expect("the TLS 1.3 server connection is configured");
        let mut tls = StreamOwned::new(connection, stream);
        let request = read_ttlv_frame(&mut tls).expect("the C batch request arrives");
        let occurrences = request
            .windows(expected_request_payload.len())
            .filter(|window| *window == expected_request_payload)
            .count();
        assert_eq!(
            occurrences, 2,
            "the C request contains two exact fixture-shaped Create Split Key payloads"
        );
        tls.write_all(&response)
            .expect("the fixture success and Pending results are written");
        tls.flush().expect("the TLS response is flushed");
        let _ = tls.get_ref().shutdown(Shutdown::Both);
    });

    let executable = CString::new("kmipkit_split_key_fixture_c_consumer")
        .expect("test executable name has no NUL byte");
    let certificate = CString::new(paths.paths[0].to_string_lossy().as_bytes())
        .expect("temporary certificate path has no NUL byte");
    let authority = CString::new(paths.paths[1].to_string_lossy().as_bytes())
        .expect("temporary authority path has no NUL byte");
    let private_key = CString::new(paths.paths[2].to_string_lossy().as_bytes())
        .expect("temporary key path has no NUL byte");
    let trust_root = CString::new(paths.paths[3].to_string_lossy().as_bytes())
        .expect("temporary trust root path has no NUL byte");
    let port = CString::new(port.to_string()).expect("port has no NUL byte");
    let mode = CString::new("create-split-key-fixture").expect("fixture mode has no NUL byte");
    let mut arguments = vec![
        executable.as_ptr().cast_mut(),
        certificate.as_ptr().cast_mut(),
        authority.as_ptr().cast_mut(),
        private_key.as_ptr().cast_mut(),
        trust_root.as_ptr().cast_mut(),
        port.as_ptr().cast_mut(),
        mode.as_ptr().cast_mut(),
    ];
    let argument_count = i32::try_from(arguments.len()).expect("C argv count fits in i32");
    let mut limits = std::ptr::null_mut();
    // SAFETY: `limits` is a writable output slot held through the synchronous C call.
    let status =
        unsafe { kmipkit_codec_limits_create(16 * 1024 * 1024, 64, 400_000, &raw mut limits) };
    assert_eq!(status, 0, "configured codec limits must succeed");
    assert!(!limits.is_null(), "configured codec limits return a handle");
    // SAFETY: argv values and the codec-limits handle remain live for the synchronous C call.
    let result = unsafe {
        kmipkit_client_abi_c_consumer_execute_tls(argument_count, arguments.as_mut_ptr(), limits)
    };
    // SAFETY: the test owns this limits handle and the C consumer only borrows it.
    unsafe { kmipkit_codec_limits_release(limits) };
    peer.join()
        .expect("the Create Split Key mTLS peer completes");
    assert_eq!(
        result, 0,
        "the C ABI preserves fixture identifiers and Pending metadata over mTLS"
    );
    assert_eq!(exchange_count.load(Ordering::Relaxed), 1);
}

#[test]
fn generated_public_client_abi_consumer_executes_a_typed_batch_over_mtls() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/api/shared-capability-parity.json"
    ))
    .expect("shared API parity fixture is valid JSON");
    let create_case = fixture["cases"]
        .as_array()
        .expect("shared fixture cases are an array")
        .iter()
        .find(|case| case["id"] == "creation.create.request-response.v1")
        .expect("shared Create fixture case exists");
    let create_expected = &create_case["expectedOutput"];
    assert_eq!(create_expected["responseObjectType"].as_u64(), Some(7));
    assert_eq!(create_expected["uniqueIdentifier"]["wireType"], "Integer");
    assert_eq!(
        create_expected["uniqueIdentifier"]["integer"].as_i64(),
        Some(-8)
    );
    assert_eq!(create_expected["resultStatus"].as_u64(), Some(0));
    assert!(create_expected["resultMessage"].is_null());

    let creation_failure_case = creation_result_message_fixture_case();

    run_generated_public_client_abi_consumer(
        &creation_failure_case,
        CreationResultMessageExpectation::MatchesFixture,
    );
}

#[test]
fn compiled_c_consumer_rejects_divergent_creation_message_input_and_expected_output() {
    let mut creation_failure_case = creation_result_message_fixture_case();
    creation_failure_case["input"]["resultMessage"] =
        serde_json::json!("server result differs from fixture expectation");
    assert_ne!(
        creation_failure_case["input"]["resultMessage"],
        creation_failure_case["expectedOutput"]["resultMessage"],
        "the test must present separate server input and expected output values"
    );

    run_generated_public_client_abi_consumer(
        &creation_failure_case,
        CreationResultMessageExpectation::RejectsDivergence,
    );
}

#[derive(Clone, Copy)]
enum CreationResultMessageExpectation {
    MatchesFixture,
    RejectsDivergence,
}

struct CreateKeyPairParityFixture {
    input: serde_json::Value,
    expected_output: serde_json::Value,
}

fn creation_result_message_fixture_case() -> serde_json::Value {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/api/shared-capability-parity.json"
    ))
    .expect("the shared API fixture is valid JSON");
    fixture["cases"]
        .as_array()
        .expect("fixture cases are an array")
        .iter()
        .find(|case| case["id"] == "creation.result-message-on-failure.v1")
        .expect("the creation failure fixture case exists")
        .clone()
}

// Keep the fixture-backed argv values and TLS server alive through one visible FFI call.
#[allow(clippy::too_many_lines)]
fn run_generated_public_client_abi_consumer(
    creation_failure_case_override: &serde_json::Value,
    message_expectation: CreationResultMessageExpectation,
) {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/api/shared-capability-parity.json"
    ))
    .expect("the shared API fixture is valid JSON");
    let case = fixture["cases"]
        .as_array()
        .expect("the shared fixture cases are an array")
        .iter()
        .find(|case| case["id"] == "batch.ordered-completed-pending-failed.v1")
        .expect("the ordered batch fixture case exists");
    let discover_versions_case = fixture["cases"]
        .as_array()
        .expect("fixture cases are an array")
        .iter()
        .find(|case| case["id"] == "discover-versions.supported-versions.v1")
        .expect("the Discover Versions fixture case exists");
    let create_case = fixture["cases"]
        .as_array()
        .expect("fixture cases are an array")
        .iter()
        .find(|case| case["id"] == "creation.create.request-response.v1")
        .expect("the Create fixture case exists");
    let create_key_pair_case = fixture["cases"]
        .as_array()
        .expect("the shared fixture cases are an array")
        .iter()
        .find(|case| case["id"] == "creation.key-pair.request-response.v1")
        .expect("the Create Key Pair fixture case exists");
    let create_key_pair_expected = &create_key_pair_case["expectedOutput"];
    assert_eq!(create_key_pair_expected["resultStatus"].as_u64(), Some(0));
    assert!(create_key_pair_expected["resultMessage"].is_null());
    assert!(
        create_key_pair_expected["resultMessagePresencePreserved"]
            .as_bool()
            .expect("fixture result-message presence is Boolean")
    );
    let create_input = create_case["input"].clone();
    let creation_failure_case = creation_failure_case_override;
    let server_result_message = CString::new(
        creation_failure_case["input"]["resultMessage"]
            .as_str()
            .expect("fixture server result message is text"),
    )
    .expect("fixture server result message contains no NUL byte");
    let expected_result_message = CString::new(
        creation_failure_case["expectedOutput"]["resultMessage"]
            .as_str()
            .expect("fixture expected result message is text"),
    )
    .expect("fixture expected result message contains no NUL byte");
    let discover_expected = &discover_versions_case["expectedOutput"];
    let fixture_versions = discover_expected["supportedVersions"]
        .as_array()
        .expect("fixture supported versions are an array")
        .iter()
        .map(|version| {
            (
                i32::try_from(
                    version["major"]
                        .as_i64()
                        .expect("fixture major version is an integer"),
                )
                .expect("fixture major version fits i32"),
                i32::try_from(
                    version["minor"]
                        .as_i64()
                        .expect("fixture minor version is an integer"),
                )
                .expect("fixture minor version fits i32"),
            )
        })
        .collect::<Vec<_>>();
    assert!(
        discover_expected["preservesSourceOrderAndRepetitions"]
            .as_bool()
            .expect("fixture repetition contract is boolean")
    );
    let input_ids = case["input"]["uniqueBatchItemIds"]
        .as_array()
        .expect("fixture batch IDs are an array")
        .iter()
        .map(|id| {
            CString::new(id.as_str().expect("fixture batch ID is hexadecimal"))
                .expect("fixture batch ID has no NUL byte")
        })
        .collect::<Vec<_>>();
    let expected = &case["expectedOutput"];
    let fixture_statuses = expected["resultStatuses"]
        .as_array()
        .expect("fixture result statuses are an array")
        .iter()
        .map(|status| {
            let value = match status.as_str().expect("fixture result status is text") {
                "Success" => "0",
                "Operation Pending" => "2",
                "Operation Failed" => "1",
                other => panic!("unsupported fixture result status {other}"),
            };
            CString::new(value).expect("numeric status has no NUL byte")
        })
        .collect::<Vec<_>>();
    let correlation = CString::new(
        expected["correlationValueHex"]
            .as_str()
            .expect("fixture correlation is hexadecimal"),
    )
    .expect("fixture correlation has no NUL byte");
    let fixture_item_count = CString::new(
        expected["itemIndexes"]
            .as_array()
            .expect("fixture item indexes are an array")
            .len()
            .to_string(),
    )
    .expect("fixture item count has no NUL byte");
    let fixture_pending_count = CString::new(
        expected["pendingCount"]
            .as_u64()
            .expect("fixture pending count is unsigned")
            .to_string(),
    )
    .expect("fixture pending count has no NUL byte");
    let fixture_version_count = CString::new(fixture_versions.len().to_string())
        .expect("fixture version count has no NUL byte");
    let fixture_version_values = fixture_versions
        .iter()
        .flat_map(|(major, minor)| [major.to_string(), minor.to_string()])
        .map(|version| CString::new(version).expect("numeric version has no NUL byte"))
        .collect::<Vec<_>>();
    let expected_exchange_count = expected["transportExchangeCount"]
        .as_u64()
        .expect("fixture transport exchange count is unsigned");
    assert_eq!(
        input_ids.len(),
        3,
        "the C fixture contract covers three items"
    );
    assert_eq!(fixture_statuses.len(), input_ids.len());

    let pki = EphemeralPki::generate().expect("ephemeral test PKI is generated");
    let client_chain = pki.client_identity().certificate_chain_der();
    let paths = TemporaryCredentialFiles::create([
        client_chain[0],
        client_chain[1],
        pki.client_identity().private_key_der(),
        pki.authority_certificate_der(),
    ]);
    let listener =
        TcpListener::bind(("127.0.0.1", 0)).expect("an ephemeral local TCP port is available");
    let port = listener
        .local_addr()
        .expect("listener has a local address")
        .port();
    let port = CString::new(port.to_string()).expect("port has no NUL byte");
    let server_config = mtls_server_config(&pki);
    let exchange_count = Arc::new(AtomicU64::new(0));
    let peer = spawn_public_client_batch_peer(
        listener,
        server_config,
        Arc::clone(&exchange_count),
        fixture_versions.clone(),
        server_result_message.as_bytes().to_vec(),
        create_input,
        CreateKeyPairParityFixture {
            input: create_key_pair_case["input"].clone(),
            expected_output: create_key_pair_expected.clone(),
        },
    );

    let executable = CString::new("kmipkit_public_client_consumer")
        .expect("test executable name has no NUL byte");
    let certificate = CString::new(paths.paths[0].to_string_lossy().as_bytes())
        .expect("temporary path has no NUL byte");
    let authority = CString::new(paths.paths[1].to_string_lossy().as_bytes())
        .expect("temporary path has no NUL byte");
    let private_key = CString::new(paths.paths[2].to_string_lossy().as_bytes())
        .expect("temporary path has no NUL byte");
    let trust_root = CString::new(paths.paths[3].to_string_lossy().as_bytes())
        .expect("temporary path has no NUL byte");
    let mut arguments = vec![
        executable.as_ptr().cast_mut(),
        certificate.as_ptr().cast_mut(),
        authority.as_ptr().cast_mut(),
        private_key.as_ptr().cast_mut(),
        trust_root.as_ptr().cast_mut(),
        port.as_ptr().cast_mut(),
    ];
    arguments.extend(input_ids.iter().map(|id| id.as_ptr().cast_mut()));
    arguments.extend(
        fixture_statuses
            .iter()
            .map(|status| status.as_ptr().cast_mut()),
    );
    arguments.push(correlation.as_ptr().cast_mut());
    arguments.push(fixture_item_count.as_ptr().cast_mut());
    arguments.push(fixture_pending_count.as_ptr().cast_mut());
    arguments.push(fixture_version_count.as_ptr().cast_mut());
    arguments.extend(
        fixture_version_values
            .iter()
            .map(|version| version.as_ptr().cast_mut()),
    );
    arguments.push(expected_result_message.as_ptr().cast_mut());
    let argument_count = i32::try_from(arguments.len()).expect("argument count fits in i32");
    let mut limits = std::ptr::null_mut();
    // SAFETY: `limits` is a writable output slot and the successful handle is
    // retained for the C call, then released by this Rust test harness.
    let status =
        unsafe { kmipkit_codec_limits_create(16 * 1024 * 1024, 64, 400_000, &raw mut limits) };
    assert_eq!(status, 0, "configured codec limits must succeed");
    assert!(
        !limits.is_null(),
        "configured codec limits must return a handle"
    );
    // SAFETY: all argv entries remain live and NUL terminated for the synchronous
    // C call; the consumer is compiled against the generated public header only.
    let result = unsafe {
        kmipkit_client_abi_c_consumer_execute_tls(argument_count, arguments.as_mut_ptr(), limits)
    };
    // SAFETY: the configured limits call returned this live handle and the C consumer only
    // borrowed it while synchronously executing the batch.
    unsafe {
        kmipkit_codec_limits_release(limits);
    }

    let mut last_status = 0;
    let mut last_category = 0;
    let mut last_cause = 0;
    let mut last_delivery = 0;
    // SAFETY: all four arguments are writable scalar outputs for this call.
    let metadata_status = unsafe {
        kmipkit_last_error_get(
            &raw mut last_status,
            &raw mut last_category,
            &raw mut last_cause,
            &raw mut last_delivery,
        )
    };
    assert_eq!(metadata_status, 0);
    match message_expectation {
        CreationResultMessageExpectation::MatchesFixture => assert_eq!(
            result,
            0,
            "generated public C consumer failed (status {last_status}, category {last_category}, cause {last_cause}, delivery {last_delivery}); server exchanges: {}",
            exchange_count.load(Ordering::Relaxed)
        ),
        CreationResultMessageExpectation::RejectsDivergence => assert_ne!(
            result, 0,
            "the compiled C consumer accepted a Result Message that differs from expectedOutput"
        ),
    }
    peer.join().expect("the loopback TLS peer completes");
    assert_eq!(
        exchange_count.load(Ordering::Relaxed),
        expected_exchange_count
    );
}

fn spawn_public_client_batch_peer(
    listener: TcpListener,
    server_config: Arc<ServerConfig>,
    exchange_count: Arc<AtomicU64>,
    fixture_versions: Vec<(i32, i32)>,
    failure_message: Vec<u8>,
    create_input: serde_json::Value,
    create_key_pair: CreateKeyPairParityFixture,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let (stream, _) = listener
            .accept()
            .expect("the public C client connects to the local peer");
        exchange_count.fetch_add(1, Ordering::Relaxed);
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .expect("server read timeout is configured");
        stream
            .set_write_timeout(Some(std::time::Duration::from_secs(5)))
            .expect("server write timeout is configured");
        let connection = ServerConnection::new(server_config)
            .expect("the TLS 1.3 server connection is configured");
        let mut tls = StreamOwned::new(connection, stream);
        let request = read_ttlv_frame(&mut tls).expect("bounded request frame arrives");
        assert!(request.starts_with(&[0x42, 0x00, 0x78]));
        assert!(
            c_request_contains_create_fixture(&request, &create_input),
            "compiled C consumer must transmit the Create request represented by the shared fixture"
        );
        assert!(
            c_request_contains_create_key_pair_fixture(&request, &create_key_pair.input),
            "compiled C consumer must transmit the Create Key Pair request represented by the shared fixture"
        );
        assert!(
            c_request_contains_large_query(&request),
            "C ABI consumer must transmit 100,001 Query Functions and Object Groups over mTLS"
        );
        assert!(
            c_request_contains_exact_derivation_parameters(&request),
            "C ABI consumer must transmit the exact ordered Derive Key parameter values"
        );
        assert!(
            c_request_contains_exact_export_selectors(&request),
            "C ABI consumer must transmit the exact ordered Export selectors"
        );
        assert!(
            c_request_contains_empty_get_constraints(&request),
            "C ABI consumer must transmit an empty Get Constraints payload"
        );
        assert!(
            c_request_contains_get_and_locate(&request),
            "C ABI consumer must transmit the Get identifier and empty Locate attributes"
        );
        assert!(
            c_request_contains_import(&request),
            "C ABI consumer must transmit the complete Import request through the public batch API"
        );
        assert!(
            c_request_contains_log(&request),
            "C ABI consumer must transmit the typed Log request and message through the public batch API"
        );
        assert!(
            c_request_contains_login(&request),
            "C ABI consumer must transmit the typed Login request through the public batch API"
        );
        tls.write_all(&mixed_discover_versions_response(
            &fixture_versions,
            &failure_message,
            &create_key_pair.expected_output,
        ))
        .expect("the typed KMIP response is written");
        tls.flush().expect("the TLS response is flushed");
        let _ = tls.get_ref().shutdown(Shutdown::Both);
    })
}

struct CreateKeyPairRequestShape {
    groups: Vec<(u32, Vec<u32>)>,
}

fn fixture_attribute_tags(input: &serde_json::Value, field: &str) -> Option<Vec<u32>> {
    input[field]["attributeTagsInOrder"]
        .as_array()?
        .iter()
        .map(|tag| u32::from_str_radix(tag.as_str()?.strip_prefix("0x")?, 16).ok())
        .collect()
}

fn create_key_pair_request_shape(input: &serde_json::Value) -> Option<CreateKeyPairRequestShape> {
    const COMMON_ATTRIBUTES: u32 = 0x0042_0126;
    const PRIVATE_KEY_ATTRIBUTES: u32 = 0x0042_0127;
    const PUBLIC_KEY_ATTRIBUTES: u32 = 0x0042_0128;
    const COMMON_MASKS: u32 = 0x0042_0163;
    const PRIVATE_MASKS: u32 = 0x0042_0164;
    const PUBLIC_MASKS: u32 = 0x0042_0165;

    if input["operation"] != "Create Key Pair"
        || input["effectiveTable191ValuesEqual"].as_bool() != Some(true)
    {
        return None;
    }
    let common_tags = fixture_attribute_tags(input, "commonAttributes")?;
    let private_tags = fixture_attribute_tags(input, "privateKeyAttributes")?;
    let public_tags = fixture_attribute_tags(input, "publicKeyAttributes")?;
    let mask_presence = [
        (
            COMMON_MASKS,
            input["protectionStorageMasks"]["commonPresent"].as_bool()?,
        ),
        (
            PRIVATE_MASKS,
            input["protectionStorageMasks"]["privatePresent"].as_bool()?,
        ),
        (
            PUBLIC_MASKS,
            input["protectionStorageMasks"]["publicPresent"].as_bool()?,
        ),
    ];
    if [
        "commonAttributes",
        "privateKeyAttributes",
        "publicKeyAttributes",
    ]
    .into_iter()
    .any(|group| input[group]["present"].as_bool() != Some(true))
    {
        return None;
    }
    let mut groups = vec![
        (COMMON_ATTRIBUTES, common_tags),
        (PRIVATE_KEY_ATTRIBUTES, private_tags),
        (PUBLIC_KEY_ATTRIBUTES, public_tags),
    ];
    groups.extend(
        mask_presence
            .into_iter()
            .filter_map(|(tag, present)| present.then_some((tag, Vec::new()))),
    );
    Some(CreateKeyPairRequestShape { groups })
}

fn c_request_contains_create_key_pair_fixture(encoded: &[u8], input: &serde_json::Value) -> bool {
    const BATCH_ITEM: u32 = 0x0042_000f;
    const CREATE_KEY_PAIR: u32 = 2;

    let Some(shape) = create_key_pair_request_shape(input) else {
        return false;
    };
    let Some(root) = decode_test_request(encoded) else {
        return false;
    };
    root.with_value(|value| {
        let ValueView::Structure(message) = value else {
            return false;
        };
        let mut matching = message.children().iter().filter(|item| {
            item.tag().raw() == BATCH_ITEM && c_batch_item_has_operation(item, CREATE_KEY_PAIR)
        });
        let Some(batch_item) = matching.next() else {
            return false;
        };
        matching.next().is_none()
            && batch_item.with_value(|value| {
                let ValueView::Structure(batch_item) = value else {
                    return false;
                };
                c_create_key_pair_payload_matches(&batch_item, &shape)
            })
    })
}

fn c_batch_item_has_operation(item: &Item, operation: u32) -> bool {
    const OPERATION: u32 = 0x0042_005c;

    item.with_value(|value| {
        let ValueView::Structure(batch_item) = value else {
            return false;
        };
        c_unique_child_index(&batch_item, OPERATION).is_some_and(|index| {
            batch_item.children()[index].with_value(
                |value| matches!(value, ValueView::Enumeration(actual) if *actual == operation),
            )
        })
    })
}

fn c_create_key_pair_payload_matches(
    batch_item: &kmipkit_ttlv::StructureView<'_>,
    shape: &CreateKeyPairRequestShape,
) -> bool {
    const REQUEST_PAYLOAD: u32 = 0x0042_0079;

    c_unique_child_index(batch_item, REQUEST_PAYLOAD).is_some_and(|index| {
        batch_item.children()[index].with_value(|value| {
            let ValueView::Structure(payload) = value else {
                return false;
            };
            payload.children().len() == shape.groups.len()
                && payload
                    .children()
                    .iter()
                    .zip(&shape.groups)
                    .all(|(item, (tag, child_tags))| {
                        c_attribute_group_matches(item, *tag, child_tags)
                    })
        })
    })
}

fn c_attribute_group_matches(item: &Item, expected_tag: u32, child_tags: &[u32]) -> bool {
    item.tag().raw() == expected_tag
        && item.with_value(|value| {
            let ValueView::Structure(group) = value else {
                return false;
            };
            group.children().len() == child_tags.len()
                && group
                    .children()
                    .iter()
                    .zip(child_tags)
                    .all(|(child, expected_child_tag)| {
                        child.tag().raw() == *expected_child_tag
                            && child.with_value(|value| {
                                matches!(value, ValueView::Enumeration(raw) if *raw == 0xdead_beef)
                            })
                    })
        })
}

fn c_request_contains_create_fixture(encoded: &[u8], input: &serde_json::Value) -> bool {
    const OBJECT_TYPE_TAG: [u8; 8] = [0x42, 0x00, 0x57, 0x05, 0, 0, 0, 4];
    const EMPTY_ATTRIBUTES: [u8; 8] = [0x42, 0x01, 0x25, 0x01, 0, 0, 0, 0];
    const EMPTY_PROTECTION_STORAGE_MASKS: [u8; 8] = [0x42, 0x01, 0x5f, 0x01, 0, 0, 0, 0];

    if input["operation"] != "Create"
        || input["attributesPresent"].as_bool() != Some(true)
        || input["attributeItems"].as_array().map(Vec::len) != Some(0)
        || input["protectionStorageMasks"]["present"].as_bool() != Some(true)
        || input["protectionStorageMasks"]["childTags"]
            .as_array()
            .map(Vec::len)
            != Some(0)
    {
        return false;
    }
    let Some(object_type) = input["objectType"]
        .as_u64()
        .and_then(|value| u32::try_from(value).ok())
    else {
        return false;
    };
    let mut object_type_ttlv = OBJECT_TYPE_TAG.to_vec();
    object_type_ttlv.extend_from_slice(&object_type.to_be_bytes());

    encoded
        .windows(object_type_ttlv.len())
        .any(|window| window == object_type_ttlv)
        && encoded
            .windows(EMPTY_ATTRIBUTES.len())
            .any(|window| window == EMPTY_ATTRIBUTES)
        && encoded
            .windows(EMPTY_PROTECTION_STORAGE_MASKS.len())
            .any(|window| window == EMPTY_PROTECTION_STORAGE_MASKS)
}

fn c_request_contains_large_query(encoded: &[u8]) -> bool {
    const BATCH_ITEM: u32 = 0x0042_000f;
    const OPERATION: u32 = 0x0042_005c;
    const QUERY: u32 = 24;
    const REQUEST_PAYLOAD: u32 = 0x0042_0079;
    const QUERY_FUNCTION: u32 = 0x0042_0074;
    const OBJECT_GROUPS: u32 = 0x0042_0166;
    const OBJECT_GROUP: u32 = 0x0042_0056;
    const EXPECTED_COUNT: usize = 100_001;

    let Ok(limits) = kmipkit_ttlv::codec::CodecLimits::new(8_388_608, 64, 400_000) else {
        return false;
    };
    let Ok(root) = kmipkit_ttlv::codec::decode_with_limits(encoded, &limits) else {
        return false;
    };
    root.with_value(|value| {
        let ValueView::Structure(message) = value else {
            return false;
        };
        message.children().iter().any(|batch_item| {
            if batch_item.tag().raw() != BATCH_ITEM {
                return false;
            }
            batch_item.with_value(|value| {
                let ValueView::Structure(batch_item) = value else {
                    return false;
                };
                let children = batch_item.children();
                let is_query = children.iter().any(|item| {
                    item.tag().raw() == OPERATION
                        && item.with_value(
                            |value| matches!(value, ValueView::Enumeration(value) if *value == QUERY),
                        )
                });
                let payload = children.iter().find(|item| item.tag().raw() == REQUEST_PAYLOAD);
                is_query
                    && payload.is_some_and(|payload| {
                        payload.with_value(|value| {
                            let ValueView::Structure(payload) = value else {
                                return false;
                            };
                            let functions = payload
                                .children()
                                .iter()
                                .filter(|item| item.tag().raw() == QUERY_FUNCTION)
                                .count();
                            let groups = payload
                                .children()
                                .iter()
                                .find(|item| item.tag().raw() == OBJECT_GROUPS);
                            functions == EXPECTED_COUNT
                                && groups.is_some_and(|groups| {
                                    groups.with_value(|value| {
                                        let ValueView::Structure(groups) = value else {
                                            return false;
                                        };
                                        groups
                                            .children()
                                            .iter()
                                            .filter(|item| item.tag().raw() == OBJECT_GROUP)
                                            .count()
                                            == EXPECTED_COUNT
                                    })
                                })
                        })
                    })
            })
        })
    })
}

const C_REKEY_BATCH_ITEM_TAG: u32 = 0x0042_000F;
const C_REKEY_OPERATION_TAG: u32 = 0x0042_005C;
const C_REKEY_OPERATION_VALUE: u32 = 0x1D;
const C_REKEY_BATCH_ID_TAG: u32 = 0x0042_0093;
const C_REKEY_REQUEST_PAYLOAD_TAG: u32 = 0x0042_0079;
const C_REKEY_PRIVATE_IDENTIFIER_TAG: u32 = 0x0042_0066;
const C_REKEY_OFFSET_TAG: u32 = 0x0042_0058;
const C_REKEY_FIELD_TAGS: [u32; 6] = [
    0x0042_0126,
    0x0042_0127,
    0x0042_0128,
    0x0042_0163,
    0x0042_0164,
    0x0042_0165,
];
const C_REKEY_MARKERS: [(&[u8], u32); 6] = [
    (b"common-attributes", 0x0054_1101),
    (b"private-attributes", 0x0054_1102),
    (b"public-attributes", 0x0054_1103),
    (b"common-masks", 0x0054_1104),
    (b"private-masks", 0x0054_1105),
    (b"public-masks", 0x0054_1106),
];

fn c_request_rekey_key_pair_issue(encoded: &[u8]) -> &'static str {
    let Some(root) = decode_test_request(encoded) else {
        return "encoded request is not valid TTLV";
    };
    root.with_value(|value| {
        let ValueView::Structure(message) = value else {
            return "request root is not a Structure";
        };
        let mut matching = message
            .children()
            .iter()
            .filter(|item| item.tag().raw() == C_REKEY_BATCH_ITEM_TAG)
            .filter(|item| c_is_rekey_key_pair_batch_item(item));
        let Some(item) = matching.next() else {
            return "request has no Re-key Key Pair item";
        };
        if matching.next().is_some() {
            return "request has multiple Re-key Key Pair items";
        }
        c_rekey_key_pair_batch_item_issue(item)
    })
}

fn c_is_rekey_key_pair_batch_item(item: &Item) -> bool {
    item.with_value(|value| {
        let ValueView::Structure(batch_item) = value else {
            return false;
        };
        batch_item.children().iter().any(|field| {
            field.tag().raw() == C_REKEY_OPERATION_TAG
                && field.with_value(|value| {
                    matches!(value, ValueView::Enumeration(actual) if *actual == C_REKEY_OPERATION_VALUE)
                })
        })
    })
}

fn c_rekey_key_pair_batch_item_issue(item: &Item) -> &'static str {
    item.with_value(|value| {
        let ValueView::Structure(batch_item) = value else {
            return "Re-key Key Pair batch item is not a Structure";
        };
        let mut operation_matches = false;
        let mut id_matches = false;
        let mut payload = None;
        for field in batch_item.children() {
            match field.tag().raw() {
                C_REKEY_OPERATION_TAG => {
                    operation_matches = field.with_value(|value| {
                        matches!(value, ValueView::Enumeration(actual) if *actual == C_REKEY_OPERATION_VALUE)
                    });
                }
                C_REKEY_BATCH_ID_TAG => {
                    id_matches = field.with_value(|value| {
                        matches!(value, ValueView::ByteString(actual) if actual == b"pair-c")
                    });
                }
                C_REKEY_REQUEST_PAYLOAD_TAG if payload.is_some() => {
                    return "request has duplicate Request Payload fields";
                }
                C_REKEY_REQUEST_PAYLOAD_TAG => payload = Some(field),
                _ => {}
            }
        }
        if !operation_matches {
            return "Operation is not Re-key Key Pair";
        }
        if !id_matches {
            return "Unique Batch Item ID does not match the C API input";
        }
        let Some(payload) = payload else {
            return "Request Payload is missing";
        };
        c_rekey_key_pair_payload_issue(payload)
    })
}

fn c_rekey_key_pair_payload_issue(payload: &Item) -> &'static str {
    payload.with_value(|value| {
        let ValueView::Structure(payload) = value else {
            return "Request Payload is not a Structure";
        };
        let children = payload.children();
        if children.len() != 8 {
            return "Request Payload does not contain identifier, offset, and six fields";
        }
        if children[0].tag().raw() != C_REKEY_PRIVATE_IDENTIFIER_TAG {
            return "Private Key Unique Identifier tag is incorrect";
        }
        if !children[0].with_value(|value| {
            matches!(value, ValueView::TextString(actual) if actual == "existing-private-key")
        }) {
            return "Private Key Unique Identifier type or value is incorrect";
        }
        if children[1].tag().raw() != C_REKEY_OFFSET_TAG
            || !children[1].with_value(
                |value| matches!(value, ValueView::Interval(actual) if *actual == 86_400),
            )
        {
            return "Offset does not match the C API input";
        }
        for (index, (marker, marker_tag)) in C_REKEY_MARKERS.iter().enumerate() {
            let field = &children[index + 2];
            if field.tag().raw() != C_REKEY_FIELD_TAGS[index] {
                return "optional fields are not in KMIP Table 310 order";
            }
            if !c_rekey_key_pair_group_matches(field, *marker_tag, marker) {
                return "optional field structure does not match its C API sentinel";
            }
        }
        ""
    })
}

fn c_rekey_key_pair_group_matches(group_item: &Item, marker_tag: u32, marker: &[u8]) -> bool {
    group_item.with_value(|value| {
        let ValueView::Structure(group) = value else {
            return false;
        };
        group.children().len() == 1
            && group.children()[0].tag().raw() == marker_tag
            && group.children()[0].with_value(
                |value| matches!(value, ValueView::ByteString(actual) if actual == marker),
            )
    })
}

fn decode_test_request(encoded: &[u8]) -> Option<Item> {
    let limits = kmipkit_ttlv::codec::CodecLimits::new(8_388_608, 64, 400_000).ok()?;
    kmipkit_ttlv::codec::decode_with_limits(encoded, &limits).ok()
}

fn c_request_matches_pkcs11_fixture(encoded: &[u8]) -> bool {
    const BATCH_ITEM: u32 = 0x0042_000f;
    const OPERATION: u32 = 0x0042_005c;
    const PKCS11: u32 = 0x33;
    const REQUEST_PAYLOAD: u32 = 0x0042_0079;
    const EXPECTED_PAYLOAD_TAGS: [u32; 4] = [0x0042_0159, 0x0042_015a, 0x0042_00d6, 0x0042_015b];

    let Some(root) = decode_test_request(encoded) else {
        return false;
    };
    root.with_value(|value| {
        let ValueView::Structure(message) = value else {
            return false;
        };
        let batch_items = message
            .children()
            .iter()
            .filter(|item| item.tag().raw() == BATCH_ITEM)
            .collect::<Vec<_>>();
        if batch_items.len() != 1 {
            return false;
        }
        batch_items[0].with_value(|value| {
            let ValueView::Structure(batch_item) = value else {
                return false;
            };
            let children = batch_item.children();
            let operation = children.iter().find(|item| item.tag().raw() == OPERATION);
            let payload = children
                .iter()
                .find(|item| item.tag().raw() == REQUEST_PAYLOAD);
            let has_pkcs11_operation = operation.is_some_and(|item| {
                item.with_value(
                    |value| matches!(value, ValueView::Enumeration(actual) if *actual == PKCS11),
                )
            });
            let has_ordered_payload = payload.is_some_and(|item| {
                item.with_value(|value| {
                    let ValueView::Structure(payload) = value else {
                        return false;
                    };
                    let fields = payload.children();
                    if fields.len() != EXPECTED_PAYLOAD_TAGS.len()
                        || fields
                            .iter()
                            .map(|field| field.tag().raw())
                            .ne(EXPECTED_PAYLOAD_TAGS)
                    {
                        return false;
                    }
                    let interface_matches = fields[0].with_value(|value| {
                        let ValueView::Structure(interface) = value else {
                            return false;
                        };
                        interface.children().len() == 1
                            && interface.children()[0].tag().raw() == 0x0042_0159
                            && interface.children()[0].with_value(|value| {
                                matches!(value, ValueView::TextString(actual) if actual == "test-interface")
                            })
                    });
                    let function_matches = fields[1].with_value(|value| {
                        let ValueView::Structure(function) = value else {
                            return false;
                        };
                        function.children().len() == 1
                            && function.children()[0].tag().raw() == 0x0042_015a
                            && function.children()[0].with_value(|value| {
                                matches!(value, ValueView::Integer(actual) if *actual == -12345)
                            })
                    });
                    let correlation_matches = fields[2].with_value(|value| {
                        matches!(value, ValueView::ByteString(actual) if actual == b"pkcs11-correlation")
                    });
                    let input_matches = fields[3].with_value(|value| {
                        let ValueView::Structure(input) = value else {
                            return false;
                        };
                        input.children().len() == 1
                            && input.children()[0].tag().raw() == 0x0042_015b
                            && input.children()[0].with_value(|value| {
                                matches!(value, ValueView::ByteString(actual) if actual == b"opaque-input")
                            })
                    });
                    interface_matches && function_matches && correlation_matches && input_matches
                })
            });
            has_pkcs11_operation && has_ordered_payload
        })
    })
}

fn fixture_pkcs11_correlation_value(expected: &serde_json::Value, case: &str) -> (bool, String) {
    let correlation = if case == "nonempty" {
        &expected["typedResponse"]["correlationValue"]
    } else {
        &expected["correlationValuePresenceCases"][case]
    };
    let present = correlation["present"]
        .as_bool()
        .expect("fixture correlation presence is Boolean");
    let hexadecimal = correlation["hex"].as_str().unwrap_or("").to_owned();
    (present, hexadecimal)
}

fn pkcs11_success_response_message(fixture: &serde_json::Value, correlation_case: &str) -> Vec<u8> {
    let expected = fixture;
    let typed = &expected["typedResponse"];
    let function = fixture_pkcs11_field_structure(&typed["function"]);
    let output_parameters = fixture_pkcs11_field_structure(&typed["outputParameters"]);
    let return_code = fixture_pkcs11_field_structure(&typed["returnCode"]);
    let function_tag = fixture_hex_u32(&typed["function"]["tag"]);
    let output_parameters_tag = fixture_hex_u32(&typed["outputParameters"]["tag"]);
    let return_code_tag = fixture_hex_u32(&typed["returnCode"]["tag"]);
    let correlation = &typed["correlationValue"];
    let correlation_tag = fixture_hex_u32(&correlation["tag"]);
    let (correlation_present, correlation_hex) =
        fixture_pkcs11_correlation_value(expected, correlation_case);
    let correlation_bytes = decode_fixture_hex(&correlation_hex);
    let vendor_tag = u32::from_str_radix(
        expected["vendorExtension"]["tag"]
            .as_str()
            .expect("shared PKCS#11 fixture vendor tag is text")
            .strip_prefix("0x")
            .expect("shared PKCS#11 fixture vendor tag is hexadecimal"),
        16,
    )
    .expect("shared PKCS#11 fixture vendor tag is valid");
    let vendor_value = expected["vendorExtension"]["integer"]
        .as_i64()
        .and_then(|value| i32::try_from(value).ok())
        .expect("shared PKCS#11 fixture vendor integer fits i32");
    let version = structure(
        0x0042_0069,
        [integer(0x0042_006a, 2), integer(0x0042_006b, 1)],
    );
    let header = structure(
        0x0042_007a,
        [version, date_time(0x0042_0092, 1), integer(0x0042_000d, 1)],
    );
    let mut payload_items = Vec::new();
    for tag in expected["responseTags"]
        .as_array()
        .expect("shared fixture response tags are an array")
    {
        let tag = fixture_hex_u32(tag);
        if tag == correlation_tag && !correlation_present {
            continue;
        }
        payload_items.push(match tag {
            value if value == function_tag => function.clone(),
            value if value == correlation_tag => {
                ttlv_item(correlation_tag, 0x08, &correlation_bytes)
            }
            value if value == output_parameters_tag => output_parameters.clone(),
            value if value == return_code_tag => return_code.clone(),
            value if value == vendor_tag => integer(vendor_tag, vendor_value),
            _ => panic!("shared fixture contains an unmapped PKCS#11 response tag"),
        });
    }
    let payload = structure(0x0042_007c, payload_items);
    let response_item = structure(
        0x0042_000f,
        [
            enumeration(0x0042_005c, 0x33),
            enumeration(0x0042_007f, 0),
            payload,
        ],
    );
    structure(0x0042_007b, [header, response_item])
}

fn fixture_hex_u32(value: &serde_json::Value) -> u32 {
    u32::from_str_radix(
        value
            .as_str()
            .expect("shared fixture hexadecimal values are text")
            .strip_prefix("0x")
            .expect("shared fixture hexadecimal values have a prefix"),
        16,
    )
    .expect("shared fixture hexadecimal value fits in u32")
}

fn fixture_pkcs11_field_structure(field: &serde_json::Value) -> Vec<u8> {
    let tag = fixture_hex_u32(&field["tag"]);
    let child_tag = fixture_hex_u32(&field["childTag"]);
    let child = match field["childType"].as_str() {
        Some("Integer") => integer(
            child_tag,
            i32::try_from(field["childValue"].as_i64().expect("Integer fixture value"))
                .expect("Integer fixture value fits i32"),
        ),
        Some("Enumeration") => enumeration(child_tag, fixture_hex_u32(&field["childValue"])),
        Some("ByteString") => ttlv_item(
            child_tag,
            0x08,
            &decode_fixture_hex(
                field["childHex"]
                    .as_str()
                    .expect("Byte String fixture value is hexadecimal"),
            ),
        ),
        _ => panic!("shared fixture has an unsupported PKCS#11 child type"),
    };
    structure(tag, [child])
}

fn c_request_contains_exact_derivation_parameters(encoded: &[u8]) -> bool {
    const BATCH_ITEM: u32 = 0x0042_000F;
    const OPERATION: u32 = 0x0042_005C;
    const DERIVE_KEY: u32 = 5;
    const REQUEST_PAYLOAD: u32 = 0x0042_0079;
    const DERIVATION_PARAMETERS: u32 = 0x0042_0032;
    const DERIVATION_DATA: u32 = 0x0042_0030;
    const SALT: u32 = 0x0042_0084;
    const ITERATION_COUNT: u32 = 0x0042_003C;
    const FIRST_DATA: &[u8] = &[0xA1, 0x00, 0xB2];
    const SECOND_DATA: &[u8] = &[0xC3, 0x00, 0xD4];
    const SALT_BYTES: &[u8] = &[0xE5, 0x00, 0xF6];

    fn byte_string_is(item: &Item, expected: &[u8]) -> bool {
        item.with_value(
            |value| matches!(value, ValueView::ByteString(actual) if actual == expected),
        )
    }

    fn unique_child_index(structure: &kmipkit_ttlv::StructureView<'_>, tag: u32) -> Option<usize> {
        let mut matching = structure
            .children()
            .iter()
            .enumerate()
            .filter(|(_, item)| item.tag().raw() == tag);
        let (index, _) = matching.next()?;
        matching.next().is_none().then_some(index)
    }

    fn has_operation(item: &Item, expected: u32) -> bool {
        item.with_value(|value| {
            let ValueView::Structure(structure) = value else {
                return false;
            };
            unique_child_index(&structure, OPERATION).is_some_and(|index| {
                structure.children()[index].with_value(
                    |value| matches!(value, ValueView::Enumeration(actual) if *actual == expected),
                )
            })
        })
    }

    fn has_exact_parameters(item: &Item) -> bool {
        item.with_value(|value| {
            let ValueView::Structure(structure) = value else {
                return false;
            };
            let children = structure.children();
            children.len() == 4
                && children[0].tag().raw() == DERIVATION_DATA
                && byte_string_is(&children[0], FIRST_DATA)
                && children[1].tag().raw() == DERIVATION_DATA
                && byte_string_is(&children[1], SECOND_DATA)
                && children[2].tag().raw() == SALT
                && byte_string_is(&children[2], SALT_BYTES)
                && children[3].tag().raw() == ITERATION_COUNT
                && children[3].with_value(
                    |value| matches!(value, ValueView::Integer(actual) if *actual == 4096),
                )
        })
    }

    fn derive_key_item_has_expected_payload(item: &Item) -> bool {
        item.with_value(|value| {
            let ValueView::Structure(batch_item) = value else {
                return false;
            };
            unique_child_index(&batch_item, REQUEST_PAYLOAD).is_some_and(|index| {
                batch_item.children()[index].with_value(|value| {
                    let ValueView::Structure(payload) = value else {
                        return false;
                    };
                    unique_child_index(&payload, DERIVATION_PARAMETERS)
                        .is_some_and(|index| has_exact_parameters(&payload.children()[index]))
                })
            })
        })
    }

    let Some(root) = decode_test_request(encoded) else {
        return false;
    };
    root.with_value(|value| {
        let ValueView::Structure(message) = value else {
            return false;
        };
        let mut derive_key_items = message
            .children()
            .iter()
            .filter(|item| item.tag().raw() == BATCH_ITEM)
            .filter(|item| has_operation(item, DERIVE_KEY));
        let Some(derive_key_item) = derive_key_items.next() else {
            return false;
        };
        derive_key_items.next().is_none() && derive_key_item_has_expected_payload(derive_key_item)
    })
}

fn c_request_contains_exact_export_selectors(encoded: &[u8]) -> bool {
    const BATCH_ITEM: u32 = 0x0042_000F;
    const OPERATION: u32 = 0x0042_005C;
    const EXPORT: u32 = 43;
    const REQUEST_PAYLOAD: u32 = 0x0042_0079;
    const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
    const KEY_FORMAT_TYPE: u32 = 0x0042_0042;
    const KEY_WRAP_TYPE: u32 = 0x0042_00F8;
    const KEY_COMPRESSION_TYPE: u32 = 0x0042_0041;
    const KEY_WRAPPING_SPECIFICATION: u32 = 0x0042_0047;

    fn unique_child_index(structure: &kmipkit_ttlv::StructureView<'_>, tag: u32) -> Option<usize> {
        let mut matching = structure
            .children()
            .iter()
            .enumerate()
            .filter(|(_, item)| item.tag().raw() == tag);
        let (index, _) = matching.next()?;
        matching.next().is_none().then_some(index)
    }

    fn has_operation(item: &Item) -> bool {
        item.with_value(|value| {
            let ValueView::Structure(structure) = value else {
                return false;
            };
            unique_child_index(&structure, OPERATION).is_some_and(|index| {
                structure.children()[index].with_value(
                    |value| matches!(value, ValueView::Enumeration(actual) if *actual == EXPORT),
                )
            })
        })
    }

    fn has_exact_selectors(item: &Item) -> bool {
        item.with_value(|value| {
            let ValueView::Structure(batch_item) = value else {
                return false;
            };
            let Some(payload_index) = unique_child_index(&batch_item, REQUEST_PAYLOAD) else {
                return false;
            };
            batch_item.children()[payload_index].with_value(|value| {
                let ValueView::Structure(payload) = value else {
                    return false;
                };
                let children = payload.children();
                children.len() == 5
                    && children[0].tag().raw() == UNIQUE_IDENTIFIER
                    && children[0].with_value(|value| {
                        matches!(value, ValueView::TextString(actual) if actual == "export-key")
                    })
                    && children[1].tag().raw() == KEY_FORMAT_TYPE
                    && children[1].with_value(|value| {
                        matches!(value, ValueView::Enumeration(actual) if *actual == 0xfedc_ba98)
                    })
                    && children[2].tag().raw() == KEY_WRAP_TYPE
                    && children[2].with_value(|value| {
                        matches!(value, ValueView::Enumeration(actual) if *actual == 0xfedc_ba99)
                    })
                    && children[3].tag().raw() == KEY_COMPRESSION_TYPE
                    && children[3].with_value(|value| {
                        matches!(value, ValueView::Enumeration(actual) if *actual == 0xfedc_ba9a)
                    })
                    && children[4].tag().raw() == KEY_WRAPPING_SPECIFICATION
                    && children[4].with_value(|value| {
                        matches!(value, ValueView::Structure(actual) if actual.children().is_empty())
                    })
            })
        })
    }

    let Some(root) = decode_test_request(encoded) else {
        return false;
    };
    root.with_value(|value| {
        let ValueView::Structure(message) = value else {
            return false;
        };
        let mut export_items = message
            .children()
            .iter()
            .filter(|item| item.tag().raw() == BATCH_ITEM)
            .filter(|item| has_operation(item));
        let Some(export_item) = export_items.next() else {
            return false;
        };
        export_items.next().is_none() && has_exact_selectors(export_item)
    })
}

fn c_request_contains_empty_get_constraints(encoded: &[u8]) -> bool {
    const BATCH_ITEM: u32 = 0x0042_000F;
    const OPERATION: u32 = 0x0042_005C;
    const GET_CONSTRAINTS: u32 = 56;
    const REQUEST_PAYLOAD: u32 = 0x0042_0079;

    fn unique_child_index(structure: &kmipkit_ttlv::StructureView<'_>, tag: u32) -> Option<usize> {
        let mut matching = structure
            .children()
            .iter()
            .enumerate()
            .filter(|(_, item)| item.tag().raw() == tag);
        let (index, _) = matching.next()?;
        matching.next().is_none().then_some(index)
    }

    fn has_empty_request_payload(item: &Item) -> bool {
        item.with_value(|value| {
            let ValueView::Structure(batch_item) = value else {
                return false;
            };
            let Some(payload_index) = unique_child_index(&batch_item, REQUEST_PAYLOAD) else {
                return false;
            };
            batch_item.children()[payload_index].with_value(|value| {
                matches!(value, ValueView::Structure(payload) if payload.children().is_empty())
            })
        })
    }

    fn has_operation(item: &Item) -> bool {
        item.with_value(|value| {
            let ValueView::Structure(batch_item) = value else {
                return false;
            };
            unique_child_index(&batch_item, OPERATION).is_some_and(|index| {
                batch_item.children()[index].with_value(
                    |value| matches!(value, ValueView::Enumeration(actual) if *actual == GET_CONSTRAINTS),
                )
            })
        })
    }

    let Some(root) = decode_test_request(encoded) else {
        return false;
    };
    root.with_value(|value| {
        let ValueView::Structure(message) = value else {
            return false;
        };
        let mut get_constraints_items = message
            .children()
            .iter()
            .filter(|item| item.tag().raw() == BATCH_ITEM)
            .filter(|item| has_operation(item));
        let Some(request_item) = get_constraints_items.next() else {
            return false;
        };
        get_constraints_items.next().is_none() && has_empty_request_payload(request_item)
    })
}

fn c_request_contains_get_and_locate(encoded: &[u8]) -> bool {
    const BATCH_ITEM: u32 = 0x0042_000F;
    let Some(root) = decode_test_request(encoded) else {
        return false;
    };
    root.with_value(|value| {
        let ValueView::Structure(message) = value else {
            return false;
        };
        let batch_items = message
            .children()
            .iter()
            .filter(|item| item.tag().raw() == BATCH_ITEM)
            .collect::<Vec<_>>();
        batch_items
            .iter()
            .filter(|item| c_request_has_get_source(item))
            .count()
            == 1
            && batch_items
                .iter()
                .filter(|item| c_request_has_empty_locate_attributes(item))
                .count()
                == 2
    })
}

fn c_request_contains_import(encoded: &[u8]) -> bool {
    const BATCH_ITEM: u32 = 0x0042_000F;
    let Some(root) = decode_test_request(encoded) else {
        return false;
    };
    root.with_value(|value| {
        let ValueView::Structure(message) = value else {
            return false;
        };
        let mut import_items = message
            .children()
            .iter()
            .filter(|item| item.tag().raw() == BATCH_ITEM)
            .filter(|item| c_import_batch_item_matches(item));
        if import_items.next().is_none() {
            return false;
        }
        import_items.next().is_none()
    })
}

fn c_request_contains_log(encoded: &[u8]) -> bool {
    const BATCH_ITEM: u32 = 0x0042_000F;
    const OPERATION: u32 = 0x0042_005C;
    const LOG: u32 = 44;
    const UNIQUE_BATCH_ITEM_ID: u32 = 0x0042_0093;
    const REQUEST_PAYLOAD: u32 = 0x0042_0079;
    const LOG_MESSAGE: u32 = 0x0042_0141;

    let Some(root) = decode_test_request(encoded) else {
        return false;
    };
    root.with_value(|value| {
        let ValueView::Structure(message) = value else {
            return false;
        };
        let matching = message
            .children()
            .iter()
            .filter(|item| item.tag().raw() == BATCH_ITEM)
            .filter(|item| {
                item.with_value(|value| {
                    let ValueView::Structure(batch_item) = value else {
                        return false;
                    };
                    let mut operation = None;
                    let mut identifier = None;
                    let mut payload = None;
                    for child in batch_item.children() {
                        match child.tag().raw() {
                            OPERATION if operation.is_none() => operation = Some(child),
                            UNIQUE_BATCH_ITEM_ID if identifier.is_none() => identifier = Some(child),
                            REQUEST_PAYLOAD if payload.is_none() => payload = Some(child),
                            OPERATION | UNIQUE_BATCH_ITEM_ID | REQUEST_PAYLOAD => return false,
                            _ => {}
                        }
                    }
                    let Some(operation) = operation else {
                        return false;
                    };
                    let Some(identifier) = identifier else {
                        return false;
                    };
                    let Some(payload) = payload else {
                        return false;
                    };
                    let correct_operation = operation.with_value(
                        |value| matches!(value, ValueView::Enumeration(actual) if *actual == LOG),
                    );
                    let correct_id = identifier.with_value(
                        |value| matches!(value, ValueView::ByteString(actual) if actual == b"log-00"),
                    );
                    let correct_payload = payload.with_value(|value| {
                        let ValueView::Structure(payload) = value else {
                            return false;
                        };
                        payload.children().len() == 1
                            && payload.children()[0].tag().raw() == LOG_MESSAGE
                            && payload.children()[0].with_value(|value| {
                                matches!(value, ValueView::TextString(actual) if actual == "c-log-message")
                            })
                    });
                    correct_operation && correct_id && correct_payload
                })
            })
            .count();
        matching == 1
    })
}

fn c_request_contains_login(encoded: &[u8]) -> bool {
    const BATCH_ITEM: u32 = 0x0042_000F;
    const OPERATION: u32 = 0x0042_005C;
    const LOGIN: u32 = 45;
    const UNIQUE_BATCH_ITEM_ID: u32 = 0x0042_0093;

    let Some(root) = decode_test_request(encoded) else {
        return false;
    };
    root.with_value(|value| {
        let ValueView::Structure(message) = value else {
            return false;
        };
        message.children().iter().any(|item| {
            if item.tag().raw() != BATCH_ITEM {
                return false;
            }
            item.with_value(|value| {
                let ValueView::Structure(batch_item) = value else {
                    return false;
                };
                let mut login = false;
                let mut identifier = false;
                for child in batch_item.children() {
                    match child.tag().raw() {
                        OPERATION => {
                            login |= child.with_value(|value| {
                                matches!(value, ValueView::Enumeration(actual) if *actual == LOGIN)
                            });
                        }
                        UNIQUE_BATCH_ITEM_ID => {
                            identifier |= child.with_value(|value| {
                                matches!(value, ValueView::ByteString(actual) if actual == b"login0")
                            });
                        }
                        _ => {}
                    }
                }
                login && identifier
            })
        })
    })
}

fn c_import_batch_item_matches(item: &Item) -> bool {
    const OPERATION: u32 = 0x0042_005C;
    const IMPORT: u32 = 42;
    const REQUEST_PAYLOAD: u32 = 0x0042_0079;
    const UNIQUE_BATCH_ITEM_ID: u32 = 0x0042_0093;

    item.with_value(|value| {
        let ValueView::Structure(batch_item) = value else {
            return false;
        };
        let Some(operation_index) = c_unique_child_index(&batch_item, OPERATION) else {
            return false;
        };
        let Some(payload_index) = c_unique_child_index(&batch_item, REQUEST_PAYLOAD) else {
            return false;
        };
        let Some(id_index) = c_unique_child_index(&batch_item, UNIQUE_BATCH_ITEM_ID) else {
            return false;
        };
        batch_item.children()[operation_index].with_value(
            |value| matches!(value, ValueView::Enumeration(actual) if *actual == IMPORT),
        ) && c_import_payload_matches(&batch_item.children()[payload_index])
            && batch_item.children()[id_index].with_value(
                |value| matches!(value, ValueView::ByteString(actual) if actual == b"import"),
            )
    })
}

fn c_import_payload_matches(item: &Item) -> bool {
    const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
    const OBJECT_TYPE: u32 = 0x0042_0057;
    const REPLACE_EXISTING: u32 = 0x0042_0124;
    const ATTRIBUTES: u32 = 0x0042_0125;
    const SECRET_DATA: u32 = 0x0042_0085;

    item.with_value(|value| {
        let ValueView::Structure(payload) = value else {
            return false;
        };
        let fields = payload.children();
        fields.len() == 5
            && fields[0].tag().raw() == UNIQUE_IDENTIFIER
            && fields[0].with_value(|value| {
                matches!(value, ValueView::TextString(actual) if actual == "import-key")
            })
            && fields[1].tag().raw() == OBJECT_TYPE
            && fields[1].with_value(|value| {
                matches!(value, ValueView::Enumeration(actual) if *actual == 7)
            })
            && fields[2].tag().raw() == REPLACE_EXISTING
            && fields[2].with_value(|value| matches!(value, ValueView::Boolean(false)))
            && fields[3].tag().raw() == ATTRIBUTES
            && fields[3].with_value(|value| {
                matches!(value, ValueView::Structure(attributes) if attributes.children().is_empty())
            })
            && fields[4].tag().raw() == SECRET_DATA
            && c_import_object_is_preserved(&fields[4])
    })
}

fn c_import_object_is_preserved(item: &Item) -> bool {
    const VENDOR_FIELD: u32 = 0x0054_1234;
    const OBJECT_BYTES: &[u8] = &[0xde, 0xad, 0x00, 0xff];

    item.with_value(|value| {
        let ValueView::Structure(object) = value else {
            return false;
        };
        let Some(index) = c_unique_child_index(&object, VENDOR_FIELD) else {
            return false;
        };
        object.children().len() == 1
            && object.children()[index].with_value(
                |value| matches!(value, ValueView::ByteString(actual) if actual == OBJECT_BYTES),
            )
    })
}

fn c_request_has_get_source(item: &Item) -> bool {
    const OPERATION: u32 = 0x0042_005C;
    const GET: u32 = 10;
    const REQUEST_PAYLOAD: u32 = 0x0042_0079;
    const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
    item.with_value(|value| {
        let ValueView::Structure(batch_item) = value else {
            return false;
        };
        let Some(operation_index) = c_unique_child_index(&batch_item, OPERATION) else {
            return false;
        };
        if !batch_item.children()[operation_index]
            .with_value(|value| matches!(value, ValueView::Enumeration(actual) if *actual == GET))
        {
            return false;
        }
        let Some(payload_index) = c_unique_child_index(&batch_item, REQUEST_PAYLOAD) else {
            return false;
        };
        batch_item.children()[payload_index].with_value(|value| {
            let ValueView::Structure(payload) = value else {
                return false;
            };
            c_unique_child_index(&payload, UNIQUE_IDENTIFIER).is_some_and(|index| {
                payload.children()[index]
                    .with_value(|value| matches!(value, ValueView::TextString("get-source")))
            }) && payload.children().len() == 1
        })
    })
}

fn c_request_has_empty_locate_attributes(item: &Item) -> bool {
    const OPERATION: u32 = 0x0042_005C;
    const LOCATE: u32 = 8;
    const REQUEST_PAYLOAD: u32 = 0x0042_0079;
    const ATTRIBUTES: u32 = 0x0042_0125;
    item.with_value(|value| {
        let ValueView::Structure(batch_item) = value else {
            return false;
        };
        let Some(operation_index) = c_unique_child_index(&batch_item, OPERATION) else {
            return false;
        };
        if !batch_item.children()[operation_index]
            .with_value(|value| matches!(value, ValueView::Enumeration(actual) if *actual == LOCATE))
        {
            return false;
        }
        let Some(payload_index) = c_unique_child_index(&batch_item, REQUEST_PAYLOAD) else {
            return false;
        };
        batch_item.children()[payload_index].with_value(|value| {
            let ValueView::Structure(payload) = value else {
                return false;
            };
            c_unique_child_index(&payload, ATTRIBUTES).is_some_and(|index| {
                payload.children()[index].with_value(|value| {
                    matches!(value, ValueView::Structure(attributes) if attributes.children().is_empty())
                })
            }) && payload.children().len() == 1
        })
    })
}

fn c_unique_child_index(structure: &kmipkit_ttlv::StructureView<'_>, tag: u32) -> Option<usize> {
    let mut matching = structure
        .children()
        .iter()
        .enumerate()
        .filter(|(_, item)| item.tag().raw() == tag);
    let (index, _) = matching.next()?;
    matching.next().is_none().then_some(index)
}

#[test]
fn generated_public_c_consumer_traverses_retained_error_ttlv_over_mtls() {
    let pki = EphemeralPki::generate().expect("ephemeral test PKI is generated");
    let client_chain = pki.client_identity().certificate_chain_der();
    let paths = TemporaryCredentialFiles::create([
        client_chain[0],
        client_chain[1],
        pki.client_identity().private_key_der(),
        pki.authority_certificate_der(),
    ]);
    let listener =
        TcpListener::bind(("127.0.0.1", 0)).expect("an ephemeral local TCP port is available");
    let port = listener
        .local_addr()
        .expect("listener has an address")
        .port();
    let port = CString::new(port.to_string()).expect("port contains no NUL byte");
    let server_config = mtls_server_config(&pki);
    let peer = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("the public C client connects");
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .expect("server read timeout is configured");
        stream
            .set_write_timeout(Some(std::time::Duration::from_secs(5)))
            .expect("server write timeout is configured");
        let connection = ServerConnection::new(server_config)
            .expect("the TLS 1.3 server connection is configured");
        let mut tls = StreamOwned::new(connection, stream);
        let request = read_ttlv_frame(&mut tls).expect("bounded request frame arrives");
        assert!(request.starts_with(&[0x42, 0x00, 0x78]));
        tls.write_all(&mac_response_missing_unique_identifier())
            .expect("the malformed typed response is written");
        tls.flush().expect("the TLS response is flushed");
        let _ = tls.get_ref().shutdown(Shutdown::Both);
    });

    let executable = CString::new("kmipkit_public_client_consumer")
        .expect("test executable name has no NUL byte");
    let certificate = CString::new(paths.paths[0].to_string_lossy().as_bytes())
        .expect("temporary path has no NUL byte");
    let authority = CString::new(paths.paths[1].to_string_lossy().as_bytes())
        .expect("temporary path has no NUL byte");
    let private_key = CString::new(paths.paths[2].to_string_lossy().as_bytes())
        .expect("temporary path has no NUL byte");
    let trust_root = CString::new(paths.paths[3].to_string_lossy().as_bytes())
        .expect("temporary path has no NUL byte");
    let mut arguments = [
        executable.as_ptr().cast_mut(),
        certificate.as_ptr().cast_mut(),
        authority.as_ptr().cast_mut(),
        private_key.as_ptr().cast_mut(),
        trust_root.as_ptr().cast_mut(),
        port.as_ptr().cast_mut(),
    ];
    let argument_count = i32::try_from(arguments.len()).expect("argument count fits in i32");
    let mut limits = std::ptr::null_mut();
    // SAFETY: `limits` is a writable output slot for this call.
    let status =
        unsafe { kmipkit_codec_limits_create(16 * 1024 * 1024, 64, 400_000, &raw mut limits) };
    assert_eq!(status, 0, "configured codec limits must succeed");
    assert!(!limits.is_null(), "configured codec limits return a handle");
    // SAFETY: argv strings and the limits handle remain live for this synchronous C call.
    let result = unsafe {
        kmipkit_client_abi_c_consumer_execute_error_ttlv_tls(
            argument_count,
            arguments.as_mut_ptr(),
            limits,
        )
    };
    // SAFETY: the test created this handle and the C consumer only borrowed it.
    unsafe { kmipkit_codec_limits_release(limits) };

    assert_eq!(
        result, 0,
        "public C consumer must inspect and release retained TTLV"
    );
    peer.join().expect("the loopback TLS peer completes");
}

fn controlled_mtls_peer(
    listener: &TcpListener,
    server_config: Arc<ServerConfig>,
    retry_server_config: Arc<ServerConfig>,
) {
    let (stream, _) = listener
        .accept()
        .expect("the C client connects to the local peer");
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .expect("server read timeout is configured");
    stream
        .set_write_timeout(Some(std::time::Duration::from_secs(5)))
        .expect("server write timeout is configured");
    let connection =
        ServerConnection::new(server_config).expect("the TLS 1.3 server connection is configured");
    let mut tls = StreamOwned::new(connection, stream);
    let request = read_ttlv_frame(&mut tls).expect("bounded request frame arrives");
    assert!(request.starts_with(&[0x42, 0x00, 0x78]));
    tls.write_all(&discover_versions_response())
        .expect("the typed KMIP response is written");
    tls.flush().expect("the TLS response is flushed");
    let _ = tls.get_ref().shutdown(Shutdown::Both);

    let (second_stream, _) = listener
        .accept()
        .expect("the response-shape-error call reaches the controlled peer");
    second_stream
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .expect("post-dispatch peer read timeout is configured");
    let connection = ServerConnection::new(Arc::clone(&retry_server_config))
        .expect("the TLS 1.3 server connection is configured");
    let mut tls = StreamOwned::new(connection, second_stream);
    let request = read_ttlv_frame(&mut tls).expect("response-shape-error request arrives fully");
    assert!(request.starts_with(&[0x42, 0x00, 0x78]));
    tls.write_all(&mismatched_discover_versions_response())
        .expect("mismatched response fixture is written");
    tls.flush()
        .expect("the malformed typed response is flushed");
    let _ = tls.get_ref().shutdown(Shutdown::Both);

    let (third_stream, _) = listener
        .accept()
        .expect("the explicit post-dispatch transport-failure call reaches the peer");
    third_stream
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .expect("transport-failure peer read timeout is configured");
    let connection = ServerConnection::new(retry_server_config)
        .expect("the TLS 1.3 server connection is configured");
    let mut tls = StreamOwned::new(connection, third_stream);
    let request = read_ttlv_frame(&mut tls).expect("post-dispatch request arrives fully");
    assert!(request.starts_with(&[0x42, 0x00, 0x78]));
    let _ = tls.get_ref().shutdown(Shutdown::Both);

    listener
        .set_nonblocking(true)
        .expect("peer can check for an automatic retry");
    let retry_deadline = std::time::Instant::now() + std::time::Duration::from_millis(500);
    loop {
        match listener.accept() {
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                if std::time::Instant::now() >= retry_deadline {
                    break;
                }
                thread::sleep(std::time::Duration::from_millis(10));
            }
            Ok((stream, _)) => {
                let _ = stream.shutdown(Shutdown::Both);
                panic!("the client automatically retried a dispatched KMIP request");
            }
            Err(error) => panic!("peer retry check failed: {error}"),
        }
    }
}

struct TemporaryCredentialFiles {
    directory: PathBuf,
    paths: [PathBuf; 4],
}

impl TemporaryCredentialFiles {
    fn create(contents: [&[u8]; 4]) -> Self {
        let directory = create_private_test_directory("kmipkit-ffi-c-consumer")
            .expect("private test directory is created exclusively");
        let paths = std::array::from_fn(|index| directory.join(format!("credential-{index}.der")));
        let files = Self { directory, paths };
        for (path, bytes) in files.paths.iter().zip(contents) {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
                .expect("ephemeral credential file is created exclusively");
            file.write_all(bytes)
                .expect("ephemeral credential fixture is written");
        }
        files
    }
}

impl Drop for TemporaryCredentialFiles {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

fn create_private_test_directory(prefix: &str) -> io::Result<PathBuf> {
    #[cfg(windows)]
    let parent = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .ok_or_else(|| io::Error::other("LOCALAPPDATA is required for private test files"))?;
    #[cfg(not(windows))]
    let parent = std::env::temp_dir();

    for _ in 0..128 {
        let sequence = TEMP_DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let path = parent.join(format!(
            "{prefix}-{}-{nonce}-{sequence}",
            std::process::id()
        ));
        #[cfg(unix)]
        let builder = {
            use std::os::unix::fs::DirBuilderExt;
            let mut builder = fs::DirBuilder::new();
            builder.mode(0o700);
            builder
        };
        #[cfg(not(unix))]
        let builder = fs::DirBuilder::new();
        match builder.create(&path) {
            Ok(()) => return Ok(path),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "could not create a unique private test directory",
    ))
}

fn mtls_server_config(pki: &EphemeralPki) -> Arc<ServerConfig> {
    let mut roots = RootCertStore::empty();
    roots
        .add(CertificateDer::from(
            pki.authority_certificate_der().to_vec(),
        ))
        .expect("the test authority is a valid root certificate");
    let verifier = WebPkiClientVerifier::builder(Arc::new(roots))
        .build()
        .expect("the mTLS client verifier builds");
    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    let builder = ServerConfig::builder_with_provider(provider)
        .with_protocol_versions(&[&rustls::version::TLS13])
        .expect("the test peer enables TLS 1.3 only");
    let identity = pki.server_identity();
    let certificates = identity
        .certificate_chain_der()
        .into_iter()
        .map(|certificate| CertificateDer::from(certificate.to_vec()))
        .collect();
    let private_key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(
        identity.private_key_der().to_vec(),
    ));
    Arc::new(
        builder
            .with_client_cert_verifier(verifier)
            .with_single_cert(certificates, private_key)
            .expect("the ephemeral server identity is valid"),
    )
}

fn read_ttlv_frame(stream: &mut impl Read) -> io::Result<Vec<u8>> {
    let mut header = [0_u8; 8];
    stream.read_exact(&mut header)?;
    let length = u32::from_be_bytes([header[4], header[5], header[6], header[7]]) as usize;
    if length > MAX_TEST_TTLV_VALUE_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "test peer refused an oversized remote TTLV frame",
        ));
    }
    let padded_length = length
        .checked_add(7)
        .map(|value| value / 8 * 8)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "TTLV padding overflow"))?;
    let frame_length = 8_usize
        .checked_add(padded_length)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "TTLV frame overflow"))?;
    let mut body = vec![0_u8; padded_length];
    stream.read_exact(&mut body)?;
    let mut frame = Vec::with_capacity(frame_length);
    frame.extend_from_slice(&header);
    frame.extend_from_slice(&body);
    Ok(frame)
}

#[test]
fn test_peer_rejects_remote_ttlv_length_before_allocating() {
    let mut stream = std::io::Cursor::new([0_u8, 0, 0, 0, 0xff, 0xff, 0xff, 0xff]);
    let error = read_ttlv_frame(&mut stream).expect_err("oversized frame is rejected");
    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
}

fn discover_versions_response() -> Vec<u8> {
    let protocol_version = structure(
        0x0042_0069,
        [integer(0x0042_006A, 2), integer(0x0042_006B, 1)],
    );
    let header = structure(
        0x0042_007A,
        [
            protocol_version.clone(),
            date_time(0x0042_0092, 1),
            integer(0x0042_000D, 2),
        ],
    );
    let response_payload = structure(0x0042_007C, [protocol_version]);
    let failed_item = structure(
        0x0042_000F,
        [
            enumeration(0x0042_005C, 30),
            ttlv_item(0x0042_0093, 0x08, &[0x00, 0x82, 0xfe, 0x02]),
            enumeration(0x0042_007F, 1),
            enumeration(0x0042_007E, 5),
            ttlv_item(0x0042_007D, 0x07, b"operation unsupported"),
        ],
    );
    let successful_item = structure(
        0x0042_000F,
        [
            enumeration(0x0042_005C, 30),
            ttlv_item(0x0042_0093, 0x08, &[0x00, 0x81, 0xff, 0x01]),
            enumeration(0x0042_007F, 0),
            response_payload,
        ],
    );
    /* The peer returns a reversed batch; item IDs restore caller-request order. */
    structure(0x0042_007B, [header, failed_item, successful_item])
}

fn rekey_key_pair_success_response() -> Vec<u8> {
    let protocol_version = structure(
        0x0042_0069,
        [integer(0x0042_006A, 2), integer(0x0042_006B, 1)],
    );
    let header = structure(
        0x0042_007A,
        [
            protocol_version,
            date_time(0x0042_0092, 1),
            integer(0x0042_000D, 1),
        ],
    );
    let payload = structure(
        0x0042_007C,
        [
            ttlv_item(0x0042_0066, 0x07, b"replacement-private"),
            ttlv_item(0x0042_006F, 0x07, b"replacement-public"),
        ],
    );
    let item = structure(
        0x0042_000F,
        [
            enumeration(0x0042_005C, 0x1D),
            ttlv_item(0x0042_0093, 0x08, b"pair-c"),
            enumeration(0x0042_007F, 0),
            payload,
        ],
    );
    structure(0x0042_007B, [header, item])
}

// This TLS batch returns 28 ordered items, including Login, Encrypt, Decrypt, Ping, and Query results.
fn mixed_discover_versions_response(
    fixture_versions: &[(i32, i32)],
    failure_message: &[u8],
    create_key_pair_expected: &serde_json::Value,
) -> Vec<u8> {
    let protocol_version = structure(
        0x0042_0069,
        [integer(0x0042_006A, 2), integer(0x0042_006B, 1)],
    );
    let header = structure(
        0x0042_007A,
        [
            protocol_version.clone(),
            date_time(0x0042_0092, 1),
            integer(0x0042_000D, 28),
        ],
    );
    let response_versions = fixture_versions
        .iter()
        .map(|(major, minor)| {
            structure(
                0x0042_0069,
                [integer(0x0042_006A, *major), integer(0x0042_006B, *minor)],
            )
        })
        .collect::<Vec<_>>();
    let response_payload = structure(0x0042_007C, response_versions);
    structure(
        0x0042_007B,
        std::iter::once(header)
            .chain(discover_version_response_items(response_payload))
            .chain(creation_response_items(
                failure_message,
                create_key_pair_expected,
            ))
            .chain(cryptographic_response_items())
            .chain([
                derive_key_response_item(),
                export_response_item(),
                get_constraints_response_item(),
                get_response_item(),
                locate_response_item(),
                get_failure_response_item(),
                locate_failure_response_item(),
                import_response_item(),
                log_response_item(),
                login_response_item(),
            ])
            .chain(encrypt_decrypt_response_items())
            .chain([ping_response_item(), query_response_item()]),
    )
}

fn encrypt_decrypt_response_items() -> [Vec<u8>; 2] {
    [
        structure(
            0x0042_000F,
            [
                enumeration(0x0042_005C, 31),
                ttlv_item(0x0042_0093, 0x08, b"enc-00"),
                enumeration(0x0042_007F, 0),
                structure(
                    0x0042_007C,
                    [
                        ttlv_item(0x0042_0094, 0x07, b"c-response-key"),
                        ttlv_item(0x0042_00C2, 0x08, b"ciphertext-c"),
                        ttlv_item(0x0042_003D, 0x08, &[0x71, 0x72]),
                        ttlv_item(0x0042_00D6, 0x08, &[0x81, 0x82]),
                        ttlv_item(0x0042_00FF, 0x08, &[0x91, 0x92]),
                    ],
                ),
            ],
        ),
        structure(
            0x0042_000F,
            [
                enumeration(0x0042_005C, 32),
                ttlv_item(0x0042_0093, 0x08, b"dec-00"),
                enumeration(0x0042_007F, 0),
                structure(
                    0x0042_007C,
                    [
                        ttlv_item(0x0042_0094, 0x07, b"c-response-key"),
                        ttlv_item(0x0042_00C2, 0x08, b"plaintext-c"),
                        ttlv_item(0x0042_00D6, 0x08, &[0x81, 0x82]),
                    ],
                ),
            ],
        ),
    ]
}

fn log_response_item() -> Vec<u8> {
    structure(
        0x0042_000F,
        [
            enumeration(0x0042_005C, 44),
            ttlv_item(0x0042_0093, 0x08, b"log-00"),
            enumeration(0x0042_007F, 0),
            structure(0x0042_007C, []),
        ],
    )
}

fn ping_response_item() -> Vec<u8> {
    structure(
        0x0042_000f,
        [
            enumeration(0x0042_005c, 59),
            ttlv_item(0x0042_0093, 0x08, b"ping00"),
            enumeration(0x0042_007f, 0),
            structure(0x0042_007c, []),
        ],
    )
}

fn query_response_item() -> Vec<u8> {
    let payload = structure(
        0x0042_007c,
        [
            enumeration(0x0042_005c, 59),
            structure(0x0042_015f, []),
            structure(0x0042_0012, [integer(0x0042_0057, 7)]),
        ],
    );
    structure(
        0x0042_000f,
        [
            enumeration(0x0042_005c, 24),
            ttlv_item(0x0042_0093, 0x08, b"query0"),
            enumeration(0x0042_007f, 0),
            payload,
        ],
    )
}

fn login_response_item() -> Vec<u8> {
    let ticket = structure(
        0x0042_0149,
        [
            enumeration(0x0042_014A, 0xFEED_BEEF),
            ttlv_item(0x0042_014B, 0x08, b"login-ticket-native-secret"),
        ],
    );
    structure(
        0x0042_000F,
        [
            enumeration(0x0042_005C, 45),
            ttlv_item(0x0042_0093, 0x08, b"login0"),
            enumeration(0x0042_007F, 0),
            structure(0x0042_007C, [ticket]),
        ],
    )
}

fn import_response_item() -> Vec<u8> {
    structure(
        0x0042_000F,
        [
            enumeration(0x0042_005C, 42),
            ttlv_item(0x0042_0093, 0x08, b"import"),
            enumeration(0x0042_007F, 0),
            structure(
                0x0042_007C,
                [
                    ttlv_item(0x0042_0094, 0x07, b"imported-id"),
                    ttlv_item(0x0054_1234, 0x08, &[0xde, 0xad, 0x00, 0xff]),
                ],
            ),
        ],
    )
}

fn get_response_item() -> Vec<u8> {
    structure(
        0x0042_000F,
        [
            enumeration(0x0042_005C, 10),
            ttlv_item(0x0042_0093, 0x08, b"get-00"),
            enumeration(0x0042_007F, 0),
            structure(
                0x0042_007C,
                [
                    enumeration(0x0042_0057, 0xf123_4567),
                    ttlv_item(0x0042_0094, 0x07, b"get-id"),
                    structure(
                        0x0054_0001,
                        [ttlv_item(0x0054_1234, 0x08, &[0xde, 0xad, 0x00, 0xff])],
                    ),
                ],
            ),
        ],
    )
}

fn locate_response_item() -> Vec<u8> {
    structure(
        0x0042_000F,
        [
            enumeration(0x0042_005C, 8),
            ttlv_item(0x0042_0093, 0x08, b"loc-00"),
            enumeration(0x0042_007F, 0),
            structure(
                0x0042_007C,
                [
                    integer(0x0042_00D5, 3),
                    ttlv_item(0x0042_0094, 0x07, b"first"),
                    ttlv_item(0x0042_0094, 0x07, b"second"),
                    ttlv_item(0x0042_0094, 0x07, b"first"),
                ],
            ),
        ],
    )
}

fn get_failure_response_item() -> Vec<u8> {
    structure(
        0x0042_000F,
        [
            enumeration(0x0042_005C, 10),
            ttlv_item(0x0042_0093, 0x08, b"get-f0"),
            enumeration(0x0042_007F, 1),
            enumeration(0x0042_007E, 5),
        ],
    )
}

fn locate_failure_response_item() -> Vec<u8> {
    structure(
        0x0042_000F,
        [
            enumeration(0x0042_005C, 8),
            ttlv_item(0x0042_0093, 0x08, b"loc-f0"),
            enumeration(0x0042_007F, 1),
            enumeration(0x0042_007E, 5),
        ],
    )
}

fn get_constraints_response_item() -> Vec<u8> {
    structure(
        0x0042_000F,
        [
            enumeration(0x0042_005C, 56),
            ttlv_item(0x0042_0093, 0x08, b"constr"),
            enumeration(0x0042_007F, 0),
            structure(
                0x0042_007C,
                [structure(
                    0x0042_0168,
                    [ttlv_item(0x0054_1234, 0x07, b"constraints-secret-sentinel")],
                )],
            ),
        ],
    )
}

fn export_response_item() -> Vec<u8> {
    structure(
        0x0042_000F,
        [
            enumeration(0x0042_005C, 43),
            ttlv_item(0x0042_0093, 0x08, b"export"),
            enumeration(0x0042_007F, 0),
            structure(
                0x0042_007C,
                [
                    enumeration(0x0042_0057, 0xf123_4567),
                    ttlv_item(0x0042_0094, 0x07, b"exported-key"),
                    structure(0x0042_0125, [enumeration(0x0042_001D, 1)]),
                    structure(
                        0x0042_008F,
                        [structure(
                            0x0042_0040,
                            [
                                enumeration(0x0042_0042, 0xf123_4567),
                                ttlv_item(0x0042_0043, 0x08, &[0xde, 0xad, 0x00, 0xff]),
                            ],
                        )],
                    ),
                ],
            ),
        ],
    )
}

fn derive_key_response_item() -> Vec<u8> {
    structure(
        0x0042_000F,
        [
            enumeration(0x0042_005C, 5),
            ttlv_item(0x0042_0093, 0x08, b"derive"),
            enumeration(0x0042_007F, 0),
            structure(
                0x0042_007C,
                [
                    ttlv_item(0x0042_0094, 0x07, b"derived-key"),
                    ttlv_item(0x0054_0001, 0x08, &[0xde, 0xad, 0x00, 0xff]),
                ],
            ),
        ],
    )
}

fn cryptographic_response_items() -> [Vec<u8>; 6] {
    let correlation = [0x41, 0x00, 0x42];
    let response_identifier = b"response-id";
    [
        structure(
            0x0042_000F,
            [
                enumeration(0x0042_005C, 39),
                ttlv_item(0x0042_0093, 0x08, b"hash-0"),
                enumeration(0x0042_007F, 0),
                structure(
                    0x0042_007C,
                    [
                        ttlv_item(0x0042_00C2, 0x08, b"hash-response-data"),
                        ttlv_item(0x0042_00D6, 0x08, &correlation),
                    ],
                ),
            ],
        ),
        structure(
            0x0042_000F,
            [
                enumeration(0x0042_005C, 35),
                ttlv_item(0x0042_0093, 0x08, b"mac-00"),
                enumeration(0x0042_007F, 0),
                structure(
                    0x0042_007C,
                    [
                        ttlv_item(0x0042_0094, 0x07, response_identifier),
                        ttlv_item(0x0042_00C4, 0x08, b"mac-response-data"),
                        ttlv_item(0x0042_00D6, 0x08, &correlation),
                    ],
                ),
            ],
        ),
        structure(
            0x0042_000F,
            [
                enumeration(0x0042_005C, 36),
                ttlv_item(0x0042_0093, 0x08, b"macvfy"),
                enumeration(0x0042_007F, 0),
                structure(
                    0x0042_007C,
                    [
                        ttlv_item(0x0042_0094, 0x07, response_identifier),
                        enumeration(0x0042_0128, 1),
                        ttlv_item(0x0042_00D6, 0x08, &correlation),
                    ],
                ),
            ],
        ),
        structure(
            0x0042_000F,
            [
                enumeration(0x0042_005C, 33),
                ttlv_item(0x0042_0093, 0x08, b"sign00"),
                enumeration(0x0042_007F, 0),
                structure(
                    0x0042_007C,
                    [
                        ttlv_item(0x0042_0094, 0x07, response_identifier),
                        ttlv_item(0x0042_00C7, 0x08, b"signature-response-data"),
                        ttlv_item(0x0042_00D6, 0x08, &correlation),
                    ],
                ),
            ],
        ),
        structure(
            0x0042_000F,
            [
                enumeration(0x0042_005C, 34),
                ttlv_item(0x0042_0093, 0x08, b"sigvfy"),
                enumeration(0x0042_007F, 0),
                structure(
                    0x0042_007C,
                    [
                        ttlv_item(0x0042_0094, 0x07, response_identifier),
                        enumeration(0x0042_0128, 1),
                        ttlv_item(0x0042_00C2, 0x08, b"recovered-response-data"),
                        ttlv_item(0x0042_00D6, 0x08, &correlation),
                    ],
                ),
            ],
        ),
        structure(
            0x0042_000F,
            [
                enumeration(0x0042_005C, 6),
                ttlv_item(0x0042_0093, 0x08, b"certif"),
                enumeration(0x0042_007F, 0),
                structure(
                    0x0042_007C,
                    [ttlv_item(0x0042_0094, 0x07, b"issued-certificate")],
                ),
            ],
        ),
    ]
}

fn discover_version_response_items(response_payload: Vec<u8>) -> [Vec<u8>; 4] {
    [
        structure(
            0x0042_000F,
            [
                enumeration(0x0042_005C, 30),
                ttlv_item(0x0042_0093, 0x08, b"item-0"),
                enumeration(0x0042_007F, 0),
                response_payload,
            ],
        ),
        structure(
            0x0042_000F,
            [
                enumeration(0x0042_005C, 30),
                ttlv_item(0x0042_0093, 0x08, b"item-1"),
                enumeration(0x0042_007F, 2),
                ttlv_item(0x0042_0006, 0x08, &[0x00, 0xa1, 0x7f]),
                structure(0x0042_007C, []),
            ],
        ),
        structure(
            0x0042_000F,
            [
                enumeration(0x0042_005C, 30),
                ttlv_item(0x0042_0093, 0x08, b"item-2"),
                enumeration(0x0042_007F, 1),
                enumeration(0x0042_007E, 1),
            ],
        ),
        structure(
            0x0042_000F,
            [
                enumeration(0x0042_005C, 30),
                ttlv_item(0x0042_0093, 0x08, b"item-3"),
                enumeration(0x0042_007F, 0),
                structure(0x0042_007C, []),
            ],
        ),
    ]
}

fn creation_response_items(
    failure_message: &[u8],
    create_key_pair_expected: &serde_json::Value,
) -> [Vec<u8>; 4] {
    let create_payload = structure(
        0x0042_007C,
        [enumeration(0x0042_0057, 7), integer(0x0042_0094, -8)],
    );
    let private_identifier = &create_key_pair_expected["privateUniqueIdentifier"];
    assert_eq!(private_identifier["wireType"], "Enumeration");
    let private_enumeration = private_identifier["enumeration"]
        .as_u64()
        .and_then(|value| u32::try_from(value).ok())
        .expect("fixture private key identifier is a U32 Enumeration");
    let public_identifier = &create_key_pair_expected["publicUniqueIdentifier"];
    assert_eq!(public_identifier["wireType"], "TextString");
    let public_text = public_identifier["text"]
        .as_str()
        .expect("fixture public key identifier is a TextString");
    let create_key_pair_payload = structure(
        0x0042_007C,
        [
            enumeration(0x0042_0066, private_enumeration),
            ttlv_item(0x0042_006F, 0x07, public_text.as_bytes()),
        ],
    );
    let create_split_key_payload = structure(
        0x0042_007C,
        (0..100_001).map(|index| integer(0x0042_0094, index)),
    );
    [
        structure(
            0x0042_000F,
            [
                enumeration(0x0042_005C, 1),
                ttlv_item(0x0042_0093, 0x08, b"item-4"),
                enumeration(0x0042_007F, 0),
                create_payload,
            ],
        ),
        structure(
            0x0042_000F,
            [
                enumeration(0x0042_005C, 1),
                ttlv_item(0x0042_0093, 0x08, b"item-5"),
                enumeration(0x0042_007F, 1),
                enumeration(0x0042_007E, 5),
                ttlv_item(0x0042_007D, 0x07, failure_message),
            ],
        ),
        structure(
            0x0042_000F,
            [
                enumeration(0x0042_005C, 2),
                ttlv_item(0x0042_0093, 0x08, b"item-6"),
                enumeration(0x0042_007F, 0),
                create_key_pair_payload,
            ],
        ),
        structure(
            0x0042_000F,
            [
                enumeration(0x0042_005C, 40),
                ttlv_item(0x0042_0093, 0x08, b"item-7"),
                enumeration(0x0042_007F, 0),
                create_split_key_payload,
            ],
        ),
    ]
}

fn mismatched_discover_versions_response() -> Vec<u8> {
    let protocol_version = structure(
        0x0042_0069,
        [integer(0x0042_006A, 2), integer(0x0042_006B, 1)],
    );
    let header = structure(
        0x0042_007A,
        [
            protocol_version.clone(),
            date_time(0x0042_0092, 1),
            integer(0x0042_000D, 1),
        ],
    );
    let item = structure(
        0x0042_000F,
        [
            enumeration(0x0042_005C, 30),
            ttlv_item(0x0042_0093, 0x08, &[0xaa]),
            enumeration(0x0042_007F, 0),
            structure(0x0042_007C, [ttlv_item(0x0042_0069, 0x08, &[2, 1])]),
        ],
    );
    structure(
        0x0042_007B,
        [
            header,
            item,
            ttlv_item(0x0054_0001, 0x08, &[0xde, 0xad, 0x00, 0xff]),
        ],
    )
}

fn mac_response_missing_unique_identifier() -> Vec<u8> {
    let protocol_version = structure(
        0x0042_0069,
        [integer(0x0042_006A, 2), integer(0x0042_006B, 1)],
    );
    let header = structure(
        0x0042_007A,
        [
            protocol_version,
            date_time(0x0042_0092, 1),
            integer(0x0042_000D, 1),
        ],
    );
    let payload = structure(
        0x0042_007C,
        [
            ttlv_item(0x0042_00C4, 0x08, &[0x5a]),
            ttlv_item(0x0054_0001, 0x08, &[0xde, 0xad, 0x00, 0xff]),
        ],
    );
    let item = structure(
        0x0042_000F,
        [
            enumeration(0x0042_005C, 35),
            ttlv_item(0x0042_0093, 0x08, b"item-0"),
            enumeration(0x0042_007F, 0),
            payload,
        ],
    );
    structure(0x0042_007B, [header, item])
}

fn structure(tag: u32, children: impl IntoIterator<Item = Vec<u8>>) -> Vec<u8> {
    let value = children.into_iter().flatten().collect::<Vec<_>>();
    ttlv_item(tag, 0x01, &value)
}

fn integer(tag: u32, value: i32) -> Vec<u8> {
    ttlv_item(tag, 0x02, &value.to_be_bytes())
}

fn enumeration(tag: u32, value: u32) -> Vec<u8> {
    ttlv_item(tag, 0x05, &value.to_be_bytes())
}

fn date_time(tag: u32, value: i64) -> Vec<u8> {
    ttlv_item(tag, 0x09, &value.to_be_bytes())
}

fn ttlv_item(tag: u32, item_type: u8, value: &[u8]) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(8 + value.len().div_ceil(8) * 8);
    encoded.extend_from_slice(&tag.to_be_bytes()[1..]);
    encoded.push(item_type);
    encoded.extend_from_slice(
        &u32::try_from(value.len())
            .expect("test response element length fits the TTLV field")
            .to_be_bytes(),
    );
    encoded.extend_from_slice(value);
    encoded.resize(8 + value.len().div_ceil(8) * 8, 0);
    encoded
}
