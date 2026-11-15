#![no_main]
#![forbid(unsafe_code)]

use kmipkit_client::__fuzz::wire_framing::TextDocumentFramer;
use kmipkit_client::__fuzz::xml_codec;
use kmipkit_transport::WireEncoding;
use kmipkit_ttlv::codec::CodecLimits;
use libfuzzer_sys::fuzz_target;

const MAX_INPUT_BYTES: usize = 4 * 1024;
const MAX_STRUCTURE_DEPTH: usize = 32;
const MAX_ITEMS: usize = 256;

fuzz_target!(|data: &[u8]| {
    if data.len() > MAX_INPUT_BYTES {
        return;
    }

    // 1. Fuzz incremental text framing scanners for both JSON and XML
    for encoding in [WireEncoding::Json, WireEncoding::Xml] {
        if let Ok(mut framer) = TextDocumentFramer::new(encoding) {
            for &byte in data {
                if framer.feed(byte).is_err() {
                    break;
                }
            }
            let _ = framer.is_complete();
        }
    }

    // 2. Fuzz XML preflight and decoding logic
    let Ok(limits) = CodecLimits::new(MAX_INPUT_BYTES, MAX_STRUCTURE_DEPTH, MAX_ITEMS) else {
        return;
    };

    if let Ok(item) = xml_codec::decode(data, &limits) {
        if let Ok(encoded) = xml_codec::encode(&item, &limits) {
            let _ = xml_codec::decode(&encoded, &limits);
        }
    }
});
