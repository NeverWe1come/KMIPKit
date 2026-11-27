//! Unit tests verifying Fase 3 domain namespaces, KeyRef, presets, and high-level models.

use kmipkit_protocol::{
    AttributesBuilder, CipherSpec, HashAlgorithm, KdfSpec, KemSpec, KeySpec, MacSpec,
    ProtocolVersion, SignatureSpec,
};
use kmipkit_test_support::ExchangeScript;

use crate::asynchronous_execution_test_support::client_for;
use crate::execute_test_support::{asynchronous_response_bytes, test_structure};
use crate::fluent::{CryptographicUsageMask, FluentBuildError};

#[test]
fn domain_namespaces_and_key_ref_build_expected_requests() {
    let ok = asynchronous_response_bytes(0x0000_003b, 0, None, None, Some(test_structure([])));
    let (mut client, _, _) = client_for(ExchangeScript::Success {
        response: ok,
        request_write_chunks: Vec::new(),
    });

    let sym_item = client
        .keys()
        .create_symmetric()
        .with_key_spec(KeySpec::Aes256)
        .usage_mask(CryptographicUsageMask::ENCRYPT | CryptographicUsageMask::DECRYPT)
        .name("master-aes")
        .to_batch_item()
        .expect("aes256 spec builds");
    assert_eq!(sym_item.operation(), 0x0000_0001);

    let err = match client
        .keys()
        .create_symmetric()
        .with_key_spec(KeySpec::Rsa2048)
        .usage_mask(CryptographicUsageMask::ENCRYPT)
        .build_request()
    {
        Ok(_) => panic!("asymmetric spec must be rejected on symmetric builder"),
        Err(err) => err,
    };
    assert_eq!(err, FluentBuildError::InvalidAttributeValue);

    let pqc_item = client
        .keys()
        .create_key_pair()
        .with_key_spec(KeySpec::MlKem768)
        .private_name("mlkem-priv")
        .public_name("mlkem-pub")
        .with_kmip_version(ProtocolVersion::V3_0)
        .to_batch_item()
        .expect("mlkem768 builds");
    assert_eq!(pqc_item.operation(), 0x0000_0002);

    let ec_item = client
        .keys()
        .create_key_pair()
        .with_key_spec(KeySpec::EcdsaP256)
        .private_name("ec-priv")
        .public_name("ec-pub")
        .to_batch_item()
        .expect("ecdsa p256 with recommended curve builds");
    assert_eq!(ec_item.operation(), 0x0000_0002);

    let derived = client
        .keys()
        .derive()
        .with_kdf_spec(
            KdfSpec::Pbkdf2Sha256,
            Some(kmipkit_protocol::SecretBytes::new(vec![1, 2, 3, 4])),
            None,
            Some(10_000),
        )
        .object_type(kmipkit_protocol::ObjectType::from_raw(2))
        .unique_identifier("base-key-1")
        .attributes(
            AttributesBuilder::new()
                .key_spec(KeySpec::Aes256)
                .name("derived-aes")
                .build()
                .expect("attrs"),
        )
        .to_batch_item()
        .expect("kdf derive builds");
    assert_eq!(derived.operation(), 0x0000_0005);

    let enc = client
        .key("k-1")
        .encrypt()
        .with_cipher_spec(CipherSpec::AesGcm)
        .data(b"hello".to_vec())
        .to_batch_item()
        .expect("encrypt preset builds");
    assert_eq!(enc.operation(), 0x0000_001F);

    let sig = client
        .key("k-1")
        .sign()
        .with_signature_spec(SignatureSpec::EcdsaSha256)
        .data(b"msg".to_vec())
        .to_batch_item()
        .expect("sign preset builds");
    assert_eq!(sig.operation(), 0x0000_0021);

    let mac = client
        .key("k-1")
        .mac()
        .with_mac_spec(MacSpec::HmacSha256)
        .data(b"msg".to_vec())
        .to_batch_item()
        .expect("mac preset builds");
    assert_eq!(mac.operation(), 0x0000_0023);

    let digest = client
        .crypto()
        .hash()
        .with_hash_algorithm(HashAlgorithm::Sha256)
        .data(b"msg".to_vec())
        .to_batch_item()
        .expect("hash preset builds");
    assert_eq!(digest.operation(), 0x0000_0027);

    let encap = client
        .key("kem-pub")
        .encapsulate()
        .with_kem_spec(KemSpec::MlKem768)
        .with_kmip_version(ProtocolVersion::V3_0)
        .to_batch_item();
    assert_eq!(encap.operation(), 0x0000_0041);

    let decap = client
        .key("kem-priv")
        .decapsulate(kmipkit_protocol::SecretBytes::new(vec![0xAA; 16]))
        .with_kem_spec(KemSpec::MlKem768)
        .with_kmip_version(ProtocolVersion::V3_0)
        .to_batch_item();
    assert_eq!(decap.operation(), 0x0000_0042);

    assert_eq!(client.certificates().get("cert-1").to_batch_item().operation(), 0x0000_000A);
    assert_eq!(client.secrets().obtain_lease("sec-1").to_batch_item().unwrap().operation(), 0x0000_0010);
    assert_eq!(client.attributes().get_list("k-1").to_batch_item().unwrap().operation(), 0x0000_000C);
    assert_eq!(client.sessions().login().to_batch_item().operation(), 0x0000_002D);
    assert_eq!(client.admin().ping().to_batch_item().operation(), 0x0000_003B);
    assert_eq!(client.async_ops().query().to_batch_item().operation(), 0x0000_0039);
}
