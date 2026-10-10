//! The marked examples in both language guides must remain executable.

const ENGLISH_GUIDE: &str = include_str!("../../../docs/user-guide/en/cryptographic-operations.md");
const SPANISH_GUIDE: &str = include_str!("../../../docs/user-guide/es/operaciones-criptograficas.md");

#[test]
fn both_guides_contain_executable_cryptographic_operation_examples() {
    for guide in [ENGLISH_GUIDE, SPANISH_GUIDE] {
        assert!(guide.contains("```rust,kmipkit-test"));
        assert!(guide.contains("HashRequest"));
        assert!(guide.contains("MacVerifyRequest"));
        assert!(guide.contains("SignatureVerifyRequest"));
    }
}
