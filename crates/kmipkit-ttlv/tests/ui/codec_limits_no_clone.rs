use kmipkit_ttlv::codec::CodecLimits;

fn require_clone<T: Clone>() {}

fn main() {
    // KMIPKit resource configuration is borrowed immutably rather than copied: to ensure deterministic limit enforcement.
    require_clone::<CodecLimits>();
}
