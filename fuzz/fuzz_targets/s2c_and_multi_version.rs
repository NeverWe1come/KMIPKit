#![no_main]
#![forbid(unsafe_code)]

#[path = "support/ttlv_clone.rs"]
mod ttlv_clone;

use kmipkit_client::server_to_client::IncomingServerBatch;
use kmipkit_protocol::{RequestMessage, ResponseMessage, VersionGate};
use kmipkit_transport::WireEncoding;
use kmipkit_ttlv::codec::{CodecLimits, decode_with_limits};
use kmipkit_ttlv::ValueView;
use libfuzzer_sys::fuzz_target;

const MAX_INPUT_BYTES: usize = 4 * 1024;
const MAX_STRUCTURE_DEPTH: usize = 32;
const MAX_ITEMS: usize = 256;

fuzz_target!(|data: &[u8]| {
    if data.len() > MAX_INPUT_BYTES {
        return;
    }
    let Ok(limits) = CodecLimits::new(MAX_INPUT_BYTES, MAX_STRUCTURE_DEPTH, MAX_ITEMS) else {
        return;
    };

    // 1. Fuzz Server-to-Client batch ingestion across all three supported encodings
    for encoding in [WireEncoding::Ttlv, WireEncoding::Json, WireEncoding::Xml] {
        let _ = IncomingServerBatch::from_encoded_bytes(data, encoding, &limits);
    }

    // 2. Fuzz ProtocolVersion and multi-version negotiation structures
    if let Ok(item) = decode_with_limits(data, &limits) {
        let req_tree = item.with_value(|v| match v {
            ValueView::Structure(s) => ttlv_clone::clone_structure(&s),
            _ => None,
        });
        if let Some(structure) = req_tree {
            if let Ok(req) = RequestMessage::try_from_ttlv(structure) {
                let v = req.header().protocol_version();
                let _ = v.is_catalogued();
                for item in req.batch_items() {
                    if let Some(op) = item.operation() {
                        let _ = VersionGate::validate_operation(v, op);
                    }
                }
            }
        }

        let resp_tree = item.with_value(|v| match v {
            ValueView::Structure(s) => ttlv_clone::clone_structure(&s),
            _ => None,
        });
        if let Some(structure) = resp_tree {
            if let Ok(resp) = ResponseMessage::try_from_ttlv(structure) {
                let v = resp.header().protocol_version();
                let _ = v.is_catalogued();
            }
        }
    }
});
