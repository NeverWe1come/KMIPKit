//! The marked examples in both language guides must remain executable.

const ENGLISH_GUIDE: &str = include_str!("../../../docs/user-guide/en/cryptographic-operations.md");
const SPANISH_GUIDE: &str =
    include_str!("../../../docs/user-guide/es/operaciones-criptograficas.md");

use kmipkit_protocol::{
    HashRequest, HashingAlgorithm, MacRequest, MacVerifyRequest, OperationData, SecretBytes,
    SignRequest, SignatureVerifyRequest, UniqueIdentifier,
};
use kmipkit_ttlv::{Item, RawTag, Structure, Value};

#[test]
fn both_guides_contain_executable_cryptographic_operation_examples() {
    for guide in [ENGLISH_GUIDE, SPANISH_GUIDE] {
        assert!(guide.contains("```rust,kmipkit-test"));
        assert!(guide.contains("HashRequest"));
        assert!(guide.contains("MacRequest"));
        assert!(guide.contains("MacVerifyRequest"));
        assert!(guide.contains("SignRequest"));
        assert!(guide.contains("SignatureVerifyRequest"));
        assert!(guide.contains("KMIPKIT-DISC-048"));
    }
}

#[test]
fn documented_operation_inputs_are_redacted_by_each_public_request_model() {
    const HASH: &str = "hash-input-example";
    const MAC: &str = "mac-input-example";
    const ORIGINAL: &str = "original-data-example";
    const MAC_DATA: &str = "mac-data-example";
    const SIGN: &str = "sign-input-example";
    const SIGNATURE: &str = "signature-data-example";
    const CORRELATION: &str = "correlation-secret-example";

    for guide in [ENGLISH_GUIDE, SPANISH_GUIDE] {
        for sample in [HASH, MAC, ORIGINAL, MAC_DATA, SIGN, SIGNATURE] {
            assert!(guide.contains(sample), "missing documented sample {sample}");
        }
    }

    let hash = HashRequest::new(hash_parameters()).with_data(OperationData::ByteString(
        SecretBytes::new(HASH.as_bytes().to_vec()),
    ));
    assert_redacted(&format!("{hash:?}"), HASH);

    let mac = MacRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString("key-id".to_owned()))
        .with_data(OperationData::ByteString(SecretBytes::new(
            MAC.as_bytes().to_vec(),
        )));
    assert_redacted(&format!("{mac:?}"), MAC);

    let mac_verify = MacVerifyRequest::new()
        .with_data(OperationData::ByteString(SecretBytes::new(
            ORIGINAL.as_bytes().to_vec(),
        )))
        .with_mac_data(SecretBytes::new(MAC_DATA.as_bytes().to_vec()));
    assert_redacted(&format!("{mac_verify:?}"), ORIGINAL);
    assert_redacted(&format!("{mac_verify:?}"), MAC_DATA);

    let sign = SignRequest::new().with_data(OperationData::ByteString(SecretBytes::new(
        SIGN.as_bytes().to_vec(),
    )));
    assert_redacted(&format!("{sign:?}"), SIGN);

    let signature_verify = SignatureVerifyRequest::new()
        .with_data(OperationData::ByteString(SecretBytes::new(
            ORIGINAL.as_bytes().to_vec(),
        )))
        .with_signature_data(SecretBytes::new(SIGNATURE.as_bytes().to_vec()));
    assert_redacted(&format!("{signature_verify:?}"), ORIGINAL);
    assert_redacted(&format!("{signature_verify:?}"), SIGNATURE);

    let final_hash_part = HashRequest::new(hash_parameters())
        .with_correlation_value(SecretBytes::new(CORRELATION.as_bytes().to_vec()))
        .with_final_indicator(true);
    assert_redacted(&format!("{final_hash_part:?}"), CORRELATION);
}

fn hash_parameters() -> Structure {
    let tag = RawTag::new(0x0042_0038)
        .expect("Hashing Algorithm uses a 24-bit KMIP tag")
        .try_checked()
        .expect("Hashing Algorithm is allocated");
    let item = Item::new(tag, Value::enumeration(HashingAlgorithm::from_raw(6).raw()))
        .expect("the sample hashing algorithm is a valid Enumeration");
    let mut parameters = Structure::new();
    parameters
        .try_push(item)
        .expect("the example parameter fits in the Structure");
    parameters
}

fn assert_redacted(debug: &str, sentinel: &str) {
    assert!(!debug.contains(sentinel));
}
