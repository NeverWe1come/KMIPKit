use crate::high_level::{
    AttributesBuilder, CipherSpec, HashAlgorithm, KdfSpec, KemSpec, KeySpec, MacSpec, PresetError,
    SignatureSpec, decode_pem, decode_pem_der,
};
use crate::named_values_generated::values::enumeration::recommended_curve;
use crate::typed_attributes::typed;
use crate::{Authentication, LinkType, ManagedObject, ProtocolVersion};

#[test]
fn key_spec_covers_classical_and_kmip30_pqc_presets() {
    assert!(KeySpec::Aes256.is_symmetric());
    assert_eq!(KeySpec::Aes256.length(), 256);
    assert_eq!(KeySpec::Aes256.min_kmip_version(), ProtocolVersion::V2_0);

    assert!(KeySpec::EcdsaP256.is_asymmetric());
    assert_eq!(
        KeySpec::EcdsaP256.recommended_curve_raw(),
        Some(recommended_curve::P_256)
    );
    assert!(KeySpec::MlKem768.requires_kmip_30());
    assert_eq!(KeySpec::MlKem768.min_kmip_version(), ProtocolVersion::V3_0);
    assert!(KeySpec::MlDsa65.requires_kmip_30());
    assert!(KeySpec::SlhDsaSha2_128f.requires_kmip_30());
    assert!(KeySpec::X25519MlKem768.requires_kmip_30());
    assert!(KeySpec::Secp256r1MlKem768.requires_kmip_30());
}

#[test]
fn cipher_signature_mac_kem_and_kdf_presets_produce_valid_parameters() {
    for cipher in [
        CipherSpec::AesCbcPkcs5,
        CipherSpec::AesCbcNoPadding,
        CipherSpec::AesGcm,
        CipherSpec::AesCtr,
        CipherSpec::ChaCha20Poly1305,
        CipherSpec::RsaOaepSha256,
        CipherSpec::RsaOaepSha384,
        CipherSpec::RsaOaepSha512,
        CipherSpec::RsaPkcs1v15,
    ] {
        assert!(cipher.to_cryptographic_parameters().is_ok());
    }
    for sig in [
        SignatureSpec::EcdsaSha256,
        SignatureSpec::EcdsaSha384,
        SignatureSpec::EcdsaSha512,
        SignatureSpec::RsaPssSha256,
        SignatureSpec::RsaPssSha384,
        SignatureSpec::RsaPssSha512,
        SignatureSpec::RsaPkcs1Sha256,
        SignatureSpec::RsaPkcs1Sha384,
        SignatureSpec::RsaPkcs1Sha512,
        SignatureSpec::Ed25519,
        SignatureSpec::Ed448,
        SignatureSpec::MlDsa44,
        SignatureSpec::MlDsa65,
        SignatureSpec::MlDsa87,
        SignatureSpec::MlDsa65Deterministic,
        SignatureSpec::SlhDsaSha2_128f,
    ] {
        let params = sig.to_cryptographic_parameters().expect("valid sig params");
        if sig == SignatureSpec::MlDsa65Deterministic {
            assert_eq!(params.deterministic(), Some(true));
        }
    }
    for mac in [
        MacSpec::HmacSha256,
        MacSpec::HmacSha384,
        MacSpec::HmacSha512,
        MacSpec::HmacSha3_256,
        MacSpec::HmacSha3_512,
        MacSpec::AesCmac,
    ] {
        assert!(mac.to_cryptographic_parameters().is_ok());
    }
    for hash in [
        HashAlgorithm::Sha256,
        HashAlgorithm::Sha384,
        HashAlgorithm::Sha512,
        HashAlgorithm::Sha3_256,
        HashAlgorithm::Sha3_384,
        HashAlgorithm::Sha3_512,
    ] {
        assert!(hash.to_cryptographic_parameters().is_ok());
    }
    for kem in [
        KemSpec::MlKem512,
        KemSpec::MlKem768,
        KemSpec::MlKem1024,
        KemSpec::DhKem,
        KemSpec::RsaSve,
        KemSpec::X25519MlKem768,
        KemSpec::Secp256r1MlKem768,
    ] {
        let params = kem.to_cryptographic_parameters().expect("valid kem params");
        assert_eq!(params.kem_algorithm(), Some(kem.kem_algorithm_raw()));
    }
    let kdf_params = KdfSpec::Pbkdf2Sha256
        .to_derivation_parameters(Some(vec![1, 2, 3, 4]), None, Some(10_000))
        .expect("valid pbkdf2 params");
    let dbg = format!("{kdf_params:?}");
    assert!(dbg.contains("has_cryptographic_parameters: true"));
    assert!(dbg.contains("has_salt: true"));
    assert!(dbg.contains("has_iteration_count: true"));
}

#[test]
fn pem_decoder_handles_valid_and_malformed_blocks() {
    let pem = "-----BEGIN CERTIFICATE-----\nAQIDBA==\n-----END CERTIFICATE-----\n";
    let decoded = decode_pem(pem).expect("valid pem");
    assert_eq!(decoded.label, "CERTIFICATE");
    assert_eq!(decoded.der, vec![1, 2, 3, 4]);

    let der = decode_pem_der(pem, &["CERTIFICATE"]).expect("matching label");
    assert_eq!(der, vec![1, 2, 3, 4]);
    assert!(matches!(
        decode_pem_der(pem, &["PUBLIC KEY"]),
        Err(PresetError::InvalidPem(_))
    ));
    assert!(matches!(decode_pem(""), Err(PresetError::EmptyInput)));
    assert!(matches!(
        decode_pem("-----BEGIN CERTIFICATE-----\n@@@=\n-----END CERTIFICATE-----"),
        Err(PresetError::InvalidBase64)
    ));
    assert!(matches!(
        decode_pem("-----BEGIN CERTIFICATE-----\nAQID\n-----END PUBLIC KEY-----"),
        Err(PresetError::InvalidPem(_))
    ));
}

#[test]
fn managed_object_and_authentication_constructors_validate_inputs() {
    let aes = ManagedObject::symmetric_key_bytes(KeySpec::Aes256, vec![0xAA; 32])
        .expect("aes256 accepts 32 bytes");
    assert_eq!(aes.object_type().raw(), 2);

    assert!(matches!(
        ManagedObject::symmetric_key_bytes(KeySpec::Aes256, vec![0xAA; 16]),
        Err(PresetError::InvalidKeyLength {
            expected: 32,
            actual: 16
        })
    ));
    assert!(matches!(
        ManagedObject::symmetric_key_bytes(KeySpec::Rsa2048, vec![0xAA; 256]),
        Err(PresetError::IncompatibleKeySpec(_))
    ));

    let priv_pem = "-----BEGIN PRIVATE KEY-----\nMIIBAQ==\n-----END PRIVATE KEY-----";
    let priv_obj =
        ManagedObject::private_key_pkcs8_pem(KeySpec::Rsa2048, priv_pem).expect("pkcs8 pem");
    assert_eq!(priv_obj.object_type().raw(), 4);

    let pub_pem = "-----BEGIN PUBLIC KEY-----\nMIIBAg==\n-----END PUBLIC KEY-----";
    let pub_obj =
        ManagedObject::public_key_spki_pem(KeySpec::EcdsaP256, pub_pem).expect("spki pem");
    assert_eq!(pub_obj.object_type().raw(), 3);

    let cert_pem = "-----BEGIN CERTIFICATE-----\nMIIBAw==\n-----END CERTIFICATE-----";
    let cert_obj = ManagedObject::certificate_x509_pem(cert_pem).expect("x509 pem");
    assert_eq!(cert_obj.object_type().raw(), 1);

    let secret = ManagedObject::secret_data_password(b"s3cr3t".to_vec()).expect("secret password");
    assert_eq!(secret.object_type().raw(), 7);
    let opaque = ManagedObject::opaque(1, vec![9, 8, 7]).expect("opaque object");
    assert_eq!(opaque.object_type().raw(), 8);

    let auth_user_pass = Authentication::username_password("alice", Some("pw")).expect("up auth");
    assert_eq!(auth_user_pass.credentials().len(), 1);
    let auth_dev =
        Authentication::device(Some("SN-1"), None, None, None, None, None).expect("device auth");
    assert_eq!(auth_dev.credentials().len(), 1);
    let auth_ticket = Authentication::ticket(1, vec![0xDE, 0xAD]).expect("ticket auth");
    assert_eq!(auth_ticket.credentials().len(), 1);
    let auth_hashed_pw = Authentication::hashed_password(
        "bob",
        1_700_000_000,
        HashAlgorithm::Sha256,
        vec![0x11; 32],
    )
    .expect("hashed password auth");
    assert_eq!(auth_hashed_pw.credentials().len(), 1);
}

#[test]
fn attributes_builder_adapts_links_across_kmip_versions() {
    let builder_v2 = || {
        AttributesBuilder::new()
            .key_spec(KeySpec::EcdsaP256)
            .name("signing-key")
            .description("ECDSA P-256")
            .object_group("prod")
            .sensitive(true)
            .extractable(false)
            .link(LinkType::Certificate, "cert-1")
    };
    let set_v2 = builder_v2()
        .build_for_version(ProtocolVersion::V2_1)
        .expect("v2 attributes");
    assert_eq!(set_v2.typed::<typed::Link>().count(), 1);

    let set_v3 = builder_v2()
        .build_for_version(ProtocolVersion::V3_0)
        .expect("v3 attributes");
    assert_eq!(set_v3.typed::<typed::Link>().count(), 0);
    assert!(
        set_v3
            .as_items()
            .iter()
            .any(|item| item.tag().raw() == LinkType::Certificate.v3_tag())
    );
}
