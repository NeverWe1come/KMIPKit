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

const MAX_TEST_TTLV_VALUE_BYTES: usize = 1_048_576;
static TEMP_DIRECTORY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[link(name = "kmipkit_extension_registry_c_consumer", kind = "static")]
unsafe extern "C" {
    fn kmipkit_extension_registry_c_consumer_main(argc: i32, argv: *mut *mut c_char) -> i32;
}

#[link(name = "kmipkit_client_abi_c_consumer", kind = "static")]
unsafe extern "C" {
    fn kmipkit_client_abi_c_consumer_main(argc: i32, argv: *mut *mut c_char) -> i32;
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
fn generated_public_client_abi_consumer_executes_a_typed_batch_over_mtls() {
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
    let peer = spawn_public_client_batch_peer(listener, server_config, Arc::clone(&exchange_count));

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
    assert_eq!(
        result,
        0,
        "generated public C consumer failed (status {last_status}, category {last_category}, cause {last_cause}, delivery {last_delivery}); server exchanges: {}",
        exchange_count.load(Ordering::Relaxed)
    );
    peer.join().expect("the loopback TLS peer completes");
    assert_eq!(exchange_count.load(Ordering::Relaxed), 1);
}

fn spawn_public_client_batch_peer(
    listener: TcpListener,
    server_config: Arc<ServerConfig>,
    exchange_count: Arc<AtomicU64>,
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
        tls.write_all(&mixed_discover_versions_response())
            .expect("the typed KMIP response is written");
        tls.flush().expect("the TLS response is flushed");
        let _ = tls.get_ref().shutdown(Shutdown::Both);
    })
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

    let Ok(root) = kmipkit_ttlv::codec::decode(encoded) else {
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

    let Ok(root) = kmipkit_ttlv::codec::decode(encoded) else {
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

    let Ok(root) = kmipkit_ttlv::codec::decode(encoded) else {
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
    let Ok(root) = kmipkit_ttlv::codec::decode(encoded) else {
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
    let Ok(root) = kmipkit_ttlv::codec::decode(encoded) else {
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

    let Ok(root) = kmipkit_ttlv::codec::decode(encoded) else {
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

    let Ok(root) = kmipkit_ttlv::codec::decode(encoded) else {
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

// This TLS batch returns 26 ordered items, including the successful Login Ticket, Encrypt, and Decrypt results.
fn mixed_discover_versions_response() -> Vec<u8> {
    let protocol_version = structure(
        0x0042_0069,
        [integer(0x0042_006A, 2), integer(0x0042_006B, 1)],
    );
    let header = structure(
        0x0042_007A,
        [
            protocol_version.clone(),
            date_time(0x0042_0092, 1),
            integer(0x0042_000D, 26),
        ],
    );
    let response_payload = structure(
        0x0042_007C,
        [protocol_version.clone(), protocol_version.clone()],
    );
    structure(
        0x0042_007B,
        std::iter::once(header)
            .chain(discover_version_response_items(response_payload))
            .chain(creation_response_items())
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
            .chain(encrypt_decrypt_response_items()),
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

fn creation_response_items() -> [Vec<u8>; 4] {
    let create_payload = structure(
        0x0042_007C,
        [enumeration(0x0042_0057, 7), integer(0x0042_0094, -8)],
    );
    let create_key_pair_payload = structure(
        0x0042_007C,
        [
            enumeration(0x0042_0066, u32::MAX),
            ttlv_item(0x0042_006F, 0x07, "public-key-α".as_bytes()),
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
                ttlv_item(0x0042_007D, 0x07, "création refusée ✓".as_bytes()),
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
