//! Client-boundary borrowed codec-limit identity tests.
//!
//! This is a `KMIPKit` project contract, not a new OASIS requirement.
//! Traceability: `KMIPKIT-0007-FR-003`, `KMIPKIT-0007-FR-009`,
//! `KMIPKIT-0007-SC-003`, and ADR-0012.

use std::cell::RefCell;
use std::rc::Rc;

use kmipkit_test_support::{ExchangeScript, ScriptedTransport};
use kmipkit_transport::{Transport, TransportError, TransportResponse};
use kmipkit_ttlv::codec::CodecLimits;

use crate::execute::{Client, ClientBatch, ClientBatchItem, ClientRequest, LimitsIdentityObserver};
use crate::execute_test_support::{ResponseItemFixture, response_bytes};

struct SharedScriptedTransport(Rc<RefCell<ScriptedTransport>>);

impl Transport for SharedScriptedTransport {
    fn exchange(
        &mut self,
        request: &[u8],
        max_response_bytes: usize,
    ) -> Result<TransportResponse, TransportError> {
        self.0.borrow_mut().exchange(request, max_response_bytes)
    }
}

#[test]
fn execute_passes_the_same_codec_limits_instance_to_writer_and_decoder() {
    let limits = CodecLimits::defaults();
    let observer = LimitsIdentityObserver::new(&limits);
    let response = response_bytes((2, 1), &[ResponseItemFixture::success(None)]);
    let fake = Rc::new(RefCell::new(ScriptedTransport::new(
        ExchangeScript::Success {
            response,
            request_write_chunks: vec![],
        },
    )));
    let mut client = Client::for_test_with_limits_observer(
        SharedScriptedTransport(Rc::clone(&fake)),
        observer.clone(),
    );
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::discover_versions()));

    client
        .execute(batch, &limits)
        .expect("the scripted Discover Versions exchange succeeds");

    assert!(observer.both_same());
    assert_eq!(fake.borrow().exchange_count(), 1);
}
