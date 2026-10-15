//! Opt-in integration tests against a local Cosmian KMS 5.28.0 deployment.
//!
//! The tests use `KMIPKit`'s typed Rust client and raw-TLS/mTLS transport for
//! lifecycle, attribute, query, and cryptographic operations. They exercise
//! selected KMIP 2.1 requests; they are not a full conformance or profile test.

use std::error::Error;
use std::path::{Path, PathBuf};

use kmipkit_client::extension_registry::{ClientConfiguration, client_extension_registry};
use kmipkit_client::{
    Client, ClientBatch, ClientBatchItem, ClientError, ClientRequest, ClientResponseView,
};
use kmipkit_protocol::extension;
use kmipkit_protocol::{
    ActivateRequest, AddAttributeRequest, AttributeReference, AttributeSet, CreateKeyPairRequest,
    CreateRequest, CreateSplitKeyRequest, CurrentAttribute, DecryptRequest, DeleteAttributeRequest,
    EncryptRequest, GetAttributeListRequest, GetAttributesRequest, HashRequest, MacRequest,
    MacVerifyRequest, ModifyAttributeRequest, NewAttribute, ObjectType, OperationData,
    QueryFunction, QueryRequest, SecretBytes, SetAttributeRequest, SignRequest,
    SignatureVerifyRequest, SplitKeyMethod, UniqueIdentifier,
};
use kmipkit_transport::{
    CertificateInput, ClientIdentity, Endpoint, PrivateKeyInput, TransportConfig, TrustSource,
};
use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::{Item, RawTag, Structure, Value, ValueView};

const COMMENT: u32 = 0x0042_00FD;
const CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_0028;
const CRYPTOGRAPHIC_LENGTH: u32 = 0x0042_002A;
const CRYPTOGRAPHIC_USAGE_MASK: u32 = 0x0042_002C;
const BATCH_ITEM: u32 = 0x0042_000F;
const OPERATION: u32 = 0x0042_005C;
const RESPONSE_PAYLOAD: u32 = 0x0042_007C;
const RESULT_STATUS: u32 = 0x0042_007F;
const RESULT_REASON: u32 = 0x0042_007E;
const QUERY_PROTECTION_STORAGE_MASKS: u32 = 0x0042_015F;
const AES: u32 = 3;
const RSA: u32 = 4;
const SYMMETRIC_KEY: u32 = 2;
const SUCCESS: u32 = 0;

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.16, Tables 211–213; Protocol Version: §9.16, Table 421.
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
// OASIS KMIP v2.1 §6.1.8, Tables 186–188.
fn cosmian_kms_accepts_kmip_2_1_create_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    let identifier = create_symmetric_key(&mut client)?;
    assert!(!identifier.is_empty(), "Create returns a Unique Identifier");
    Ok(())
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.1, Tables 163–166.
fn cosmian_kms_accepts_kmip_2_1_activate_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    let identifier = create_symmetric_key(&mut client)?;
    activate_key(&mut client, &identifier)
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.9, Tables 189–192.
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
// OASIS KMIP v2.1 §6.1.10, Tables 193–195; Split Key Method: §11.36, Table 470.
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
// OASIS KMIP v2.1 §6.1.2, Tables 167–169.
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
// OASIS KMIP v2.1 §6.1.13, Tables 202–204.
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
// OASIS KMIP v2.1 §6.1.20, Tables 223–225.
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
// OASIS KMIP v2.1 §6.1.21, Tables 226–228.
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
// OASIS KMIP v2.1 §6.1.34, Tables 265–267.
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
// OASIS KMIP v2.1 §6.1.51, Tables 322–324.
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
// OASIS KMIP v2.1 §6.1.40, Tables 282–284; Protection Storage Masks is required by Table 283.
fn cosmian_kms_query_response_without_storage_masks_is_reported_over_mutual_tls()
-> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    let result = client.execute(
        ClientBatch::new(ClientBatchItem::new(ClientRequest::query(
            QueryRequest::new([QueryFunction::OPERATIONS]),
        ))),
        &CodecLimits::defaults(),
    );
    match result {
        Ok(response) => {
            let item = response
                .get(0)
                .ok_or("Cosmian response has no Query result")?;
            let outcome = item.outcome();
            let response_view = outcome.response();
            let query = response_view
                .query()
                .ok_or("KMIPKit did not return a typed Query result")?;
            assert!(query.operations().next().is_some());
            Ok(())
        }
        Err(error) => {
            assert_eq!(
                error.category(),
                kmipkit_client::ClientErrorCategory::Protocol
            );
            let response = error
                .response_ttlv()
                .ok_or("Query parse failure retains the complete response TTLV")?;
            let observation = response.with_ttlv(|root| {
                let batch = root
                    .children()
                    .iter()
                    .find(|item| item.tag().raw() == BATCH_ITEM)?;
                batch.with_value(|value| {
                    let ValueView::Structure(batch_item) = value else {
                        return None;
                    };
                    let status = batch_item.children().iter().find_map(|field| {
                        (field.tag().raw() == RESULT_STATUS)
                            .then(|| {
                                field.with_value(|value| match value {
                                    ValueView::Enumeration(value) => Some(*value),
                                    _ => None,
                                })
                            })
                            .flatten()
                    })?;
                    let payload = batch_item
                        .children()
                        .iter()
                        .find(|field| field.tag().raw() == RESPONSE_PAYLOAD)?;
                    payload.with_value(|value| {
                        let ValueView::Structure(payload) = value else {
                            return None;
                        };
                        let operation_count = payload
                            .children()
                            .iter()
                            .filter(|field| field.tag().raw() == OPERATION)
                            .count();
                        let has_protection_storage_masks = payload
                            .children()
                            .iter()
                            .any(|field| field.tag().raw() == QUERY_PROTECTION_STORAGE_MASKS);
                        Some((status, operation_count, has_protection_storage_masks))
                    })
                })
            });
            let (status, operation_count, has_masks) =
                observation.ok_or("Cosmian Query response has no recognizable result payload")?;
            assert_eq!(status, SUCCESS, "Cosmian reports Query as successful");
            assert!(operation_count > 0, "Cosmian returns supported Operations");
            assert!(
                !has_masks,
                "the observed structured response omits Protection Storage Masks"
            );
            println!(
                "Cosmian Query interoperability limitation: KMIP result is Success with {operation_count} Operations, but the response omits required Protection Storage Masks and KMIPKit rejects the structured payload."
            );
            Ok(())
        }
    }
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.36, Tables 271–272.
fn cosmian_kms_rejects_unsupported_kmip_2_1_ping_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::ping()));
    match client.execute(batch, &CodecLimits::defaults()) {
        Ok(response) => {
            let item = response
                .get(0)
                .ok_or("Cosmian response has no Ping result")?;
            let outcome = item.outcome();
            let view = outcome.response();
            let ping = view
                .ping()
                .ok_or("KMIPKit did not return a typed Ping result")?;
            assert_eq!(ping.result().status().raw(), SUCCESS);
            Ok(())
        }
        Err(error) => {
            assert_eq!(
                error.category(),
                kmipkit_client::ClientErrorCategory::Protocol
            );
            let (operation, status, reason) = response_result_fields(&error)
                .ok_or("Cosmian Ping rejection retains a result in its error response")?;
            assert_eq!(operation, None, "the error response omits Operation");
            assert_eq!(status, Some(1), "Cosmian reports Operation Failed");
            assert_eq!(reason, Some(4), "Cosmian reports Invalid Message for Ping");
            println!(
                "Cosmian rejects KMIP Ping with Operation Failed / Invalid Message; the error response omits the Batch Item Operation field."
            );
            Ok(())
        }
    }
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.17, Tables 214–216.
fn cosmian_kms_accepts_kmip_2_1_encrypt_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    let identifier = create_symmetric_key(&mut client)?;
    activate_key(&mut client, &identifier)?;
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
// OASIS KMIP v2.1 §6.1.11, Tables 196–198.
fn cosmian_kms_accepts_kmip_2_1_decrypt_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    let identifier = create_symmetric_key(&mut client)?;
    activate_key(&mut client, &identifier)?;
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
            Ok(ciphertext.with_bytes(|value| SecretBytes::new(value.to_vec())))
        },
    )?;
    let decrypted = execute_one(
        &mut client,
        ClientRequest::decrypt(
            DecryptRequest::new(
                Some(UniqueIdentifier::TextString(identifier)),
                Some(OperationData::ByteString(encrypted)),
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
            Ok(decrypted.with_bytes(|value| SecretBytes::new(value.to_vec())))
        },
    )?;
    assert!(
        decrypted.with_bytes(|bytes| bytes == plaintext.as_slice()),
        "Decrypt restores the test plaintext"
    );
    Ok(())
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.24, Tables 235–237.
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
// OASIS KMIP v2.1 §6.1.32, Tables 259–261; MAC Data tag: §11.56, Table 487.
fn cosmian_kms_accepts_kmip_2_1_mac_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    let identifier = create_mac_key(&mut client)?;
    execute_one(
        &mut client,
        ClientRequest::mac(
            MacRequest::new()
                .with_unique_identifier(UniqueIdentifier::TextString(identifier))
                .with_data(OperationData::ByteString(SecretBytes::new(
                    b"KMIPKit MAC integration".to_vec(),
                ))),
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
// OASIS KMIP v2.1 §6.1.33, Tables 262–264; MAC Data and Validity Indicator tags: §11.56, Table 487.
fn cosmian_kms_accepts_kmip_2_1_mac_verify_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    let identifier = create_mac_key(&mut client)?;
    let data = b"KMIPKit MAC Verify integration".to_vec();
    let mac = execute_one(
        &mut client,
        ClientRequest::mac(
            MacRequest::new()
                .with_unique_identifier(UniqueIdentifier::TextString(identifier.clone()))
                .with_data(OperationData::ByteString(SecretBytes::new(data.clone()))),
        ),
        "MAC setup for MAC Verify",
        |response| {
            let mac = response
                .mac()
                .and_then(|result| result.mac_data())
                .ok_or("successful MAC setup has no MAC Data")?;
            Ok(mac.with_bytes(|bytes| SecretBytes::new(bytes.to_vec())))
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
                .with_mac_data(mac),
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
// OASIS KMIP v2.1 §6.1.55, Tables 334–336; Signature Data tag: §11.56, Table 487.
fn cosmian_kms_accepts_kmip_2_1_sign_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    let (private_identifier, _) = create_rsa_key_pair(&mut client)?;
    activate_key(&mut client, &private_identifier)?;
    execute_one(
        &mut client,
        ClientRequest::sign(
            SignRequest::new()
                .with_unique_identifier(UniqueIdentifier::TextString(private_identifier))
                .with_data(OperationData::ByteString(SecretBytes::new(
                    b"KMIPKit signing integration".to_vec(),
                ))),
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
// OASIS KMIP v2.1 §6.1.56, Tables 337–339; Signature Data and Validity Indicator tags: §11.56, Table 487.
fn cosmian_kms_accepts_kmip_2_1_signature_verify_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    let (private_identifier, public_identifier) = create_rsa_key_pair(&mut client)?;
    activate_key(&mut client, &private_identifier)?;
    activate_key(&mut client, &public_identifier)?;
    let data = b"KMIPKit signature verification integration".to_vec();
    let signature = execute_one(
        &mut client,
        ClientRequest::sign(
            SignRequest::new()
                .with_unique_identifier(UniqueIdentifier::TextString(private_identifier))
                .with_data(OperationData::ByteString(SecretBytes::new(data.clone()))),
        ),
        "Sign setup for Signature Verify",
        |response| {
            let signature = response
                .sign()
                .and_then(|result| result.signature_data())
                .ok_or("successful Sign setup has no Signature Data")?;
            Ok(signature.with_bytes(|bytes| SecretBytes::new(bytes.to_vec())))
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
                .with_signature_data(signature),
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

fn response_result_fields(error: &ClientError) -> Option<(Option<u32>, Option<u32>, Option<u32>)> {
    error.response_ttlv()?.with_ttlv(|root| {
        let batch_item = root
            .children()
            .iter()
            .find(|item| item.tag().raw() == BATCH_ITEM)?;
        batch_item.with_value(|value| {
            let ValueView::Structure(batch_item) = value else {
                return None;
            };
            let read_enumeration = |tag| {
                batch_item.children().iter().find_map(|field| {
                    (field.tag().raw() == tag)
                        .then(|| {
                            field.with_value(|value| match value {
                                ValueView::Enumeration(value) => Some(*value),
                                _ => None,
                            })
                        })
                        .flatten()
                })
            };
            Some((
                read_enumeration(OPERATION),
                read_enumeration(RESULT_STATUS),
                read_enumeration(RESULT_REASON),
            ))
        })
    })
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
        kmip_item(CRYPTOGRAPHIC_USAGE_MASK, Value::integer(384))?, // MAC Generate + MAC Verify, §12.1 Table 495.
    ])?;
    let identifier = execute_one(
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
    )?;
    activate_key(client, &identifier)?;
    Ok(identifier)
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

fn activate_key(client: &mut Client, identifier: &str) -> Result<(), Box<dyn Error>> {
    execute_one(
        client,
        ClientRequest::activate(ActivateRequest::new(Some(UniqueIdentifier::TextString(
            identifier.to_owned(),
        )))),
        "Activate test key",
        |response| {
            let returned_identifier = response
                .activate()
                .and_then(|result| result.unique_identifier())
                .ok_or("successful Activate response has no Unique Identifier")?;
            text_identifier(returned_identifier)?;
            Ok(())
        },
    )
}

fn rsa_key_pair_request() -> Result<CreateKeyPairRequest, Box<dyn Error>> {
    let common_attributes = AttributeSet::try_new([
        kmip_item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(RSA))?,
        kmip_item(CRYPTOGRAPHIC_LENGTH, Value::integer(2048))?,
    ])?;
    // OASIS KMIP v2.1 §12.1, Table 495: Sign is 0x01; Verify is 0x02.
    let private_attributes =
        AttributeSet::try_new([kmip_item(CRYPTOGRAPHIC_USAGE_MASK, Value::integer(1))?])?;
    let public_attributes =
        AttributeSet::try_new([kmip_item(CRYPTOGRAPHIC_USAGE_MASK, Value::integer(2))?])?;
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
