#![no_main]
#![forbid(unsafe_code)]

use kmipkit_ttlv::codec::{CodecLimits, decode_with_limits};
use libfuzzer_sys::fuzz_target;

const MAX_INPUT_BYTES: usize = 4 * 1024;
const MAX_STRUCTURE_DEPTH: usize = 64;
const MAX_ITEMS: usize = 512;

fuzz_target!(|data: &[u8]| {
    if data.len() > MAX_INPUT_BYTES {
        return;
    }

    let Ok(limits) = CodecLimits::new(MAX_INPUT_BYTES, MAX_STRUCTURE_DEPTH, MAX_ITEMS) else {
        return;
    };
    let _ = decode_with_limits(data, &limits);
});
