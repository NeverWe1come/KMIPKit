//! Opt-in integration tests against a local Cosmian KMS 5.28.0 deployment.
//!
//! The tests use `KMIPKit`'s typed Rust client and raw-TLS/mTLS transport for
//! Discover Versions, Create, Create Key Pair, Create Split Key, and six
//! supported attribute operations. They exercise selected KMIP 2.1 requests;
//! they are not a full conformance or profile test.

use std::error::Error;
use std::path::{Path, PathBuf};

use kmipkit_client::extension_registry::{ClientConfiguration, client_extension_registry};
use kmipkit_client::{Client, ClientBatch, ClientBatchItem, ClientRequest, ClientResponseView};
use kmipkit_protocol::extension;
use kmipkit_protocol::{
    AddAttributeRequest, AttributeReference, AttributeSet, CreateKeyPairRequest, CreateRequest,
    CreateSplitKeyRequest, CurrentAttribute, DecryptRequest, DeleteAttributeRequest,
    EncryptRequest, GetAttributeListRequest, GetAttributesRequest, HashRequest, MacRequest,
    MacVerifyRequest, ModifyAttributeRequest, NewAttribute, ObjectType, OperationData,
    QueryFunction, QueryRequest, SetAttributeRequest, SignRequest, SignatureVerifyRequest,
    SplitKeyMethod, UniqueIdentifier,
};
use kmipkit_transport::{
    CertificateInput, ClientIdentity, Endpoint, PrivateKeyInput, TransportConfig, TrustSource,
};
use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::{Item, RawTag, Structure, Value};

const COMMENT: u32 = 0x0042_00FD;
const CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_0028;
const CRYPTOGRAPHIC_LENGTH: u32 = 0x0042_002A;
const CRYPTOGRAPHIC_USAGE_MASK: u32 = 0x0042_002C;
const AES: u32 = 3;
const RSA: u32 = 4;
const SYMMETRIC_KEY: u32 = 2;
const SUCCESS: u32 = 0;

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
fn cosmian_kms_accepts_kmip_2_1_discover_versions_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    execute_one(
        &mut client,
        ClientRequest::discover_versions(),
        "Discover Versions",
        |response| {
            let versions = response
                .discover_versions()
                .and_then(|result| result.supported_versions())
                .ok_or("successful Discover Versions response has no version list")?;
            assert!(
                versions.contains(&kmipkit_protocol::ProtocolVersion::from_raw(2, 1)),
                "Cosmian reports the KMIP 2.1 version offered by KMIPKit"
            );
            Ok(())
        },
    )
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
fn cosmian_kms_accepts_kmip_2_1_create_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    let identifier = create_symmetric_key(&mut client)?;
    assert!(!identifier.is_empty(), "Create returns a Unique Identifier");
    Ok(())
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
fn cosmian_kms_accepts_kmip_2_1_create_key_pair_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    execute_one(
        &mut client,
        ClientRequest::CreateKeyPair(rsa_key_pair_request()?),
        "Create Key Pair",
        |response| {
            let key_pair = response
                .create_key_pair()
                .ok_or("KMIPKit did not return a typed Create Key Pair result")?;
            let private_identifier = key_pair
                .private_key_unique_identifier()
                .ok_or("successful Create Key Pair response has no private-key identifier")?;
            let public_identifier = key_pair
                .public_key_unique_identifier()
                .ok_or("successful Create Key Pair response has no public-key identifier")?;
            assert!(
                !matches!(private_identifier, UniqueIdentifier::TextString(value) if value.is_empty()),
                "private-key identifier is nonempty"
            );
            assert!(
                !matches!(public_identifier, UniqueIdentifier::TextString(value) if value.is_empty()),
                "public-key identifier is nonempty"
            );
            Ok(())
        },
    )
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
fn cosmian_kms_accepts_kmip_2_1_create_split_key_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    let source_identifier = create_symmetric_key(&mut client)?;
    execute_one(
        &mut client,
        ClientRequest::CreateSplitKey(
            CreateSplitKeyRequest::new(
                ObjectType::from_raw(SYMMETRIC_KEY),
                2,
                2,
                SplitKeyMethod::XOR,
                AttributeSet::new(),
            )
            .with_unique_identifier(UniqueIdentifier::TextString(source_identifier)),
        ),
        "Create Split Key",
        |response| {
            let identifiers = response
                .create_split_key()
                .ok_or("KMIPKit did not return a typed Create Split Key result")?
                .unique_identifiers();
            assert_eq!(
                identifiers.len(),
                2,
                "two-of-two split returns two identifiers"
            );
            Ok(())
        },
    )
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
fn cosmian_kms_accepts_kmip_2_1_add_attribute_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    let identifier = create_rsa_public_key(&mut client)?;
    add_comment(
        &mut client,
        &identifier,
        "KMIPKit integration Add Attribute",
    )
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
fn cosmian_kms_accepts_kmip_2_1_delete_attribute_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    let identifier = create_rsa_public_key(&mut client)?;
    add_comment(
        &mut client,
        &identifier,
        "KMIPKit integration Delete Attribute",
    )?;
    execute_one(
        &mut client,
        ClientRequest::delete_attribute(DeleteAttributeRequest::new(
            Some(identifier),
            None,
            Some(AttributeReference::tag(COMMENT)),
        )),
        "Delete Attribute",
        |response| {
            let returned_identifier = response
                .delete_attribute()
                .and_then(|result| result.unique_identifier())
                .ok_or("successful Delete Attribute response has no Unique Identifier")?;
            assert_ne!(returned_identifier, "");
            Ok(())
        },
    )
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
fn cosmian_kms_accepts_kmip_2_1_get_attributes_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    let identifier = create_rsa_public_key(&mut client)?;
    add_comment(
        &mut client,
        &identifier,
        "KMIPKit integration Get Attributes",
    )?;
    execute_one(
        &mut client,
        ClientRequest::get_attributes(GetAttributesRequest::try_new(
            Some(identifier),
            [AttributeReference::tag(COMMENT)],
        )?),
        "Get Attributes",
        |response| {
            let attributes = response
                .get_attributes()
                .and_then(|result| result.attributes())
                .ok_or("successful Get Attributes response has no Attributes set")?;
            assert!(
                attributes
                    .as_items()
                    .iter()
                    .any(|attribute| attribute.tag().raw() == COMMENT),
                "Get Attributes returns the requested Comment"
            );
            Ok(())
        },
    )
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
fn cosmian_kms_accepts_kmip_2_1_get_attribute_list_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    let identifier = create_rsa_public_key(&mut client)?;
    execute_one(
        &mut client,
        ClientRequest::get_attribute_list(GetAttributeListRequest::new(Some(identifier))),
        "Get Attribute List",
        |response| {
            let attribute_references = response
                .get_attribute_list()
                .and_then(|result| result.attribute_references())
                .ok_or("successful Get Attribute List response has no Attribute References")?;
            assert!(
                !attribute_references.is_empty(),
                "Get Attribute List returns at least one Attribute Reference"
            );
            Ok(())
        },
    )
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
fn cosmian_kms_accepts_kmip_2_1_modify_attribute_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    let identifier = create_rsa_public_key(&mut client)?;
    add_comment(
        &mut client,
        &identifier,
        "KMIPKit integration Modify Attribute",
    )?;
    execute_one(
        &mut client,
        ClientRequest::modify_attribute(ModifyAttributeRequest::new(
            Some(identifier),
            Some(CurrentAttribute::new(comment_item(
                "KMIPKit integration Modify Attribute",
            )?)),
            NewAttribute::new(comment_item("KMIPKit integration Modified Attribute")?),
        )),
        "Modify Attribute",
        |response| {
            let returned_identifier = response
                .modify_attribute()
                .and_then(|result| result.unique_identifier())
                .ok_or("successful Modify Attribute response has no Unique Identifier")?;
            assert_ne!(returned_identifier, "");
            Ok(())
        },
    )
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
fn cosmian_kms_accepts_kmip_2_1_set_attribute_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    let identifier = create_rsa_public_key(&mut client)?;
    add_comment(
        &mut client,
        &identifier,
        "KMIPKit integration Set Attribute",
    )?;
    execute_one(
        &mut client,
        ClientRequest::set_attribute(SetAttributeRequest::new(
            Some(identifier),
            NewAttribute::new(comment_item(
                "KMIPKit integration Set Attribute replacement",
            )?),
        )),
        "Set Attribute",
        |response| {
            let returned_identifier = response
                .set_attribute()
                .and_then(|result| result.unique_identifier())
                .ok_or("successful Set Attribute response has no Unique Identifier")?;
            assert_ne!(returned_identifier, "");
            Ok(())
        },
    )
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
fn cosmian_kms_accepts_kmip_2_1_query_operations_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    execute_one(
        &mut client,
        ClientRequest::query(QueryRequest::new([QueryFunction::OPERATIONS])),
        "Query Operations",
        |response| {
            let query = response
                .query()
                .ok_or("KMIPKit did not return a typed Query result")?;
            assert!(
                !query.operations().collect::<Vec<_>>().is_empty(),
                "Cosmian Query Operations reports at least one operation"
            );
            Ok(())
        },
    )
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
fn cosmian_kms_reports_kmip_2_1_ping_result_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::ping()));
    let response = client.execute(batch, &CodecLimits::defaults())?;
    assert_eq!(response.len(), 1, "Cosmian returns one Ping result");
    let item = response
        .get(0)
        .ok_or("Cosmian response has no Ping result")?;
    let outcome = item.outcome();
    let response_view = outcome.response();
    let ping = response_view
        .ping()
        .ok_or("KMIPKit did not return a typed Ping result")?;
    let status = ping.result().status();
    let reason = ping.result().reason();
    println!(
        "Cosmian Ping result: status={} ({}), reason={}",
        status.raw(),
        status.known_name().unwrap_or("unknown"),
        reason.map_or("none", |value| value.known_name().unwrap_or("unknown")),
    );
    assert!(
        status.raw() == SUCCESS || (status.raw() == 1 && reason.is_some()),
        "Ping is either successful or rejected with a server-reported KMIP failure"
    );
    Ok(())
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
fn cosmian_kms_accepts_kmip_2_1_encrypt_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    let identifier = create_symmetric_key(&mut client)?;
    let request = EncryptRequest::new(
        Some(UniqueIdentifier::TextString(identifier)),
        Some(OperationData::ByteString(
            kmipkit_protocol::SecretBytes::new(b"KMIPKit Cosmian integration plaintext".to_vec()),
        )),
    )
    .with_cryptographic_parameters(cipher_parameters()?)
    .with_iv_counter_nonce(kmipkit_protocol::SecretBytes::new(vec![0x31; 16]));
    execute_one(
        &mut client,
        ClientRequest::encrypt(request),
        "Encrypt",
        |response| {
            let encrypted = response
                .encrypt()
                .ok_or("KMIPKit did not return a typed Encrypt result")?;
            assert!(
                encrypted.data().is_some(),
                "successful Encrypt response returns ciphertext Data"
            );
            Ok(())
        },
    )
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
fn cosmian_kms_accepts_kmip_2_1_decrypt_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    let identifier = create_symmetric_key(&mut client)?;
    let plaintext = b"KMIPKit Cosmian decrypt integration".to_vec();
    let encrypted = execute_one(
        &mut client,
        ClientRequest::encrypt(
            EncryptRequest::new(
                Some(UniqueIdentifier::TextString(identifier.clone())),
                Some(OperationData::ByteString(
                    kmipkit_protocol::SecretBytes::new(plaintext.clone()),
                )),
            )
            .with_cryptographic_parameters(cipher_parameters()?)
            .with_iv_counter_nonce(kmipkit_protocol::SecretBytes::new(vec![0x32; 16])),
        ),
        "Encrypt setup for Decrypt",
        |response| {
            let ciphertext = response
                .encrypt()
                .and_then(|result| result.data())
                .ok_or("successful Encrypt setup has no ciphertext Data")?;
            let mut bytes = Vec::new();
            ciphertext.with_bytes(|value| bytes.extend_from_slice(value));
            Ok(bytes)
        },
    )?;
    let decrypted = execute_one(
        &mut client,
        ClientRequest::decrypt(
            DecryptRequest::new(
                Some(UniqueIdentifier::TextString(identifier)),
                Some(OperationData::ByteString(
                    kmipkit_protocol::SecretBytes::new(encrypted),
                )),
            )
            .with_cryptographic_parameters(cipher_parameters()?)
            .with_iv_counter_nonce(kmipkit_protocol::SecretBytes::new(vec![0x32; 16])),
        ),
        "Decrypt",
        |response| {
            let decrypted = response
                .decrypt()
                .and_then(|result| result.data())
                .ok_or("successful Decrypt response has no plaintext Data")?;
            let mut bytes = Vec::new();
            decrypted.with_bytes(|value| bytes.extend_from_slice(value));
            Ok(bytes)
        },
    )?;
    assert_eq!(decrypted, plaintext, "Decrypt restores the test plaintext");
    Ok(())
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
fn cosmian_kms_accepts_kmip_2_1_hash_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    let parameters = structure([kmip_item(0x0042_0038, Value::enumeration(6))?])?;
    execute_one(
        &mut client,
        ClientRequest::hash(
            HashRequest::new(parameters).with_data(OperationData::ByteString(
                kmipkit_protocol::SecretBytes::new(b"KMIPKit Cosmian hash integration".to_vec()),
            )),
        ),
        "Hash",
        |response| {
            let digest = response
                .hash()
                .and_then(|result| result.data())
                .ok_or("successful Hash response has no digest Data")?;
            digest.with_bytes(|bytes| {
                assert!(!bytes.is_empty(), "the returned digest is nonempty");
            });
            Ok(())
        },
    )
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
fn cosmian_kms_accepts_kmip_2_1_mac_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    let identifier = create_mac_key(&mut client)?;
    execute_one(
        &mut client,
        ClientRequest::mac(
            MacRequest::new()
                .with_unique_identifier(UniqueIdentifier::TextString(identifier))
                .with_data(OperationData::ByteString(
                    kmipkit_protocol::SecretBytes::new(b"KMIPKit MAC integration".to_vec()),
                )),
        ),
        "MAC",
        |response| {
            let mac = response
                .mac()
                .and_then(|result| result.mac_data())
                .ok_or("successful MAC response has no MAC Data")?;
            mac.with_bytes(|bytes| assert!(!bytes.is_empty(), "the returned MAC is nonempty"));
            Ok(())
        },
    )
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
fn cosmian_kms_accepts_kmip_2_1_mac_verify_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    let identifier = create_mac_key(&mut client)?;
    let data = b"KMIPKit MAC Verify integration".to_vec();
    let mac = execute_one(
        &mut client,
        ClientRequest::mac(
            MacRequest::new()
                .with_unique_identifier(UniqueIdentifier::TextString(identifier.clone()))
                .with_data(OperationData::ByteString(
                    kmipkit_protocol::SecretBytes::new(data.clone()),
                )),
        ),
        "MAC setup for MAC Verify",
        |response| {
            let mac = response
                .mac()
                .and_then(|result| result.mac_data())
                .ok_or("successful MAC setup has no MAC Data")?;
            let mut bytes = Vec::new();
            mac.with_bytes(|value| bytes.extend_from_slice(value));
            Ok(bytes)
        },
    )?;
    execute_one(
        &mut client,
        ClientRequest::mac_verify(
            MacVerifyRequest::new()
                .with_unique_identifier(UniqueIdentifier::TextString(identifier))
                .with_data(OperationData::ByteString(
                    kmipkit_protocol::SecretBytes::new(data),
                ))
                .with_mac_data(kmipkit_protocol::SecretBytes::new(mac)),
        ),
        "MAC Verify",
        |response| {
            let result = response
                .mac_verify()
                .ok_or("KMIPKit did not return a typed MAC Verify result")?;
            assert!(
                result.validity_indicator().is_some(),
                "successful single-part MAC Verify returns a Validity Indicator"
            );
            Ok(())
        },
    )
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
fn cosmian_kms_accepts_kmip_2_1_sign_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    let (private_identifier, _) = create_rsa_key_pair(&mut client)?;
    execute_one(
        &mut client,
        ClientRequest::sign(
            SignRequest::new()
                .with_unique_identifier(UniqueIdentifier::TextString(private_identifier))
                .with_data(OperationData::ByteString(
                    kmipkit_protocol::SecretBytes::new(b"KMIPKit signing integration".to_vec()),
                )),
        ),
        "Sign",
        |response| {
            let signature = response
                .sign()
                .and_then(|result| result.signature_data())
                .ok_or("successful Sign response has no Signature Data")?;
            signature.with_bytes(|bytes| {
                assert!(!bytes.is_empty(), "the returned signature is nonempty");
            });
            Ok(())
        },
    )
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
fn cosmian_kms_accepts_kmip_2_1_signature_verify_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    let (private_identifier, public_identifier) = create_rsa_key_pair(&mut client)?;
    let data = b"KMIPKit signature verification integration".to_vec();
    let signature = execute_one(
        &mut client,
        ClientRequest::sign(
            SignRequest::new()
                .with_unique_identifier(UniqueIdentifier::TextString(private_identifier))
                .with_data(OperationData::ByteString(
                    kmipkit_protocol::SecretBytes::new(data.clone()),
                )),
        ),
        "Sign setup for Signature Verify",
        |response| {
            let signature = response
                .sign()
                .and_then(|result| result.signature_data())
                .ok_or("successful Sign setup has no Signature Data")?;
            let mut bytes = Vec::new();
            signature.with_bytes(|value| bytes.extend_from_slice(value));
            Ok(bytes)
        },
    )?;
    execute_one(
        &mut client,
        ClientRequest::signature_verify(
            SignatureVerifyRequest::new()
                .with_unique_identifier(UniqueIdentifier::TextString(public_identifier))
                .with_data(OperationData::ByteString(
                    kmipkit_protocol::SecretBytes::new(data),
                ))
                .with_signature_data(kmipkit_protocol::SecretBytes::new(signature)),
        ),
        "Signature Verify",
        |response| {
            let result = response
                .signature_verify()
                .ok_or("KMIPKit did not return a typed Signature Verify result")?;
            assert!(
                result.validity_indicator().is_some(),
                "successful single-part Signature Verify returns a Validity Indicator"
            );
            Ok(())
        },
    )
}

fn client() -> Result<Client, Box<dyn Error>> {
    let certificate_directory = certificate_directory();
    let transport_configuration = TransportConfig::builder(Endpoint::raw_tls("127.0.0.1", 5696))
        .tls_server_name("localhost")
        .client_identity(ClientIdentity::new(
            CertificateInput::from_pem_file(certificate_directory.join("client/client.crt"))?,
            PrivateKeyInput::from_pem_file(certificate_directory.join("client/client.key"))?,
        ))
        .trust_source(TrustSource::certificate_authorities(vec![
            CertificateInput::from_pem_file(certificate_directory.join("client/ca.crt"))?,
        ]))
        .build()?;
    let registry = client_extension_registry(Vec::new(), extension::defaults())?;
    Ok(Client::new(
        ClientConfiguration::new(registry),
        transport_configuration,
    )?)
}

fn execute_one<T>(
    client: &mut Client,
    request: ClientRequest,
    operation: &str,
    validate_response: impl FnOnce(ClientResponseView<'_>) -> Result<T, Box<dyn Error>>,
) -> Result<T, Box<dyn Error>> {
    let response = client.execute(
        ClientBatch::new(ClientBatchItem::new(request)),
        &CodecLimits::defaults(),
    )?;
    assert_eq!(response.len(), 1, "Cosmian returns one {operation} result");
    let item = response
        .get(0)
        .ok_or_else(|| format!("Cosmian response has no {operation} result"))?;
    let outcome = item.outcome();
    let status = outcome.result().status();
    let reason = outcome
        .result()
        .reason()
        .map(kmipkit_protocol::ResultReason::raw);
    assert_eq!(
        status.raw(),
        SUCCESS,
        "Cosmian {operation} failed: Result Status {} ({}) and Result Reason {:?}",
        status.raw(),
        status.known_name().unwrap_or("unknown"),
        reason,
    );
    validate_response(outcome.response())
}

fn create_symmetric_key(client: &mut Client) -> Result<String, Box<dyn Error>> {
    let attributes = AttributeSet::try_new([
        kmip_item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(AES))?,
        kmip_item(CRYPTOGRAPHIC_LENGTH, Value::integer(256))?,
        kmip_item(CRYPTOGRAPHIC_USAGE_MASK, Value::integer(12))?,
    ])?;
    execute_one(
        client,
        ClientRequest::Create(CreateRequest::new(
            ObjectType::from_raw(SYMMETRIC_KEY),
            attributes,
        )),
        "Create",
        |response| {
            let identifier = response
                .create()
                .and_then(|result| result.unique_identifier())
                .ok_or("successful Create response has no Unique Identifier")?;
            text_identifier(identifier)
        },
    )
}

fn create_mac_key(client: &mut Client) -> Result<String, Box<dyn Error>> {
    let attributes = AttributeSet::try_new([
        kmip_item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(9))?,
        kmip_item(CRYPTOGRAPHIC_LENGTH, Value::integer(256))?,
        kmip_item(CRYPTOGRAPHIC_USAGE_MASK, Value::integer(48))?,
    ])?;
    execute_one(
        client,
        ClientRequest::Create(CreateRequest::new(
            ObjectType::from_raw(SYMMETRIC_KEY),
            attributes,
        )),
        "Create HMAC key",
        |response| {
            let identifier = response
                .create()
                .and_then(|result| result.unique_identifier())
                .ok_or("successful Create HMAC key response has no Unique Identifier")?;
            text_identifier(identifier)
        },
    )
}

fn create_rsa_key_pair(client: &mut Client) -> Result<(String, String), Box<dyn Error>> {
    execute_one(
        client,
        ClientRequest::CreateKeyPair(rsa_key_pair_request()?),
        "Create Key Pair test setup",
        |response| {
            let key_pair = response
                .create_key_pair()
                .ok_or("successful Create Key Pair setup has no typed result")?;
            let private_identifier = key_pair
                .private_key_unique_identifier()
                .ok_or("successful Create Key Pair setup has no private-key identifier")?;
            let public_identifier = key_pair
                .public_key_unique_identifier()
                .ok_or("successful Create Key Pair setup has no public-key identifier")?;
            Ok((
                text_identifier(private_identifier)?,
                text_identifier(public_identifier)?,
            ))
        },
    )
}

fn rsa_key_pair_request() -> Result<CreateKeyPairRequest, Box<dyn Error>> {
    let common_attributes = AttributeSet::try_new([
        kmip_item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(RSA))?,
        kmip_item(CRYPTOGRAPHIC_LENGTH, Value::integer(2048))?,
    ])?;
    let private_attributes =
        AttributeSet::try_new([kmip_item(CRYPTOGRAPHIC_USAGE_MASK, Value::integer(24))?])?;
    let public_attributes =
        AttributeSet::try_new([kmip_item(CRYPTOGRAPHIC_USAGE_MASK, Value::integer(36))?])?;
    Ok(CreateKeyPairRequest::new()
        .with_common_attributes(common_attributes)
        .with_private_key_attributes(private_attributes)
        .with_public_key_attributes(public_attributes))
}

fn create_rsa_public_key(client: &mut Client) -> Result<String, Box<dyn Error>> {
    execute_one(
        client,
        ClientRequest::CreateKeyPair(rsa_key_pair_request()?),
        "Create Key Pair test setup",
        |response| {
            let identifier = response
                .create_key_pair()
                .and_then(|result| result.public_key_unique_identifier())
                .ok_or("successful Create Key Pair setup has no public-key identifier")?;
            text_identifier(identifier)
        },
    )
}

fn add_comment(client: &mut Client, identifier: &str, value: &str) -> Result<(), Box<dyn Error>> {
    execute_one(
        client,
        ClientRequest::add_attribute(AddAttributeRequest::new(
            Some(identifier.to_owned()),
            NewAttribute::new(comment_item(value)?),
        )),
        "Add Attribute",
        |response| {
            let returned_identifier = response
                .add_attribute()
                .and_then(|result| result.unique_identifier())
                .ok_or("successful Add Attribute response has no Unique Identifier")?;
            assert_ne!(returned_identifier, "");
            Ok(())
        },
    )
}

fn text_identifier(identifier: &UniqueIdentifier) -> Result<String, Box<dyn Error>> {
    match identifier {
        UniqueIdentifier::TextString(value) if !value.is_empty() => Ok(value.clone()),
        _ => Err("server-generated Unique Identifier is not a nonempty Text String".into()),
    }
}

fn comment_item(value: &str) -> Result<Item, Box<dyn Error>> {
    kmip_item(COMMENT, Value::text_string(value.to_owned()))
}

fn kmip_item(raw_tag: u32, value: Value) -> Result<Item, Box<dyn Error>> {
    let tag = RawTag::new(raw_tag)?.try_checked()?;
    Ok(Item::new(tag, value)?)
}

fn structure(items: impl IntoIterator<Item = Item>) -> Result<Structure, Box<dyn Error>> {
    let mut structure = Structure::new();
    for item in items {
        structure.try_push(item)?;
    }
    Ok(structure)
}

fn cipher_parameters() -> Result<Structure, Box<dyn Error>> {
    structure([
        kmip_item(0x0042_0011, Value::enumeration(1))?, // CBC
        kmip_item(0x0042_005F, Value::enumeration(3))?, // PKCS5 padding
    ])
}

fn certificate_directory() -> PathBuf {
    std::env::var_os("KMIPKIT_COSMIAN_CERT_DIR").map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.local/cosmian-kms/certs"),
        PathBuf::from,
    )
}
