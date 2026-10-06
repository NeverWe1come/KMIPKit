//! Client-boundary borrowed codec-limit identity test candidates.
//!
//! This is a KMIPKit project contract, not a new OASIS requirement.
//! Traceability: `KMIPKIT-0007-FR-003`, `KMIPKIT-0007-FR-009`,
//! `KMIPKIT-0007-SC-003`, and ADR-0012.

use kmipkit_ttlv::codec::CodecLimits;

/// Test-only codec stubs observe only the borrowed argument's address.
fn candidate_ttlv_encode_with_limits(limits: &CodecLimits) -> *const CodecLimits {
    std::ptr::from_ref(limits)
}

fn candidate_ttlv_decode_with_limits(limits: &CodecLimits) -> *const CodecLimits {
    std::ptr::from_ref(limits)
}

/// Private client helper candidates forward the borrowed reference unchanged.
fn candidate_bounded_encode_helper(limits: &CodecLimits) -> *const CodecLimits {
    candidate_ttlv_encode_with_limits(limits)
}

fn candidate_bounded_decode_helper(limits: &CodecLimits) -> *const CodecLimits {
    candidate_ttlv_decode_with_limits(limits)
}

#[test]
fn bounded_encode_and_decode_helpers_observe_the_same_codec_limits_instance() {
    // These independent equal-default fixtures model the deliberately broken
    // candidate path where encode and decode receive separate limit objects.
    // No values are read or compared; only the helper-observed addresses matter.
    let encode_fixture_limits = CodecLimits::defaults();
    let decode_fixture_limits = CodecLimits::defaults();
    let encode_limits = candidate_bounded_encode_helper(&encode_fixture_limits);
    let decode_limits = candidate_bounded_decode_helper(&decode_fixture_limits);

    assert!(
        std::ptr::eq(encode_limits, decode_limits),
        "codec limits argument addresses differ: encode={encode_limits:p}, decode={decode_limits:p}"
    );
}
