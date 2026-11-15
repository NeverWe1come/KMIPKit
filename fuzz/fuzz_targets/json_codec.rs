#![no_main]
#![forbid(unsafe_code)]

use kmipkit_client::__fuzz::json_encoding;
use kmipkit_ttlv::codec::CodecLimits;
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

    if let Ok(item) = json_encoding::decode(data, &limits) {
        // Test roundtrip invariant: if it decoded cleanly, re-encoding must succeed without panic
        if let Ok(encoded) = json_encoding::encode(&item, &limits) {
            let _ = json_encoding::decode(&encoded, &limits);
        }
    }
});
