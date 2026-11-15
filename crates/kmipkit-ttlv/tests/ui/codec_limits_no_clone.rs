use kmipkit_ttlv::codec::CodecLimits;

fn require_clone<T: Clone>() {}

fn main() {
    // KMIPKit resource configuration is borrowed immutably rather than copied: FR-007.
    require_clone::<CodecLimits>();
}
