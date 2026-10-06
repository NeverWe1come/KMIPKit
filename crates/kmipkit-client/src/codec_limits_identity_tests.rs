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

struct ObservedCodecLimitsAddresses {
    encode: *const CodecLimits,
    decode: *const CodecLimits,
}

/// Deliberately broken client-path candidate. The decode branch reconstructs
/// defaults instead of forwarding the single caller-owned borrow; Green must
/// remove this test-only divergence and pass `limits` to both helper seams.
fn candidate_client_codec_path(limits: &CodecLimits) -> ObservedCodecLimitsAddresses {
    let encode = candidate_bounded_encode_helper(limits);
    let reconstructed_decode_limits = CodecLimits::defaults();
    let decode = candidate_bounded_decode_helper(&reconstructed_decode_limits);

    ObservedCodecLimitsAddresses { encode, decode }
}

#[test]
fn bounded_encode_and_decode_helpers_observe_the_same_codec_limits_instance() {
    let caller_limits = CodecLimits::defaults();
    let observed = candidate_client_codec_path(&caller_limits);

    assert!(
        std::ptr::eq(observed.encode, observed.decode),
        "codec limits argument addresses differ: encode={:p}, decode={:p}",
        observed.encode,
        observed.decode
    );
}
