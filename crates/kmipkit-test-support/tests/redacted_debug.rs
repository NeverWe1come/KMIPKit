use kmipkit_test_support::{ExchangeScript, ScriptedTransport};

const SUCCESS_SENTINEL: &[u8] = b"KMIPKIT_TEST_SUPPORT_SUCCESS_BODY_SENTINEL";
const PARTIAL_SENTINEL: &[u8] = b"KMIPKIT_TEST_SUPPORT_PARTIAL_BODY_SENTINEL";

#[test]
fn exchange_script_debug_redacts_success_response_bytes() {
    let script = ExchangeScript::Success {
        response: SUCCESS_SENTINEL.to_vec(),
        request_write_chunks: vec![1],
    };

    let debug = format!("{script:?}");
    let raw_bytes = format!("{:?}", SUCCESS_SENTINEL.to_vec());
    assert!(!debug.contains(&raw_bytes));
}

#[test]
fn exchange_script_debug_redacts_partial_response_bytes() {
    let script = ExchangeScript::FailAfterPartialRead {
        written_bytes: 1,
        response_bytes: PARTIAL_SENTINEL.to_vec(),
    };

    let debug = format!("{script:?}");
    let raw_bytes = format!("{:?}", PARTIAL_SENTINEL.to_vec());
    assert!(!debug.contains(&raw_bytes));
}

#[test]
fn scripted_transport_debug_redacts_response_bytes_in_its_script() {
    let transport = ScriptedTransport::new(ExchangeScript::Success {
        response: SUCCESS_SENTINEL.to_vec(),
        request_write_chunks: vec![1],
    });

    let debug = format!("{transport:?}");
    let raw_bytes = format!("{:?}", SUCCESS_SENTINEL.to_vec());
    assert!(!debug.contains(&raw_bytes));
}
