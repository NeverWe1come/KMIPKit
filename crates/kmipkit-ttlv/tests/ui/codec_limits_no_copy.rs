use kmipkit_ttlv::codec::CodecLimits;

fn require_copy<T: Copy>() {}

fn main() {
    // KMIPKit resource configuration is borrowed immutably rather than copied: to ensure deterministic limit enforcement.
    require_copy::<CodecLimits>();
}
