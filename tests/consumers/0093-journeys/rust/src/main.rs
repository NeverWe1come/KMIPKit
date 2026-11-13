use kmipkit::{
    Client, CryptographicUsageMask, RequestDeliveryState,
    client_config::ClientConfiguration,
    protocol::{
        Attribute, CryptographicParameters, CryptographicParametersBuilder, UniqueIdentifier,
        attribute::typed, named_values_generated::values::enumeration::recommended_curve,
    },
    transport_config::{
        CertificateInput, ClientIdentity, Endpoint, PrivateKeyInput, TransportConfig, TrustSource,
    },
    ttlv::{Item, RawTag, Structure, Value, codec::CodecLimits},
};

const AES_GCM_PLAINTEXT: &[u8] = b"KMIPKit AES-GCM journey";
const AES_GCM_CIPHERTEXT: &[u8] = &[
    0x0c, 0x4f, 0x9f, 0x4b, 0x8e, 0x8c, 0xb6, 0x3b, 0xcc, 0x04, 0xc4, 0xa6, 0xf6, 0xaa, 0x35, 0x4d,
    0xe9, 0xb9, 0xf2, 0x46, 0x9e, 0x1e, 0x26,
];
const AES_GCM_IV: &[u8] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
const AES_GCM_AAD: &[u8] = b"kmipkit-gcm-aad";
const AES_GCM_TAG: &[u8] = &[
    0xb5, 0x16, 0xad, 0x2a, 0x79, 0xbf, 0x55, 0xd1, 0x47, 0x48, 0x2f, 0x13, 0xfe, 0xed, 0x6e, 0xa6,
];
const SIGNATURE_PLAINTEXT: &[u8] = b"KMIPKit ECDSA signature journey";
const SIGNATURE_DATA: &[u8] = &[
    0x30, 0x45, 0x02, 0x21, 0x00, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x00, 0x11, 0x22, 0x33, 0x44,
    0x55, 0x66, 0x77, 0x88, 0x99, 0x02, 0x21, 0x00, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa,
    0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x00, 0x11,
];

fn ecdsa_sha256_parameters() -> Result<CryptographicParameters, Box<dyn std::error::Error>> {
    Ok(CryptographicParametersBuilder::new()
        .with_hashing_algorithm(6)
        .with_digital_signature_algorithm(14)
        .build()?)
}

fn aes_gcm_parameters() -> Result<CryptographicParameters, Box<dyn std::error::Error>> {
    Ok(CryptographicParametersBuilder::new()
        .with_cryptographic_algorithm(3)
        .with_block_cipher_mode(9)
        .with_iv_length(96)
        .with_tag_length(128)
        .build()?)
}

fn execute_fluent_key_lifecycle(
    client: &mut Client,
    limits: &CodecLimits,
) -> Result<(), Box<dyn std::error::Error>> {
    let created = client
        .request()
        .create_symmetric_key()
        .aes_256()
        .usage_mask(CryptographicUsageMask::ENCRYPT | CryptographicUsageMask::DECRYPT)
        .name("fluent-key")
        .execute(limits)?;
    let identifier = created
        .outcome()
        .create_response()
        .and_then(|response| response.unique_identifier())
        .cloned()
        .ok_or("Create did not return its typed Unique Identifier")?;
    assert_eq!(identifier, UniqueIdentifier::from("fluent-key"));

    let retrieved = client.request().get(identifier.clone()).execute(limits)?;
    let get_response = retrieved
        .outcome()
        .get_response()
        .ok_or("Get did not return a typed successful response")?;
    assert_eq!(get_response.result().status().raw(), 0);
    assert_eq!(get_response.unique_identifier(), Some(&identifier));
    assert!(get_response.object().is_some());
    let retrieved_identifier = get_response
        .unique_identifier()
        .cloned()
        .ok_or("Get did not return its typed Unique Identifier")?;

    let destroyed = client
        .request()
        .destroy(retrieved_identifier.clone())
        .execute(limits)?;
    let destroy_response = destroyed
        .outcome()
        .destroy_response()
        .ok_or("Destroy did not return a typed successful response")?;
    assert_eq!(destroy_response.result().status().raw(), 0);
    assert_eq!(
        destroy_response.unique_identifier(),
        Some(&retrieved_identifier)
    );
    println!("rust: fluent Create/Get/Destroy journey verified");
    Ok(())
}

fn execute_aes_gcm_journey(
    client: &mut Client,
    limits: &CodecLimits,
) -> Result<(), Box<dyn std::error::Error>> {
    let created = client
        .request()
        .create_symmetric_key()
        .aes_256()
        .usage_mask(CryptographicUsageMask::ENCRYPT | CryptographicUsageMask::DECRYPT)
        .name("fluent-aes-gcm-key")
        .execute(limits)?;
    let identifier = created
        .outcome()
        .create_response()
        .and_then(|response| response.unique_identifier())
        .cloned()
        .ok_or("Create did not return the AES-GCM key identifier")?;

    let encrypted = client
        .request()
        .encrypt()
        .unique_identifier(identifier.clone())
        .cryptographic_parameters(aes_gcm_parameters()?)
        .data(AES_GCM_PLAINTEXT.to_vec())
        .iv_counter_nonce(AES_GCM_IV.to_vec())
        .authenticated_encryption_additional_data(AES_GCM_AAD.to_vec())
        .execute(limits)?;
    let encrypt_response = encrypted
        .outcome()
        .encrypt_response()
        .ok_or("Encrypt did not return a typed successful response")?;
    if encrypt_response.result().status().raw() != 0
        || encrypt_response.unique_identifier() != Some(&identifier)
        || encrypt_response
            .iv_counter_nonce()
            .is_none_or(|iv| !iv.with_bytes(|bytes| bytes == AES_GCM_IV))
        || encrypt_response
            .authenticated_encryption_tag()
            .is_none_or(|tag| !tag.with_bytes(|bytes| bytes == AES_GCM_TAG))
    {
        return Err("Encrypt response did not preserve the AES-GCM fixture values".into());
    }
    let ciphertext = encrypt_response
        .data()
        .ok_or("Encrypt response omitted ciphertext")?
        .with_bytes(<[u8]>::to_vec);
    if ciphertext != AES_GCM_CIPHERTEXT {
        return Err("Encrypt response ciphertext did not match the fixture vector".into());
    }
    let authentication_tag = encrypt_response
        .authenticated_encryption_tag()
        .ok_or("Encrypt response omitted the authentication tag")?
        .with_bytes(<[u8]>::to_vec);

    let decrypted = client
        .request()
        .decrypt()
        .unique_identifier(identifier.clone())
        .cryptographic_parameters(aes_gcm_parameters()?)
        .data(ciphertext)
        .iv_counter_nonce(AES_GCM_IV.to_vec())
        .authenticated_encryption_additional_data(AES_GCM_AAD.to_vec())
        .authenticated_encryption_tag(authentication_tag)
        .execute(limits)?;
    let decrypt_response = decrypted
        .outcome()
        .decrypt_response()
        .ok_or("Decrypt did not return a typed successful response")?;
    if decrypt_response.result().status().raw() != 0
        || decrypt_response.unique_identifier() != Some(&identifier)
        || decrypt_response
            .data()
            .is_none_or(|data| !data.with_bytes(|bytes| bytes == AES_GCM_PLAINTEXT))
    {
        return Err("Decrypt response plaintext did not match the original input".into());
    }

    let destroyed = client
        .request()
        .destroy(identifier.clone())
        .execute(limits)?;
    let destroy_response = destroyed
        .outcome()
        .destroy_response()
        .ok_or("Destroy did not return a typed successful response")?;
    if destroy_response.result().status().raw() != 0
        || destroy_response.unique_identifier() != Some(&identifier)
    {
        return Err("Destroy did not return the AES-GCM key identifier".into());
    }
    println!("rust: fluent AES-GCM journey verified");
    Ok(())
}

fn execute_key_pair_signature_journey(
    client: &mut Client,
    limits: &CodecLimits,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut domain_parameters = Structure::new();
    domain_parameters.try_push(Item::new(
        RawTag::new(0x420075)?.try_checked()?,
        Value::enumeration(recommended_curve::P_256),
    )?)?;
    let created = client
        .request()
        .create_key_pair()
        .algorithm(kmipkit::protocol::CryptographicAlgorithm::from_raw(6))
        .length(256)
        .common_attribute(Attribute::<typed::CryptographicDomainParameters>::new(
            domain_parameters,
        )?)
        .private_usage_mask(CryptographicUsageMask::SIGN)
        .public_usage_mask(CryptographicUsageMask::VERIFY)
        .execute(limits)?;
    let response = created
        .outcome()
        .create_key_pair_response()
        .ok_or("Create Key Pair did not return a typed successful response")?;
    if response.result().status().raw() != 0 {
        return Err("Create Key Pair did not succeed".into());
    }
    let private_identifier = response
        .private_key_unique_identifier()
        .cloned()
        .ok_or("Create Key Pair omitted the private key identifier")?;
    let public_identifier = response
        .public_key_unique_identifier()
        .cloned()
        .ok_or("Create Key Pair omitted the public key identifier")?;

    let signed = client
        .request()
        .sign()
        .unique_identifier(private_identifier.clone())
        .cryptographic_parameters(ecdsa_sha256_parameters()?)
        .data(SIGNATURE_PLAINTEXT.to_vec())
        .execute(limits)?;
    let sign_response = signed
        .outcome()
        .sign_response()
        .ok_or("Sign did not return a typed successful response")?;
    if sign_response.result().status().raw() != 0
        || sign_response.unique_identifier() != Some(&private_identifier)
        || sign_response
            .signature_data()
            .is_none_or(|data| !data.with_bytes(|bytes| bytes == SIGNATURE_DATA))
    {
        return Err("Sign response did not preserve the expected signature".into());
    }
    let signature = sign_response
        .signature_data()
        .ok_or("Sign response omitted Signature Data")?
        .with_bytes(<[u8]>::to_vec);

    let verified = client
        .request()
        .signature_verify()
        .unique_identifier(public_identifier.clone())
        .cryptographic_parameters(ecdsa_sha256_parameters()?)
        .data(SIGNATURE_PLAINTEXT.to_vec())
        .signature_data(signature)
        .execute(limits)?;
    let verify_response = verified
        .outcome()
        .signature_verify_response()
        .ok_or("Signature Verify did not return a typed successful response")?;
    if verify_response.result().status().raw() != 0
        || verify_response.unique_identifier() != Some(&public_identifier)
        || verify_response
            .validity_indicator()
            .is_none_or(|value| value.raw() != 1)
    {
        return Err("Signature Verify did not confirm the fixture signature".into());
    }
    println!("rust: fluent Create Key Pair/Sign/Signature Verify journey verified");
    Ok(())
}

fn execute_expert_attribute_journey(
    client: &mut Client,
    limits: &CodecLimits,
) -> Result<(), Box<dyn std::error::Error>> {
    let invalid_item = Item::new(
        RawTag::new(0x420056)?.try_checked()?,
        Value::integer(1),
    )?;
    let invalid = client
        .request()
        .create_symmetric_key()
        .aes_256()
        .usage_mask(CryptographicUsageMask::ENCRYPT | CryptographicUsageMask::DECRYPT)
        .name("fluent-invalid-expert-attributes")
        .attribute_item(invalid_item)
        .execute(limits);
    match invalid {
        Err(error) if error.delivery_state() == Some(RequestDeliveryState::NotSent) => {}
        _ => return Err("wrong-type Object Group was not rejected before transport".into()),
    }

    let created = client
        .request()
        .create_symmetric_key()
        .aes_256()
        .usage_mask(CryptographicUsageMask::ENCRYPT | CryptographicUsageMask::DECRYPT)
        .name("fluent-expert-attributes")
        .attribute(Attribute::<typed::ObjectGroup>::new(
            "fluent-expert-group".to_owned(),
        )?)
        .execute(limits)?;
    let identifier = created
        .outcome()
        .create_response()
        .and_then(|response| response.unique_identifier())
        .cloned()
        .ok_or("expert attribute Create omitted its typed Unique Identifier")?;
    if identifier != UniqueIdentifier::from("fluent-expert-attributes") {
        return Err("expert attribute Create returned an unexpected identifier".into());
    }

    let destroyed = client.request().destroy(identifier.clone()).execute(limits)?;
    let response = destroyed
        .outcome()
        .destroy_response()
        .ok_or("expert attribute Destroy omitted its typed response")?;
    if response.result().status().raw() != 0 || response.unique_identifier() != Some(&identifier) {
        return Err("expert attribute Destroy did not preserve its identifier".into());
    }
    println!("rust: fluent catalogued expert-attribute journey verified");
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port: u16 = std::env::var("KMIPKIT_CONSUMER_TEST_PORT")?.parse()?;
    let fixtures = std::path::PathBuf::from(std::env::var("KMIPKIT_CONSUMER_TEST_FIXTURES")?);
    let registry = kmipkit::client_config::client_extension_registry(
        Vec::new(),
        kmipkit::protocol::extension::defaults(),
    )?;
    let configuration = ClientConfiguration::new(registry);
    let transport = TransportConfig::builder(Endpoint::raw_tls("127.0.0.1", port))
        .client_identity(ClientIdentity::new(
            CertificateInput::from_pem_file(fixtures.join("client-cert.pem"))?,
            PrivateKeyInput::from_pem_file(fixtures.join("client-key.pem"))?,
        ))
        .trust_source(TrustSource::certificate_authorities(vec![
            CertificateInput::from_pem_file(fixtures.join("server-cert.pem"))?,
        ]))
        .tls_server_name("localhost")
        .build()?;
    let mut client = Client::new(configuration, transport)?;
    let limits = CodecLimits::defaults();
    execute_fluent_key_lifecycle(&mut client, &limits)?;
    execute_aes_gcm_journey(&mut client, &limits)?;
    execute_key_pair_signature_journey(&mut client, &limits)?;
    execute_expert_attribute_journey(&mut client, &limits)
}
