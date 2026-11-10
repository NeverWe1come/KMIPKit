//! Explicit public typed operation adapters. No raw outbound request escape hatch.
use crate::driver::{item, structure};
use crate::models::{Case, Result};
use kmipkit::ClientRequest;
use kmipkit::protocol::*;
use kmipkit::ttlv::{Value, ValueView};

pub(crate) fn generic_model_check() -> bool {
    // OASIS KMIP 2.0/2.1 §2: future Enumeration values survive generic navigation.
    let Ok(value) = item(0x42005c, Value::enumeration(0x7fff_fffe)) else { return false };
    let Ok(tree) = structure(0x420079, vec![value]) else { return false };
    tree.view().children().first().is_some_and(|v| v.with_value(|view| matches!(view, ValueView::Enumeration(value) if *value == 0x7fff_fffe)))
}

#[path = "local_cases_generated.rs"]
pub(crate) mod local;

pub(crate) fn attributes(name: &str, algorithm: u32, bits: i32) -> Result<AttributeSet> {
    AttributeSet::try_new(vec![
        item(0x420028, Value::enumeration(algorithm))?,
        item(0x42002a, Value::integer(bits))?,
        item(0x42002c, Value::integer(0x0c))?,
        item(0x420053, Value::structure(structure(0x420053, vec![
            item(0x420055, Value::text_string(name.to_owned()))?, item(0x420054, Value::enumeration(1))?
        ])?))?,
    ]).map_err(|_| "invalid explicit key attributes")
}

pub(crate) fn create(name: &str) -> Result<ClientRequest> {
    Ok(ClientRequest::Create(CreateRequest::new(ObjectType::from_raw(2), attributes(name, 3, 256)?)))
}

fn data() -> OperationData { OperationData::ByteString(SecretBytes::new(vec![0x41; 16])) }
fn parameters() -> Result<kmipkit::ttlv::Structure> {
    CryptographicParametersBuilder::new().with_block_cipher_mode(1).with_padding_method(1).with_cryptographic_algorithm(3)
        .build().map(CryptographicParameters::into_ttlv).map_err(|_| "invalid explicit crypto parameters")
}
fn hash_parameters() -> Result<kmipkit::ttlv::Structure> {
    CryptographicParametersBuilder::new().with_hashing_algorithm(6).build().map(CryptographicParameters::into_ttlv).map_err(|_| "invalid hash parameters")
}

pub(crate) fn request(case: &Case, identifier: Option<UniqueIdentifier>, name: &str) -> Result<ClientRequest> {
    let id_text = identifier.as_ref().and_then(|v| match v { UniqueIdentifier::TextString(text) => Some(text.clone()), _ => None });
    let id = || identifier.clone().ok_or("owned object prerequisite unavailable");
    Ok(match case.operation.as_str() {
        "Discover Versions" => ClientRequest::discover_versions(),
        "Ping" => ClientRequest::ping(),
        "Query" => ClientRequest::Query(QueryRequest::new([QueryFunction::OPERATIONS, QueryFunction::OBJECTS, QueryFunction::CAPABILITIES])),
        "Login" => ClientRequest::Login(LoginRequest::new().with_request_count(1)),
        "Log" => ClientRequest::Log(LogRequest::new(SecretText::new("KMIPKit dedicated interoperability probe".to_owned()))),
        "Interop" => ClientRequest::Interop(InteropRequest::new(1, SecretText::new(name.to_owned()))),
        "Create" => {
            match case.requires["object_type"].as_str() {
                Some("Private Key" | "Public Key") => ClientRequest::CreateKeyPair(CreateKeyPairRequest::new().with_common_attributes(attributes(name, 4, 2048)?)),
                Some("Split Key") => ClientRequest::CreateSplitKey(CreateSplitKeyRequest::new(ObjectType::from_raw(2), 2, 2, SplitKeyMethod::XOR, attributes(name, 3, 256)?)),
                _ => create(name)?,
            }
        }
        "Create Key Pair" => ClientRequest::CreateKeyPair(CreateKeyPairRequest::new().with_common_attributes(attributes(name, 4, 2048)?)),
        "Create Split Key" => ClientRequest::CreateSplitKey(CreateSplitKeyRequest::new(ObjectType::from_raw(2), 2, 2, SplitKeyMethod::XOR, attributes(name, 3, 256)?)),
        "Activate" => ClientRequest::Activate(ActivateRequest::new(Some(id()?))),
        "Archive" => ClientRequest::Archive(ArchiveRequest::new(Some(id()?))),
        "Destroy" => ClientRequest::Destroy(DestroyRequest::new(Some(id()?))),
        "Recover" => ClientRequest::Recover(RecoverRequest::new(Some(id()?))),
        "Revoke" => ClientRequest::Revoke(RevokeRequest::new(RevocationReason::CESSATION_OF_OPERATION).with_unique_identifier(id()?)),
        "Get" => ClientRequest::Get(GetRequest::new().with_unique_identifier(id()?)),
        "Export" => ClientRequest::Export(ExportRequest::new().with_unique_identifier(id()?)),
        "Get Attributes" => {
            let refs = if let Some(attribute) = case.requires["attribute"].as_str() {
                vec![AttributeReference::tag(local::attribute_tag(attribute).ok_or("attribute tag unavailable")?)]
            } else { Vec::new() };
            ClientRequest::GetAttributes(GetAttributesRequest::try_new(id_text, refs).map_err(|_| "invalid attribute references")?)
        }
        "Get Attribute List" => ClientRequest::GetAttributeList(GetAttributeListRequest::new(id_text)),
        "Locate" => ClientRequest::Locate(LocateRequest::new(attributes(name, 3, 256)?).with_maximum_items(1)),
        "Check" => ClientRequest::Check(CheckRequest::new().with_unique_identifier(id()?).with_cryptographic_usage_mask(4)),
        "Obtain Lease" => ClientRequest::ObtainLease(ObtainLeaseRequest::new().with_unique_identifier(id()?)),
        "Get Usage Allocation" => ClientRequest::GetUsageAllocation(GetUsageAllocationRequest::new(1).with_unique_identifier(id()?)),
        "Get Constraints" => ClientRequest::get_constraints(),
        "Set Constraints" => ClientRequest::SetConstraints(SetConstraintsRequest::new(structure(0x42012a, Vec::new())?)),
        "Set Defaults" => ClientRequest::SetDefaults(SetDefaultsRequest::new()),
        "Re-key" => ClientRequest::Rekey(RekeyRequest::new().with_unique_identifier(id()?)),
        "Re-key Key Pair" => ClientRequest::RekeyKeyPair(RekeyKeyPairRequest::new().with_private_key_unique_identifier(id()?)),
        "Add Attribute" => ClientRequest::AddAttribute(AddAttributeRequest::new(id_text, NewAttribute::new(item(0x4200fc, Value::text_string("interop".to_owned()))?))),
        "Set Attribute" => ClientRequest::SetAttribute(SetAttributeRequest::new(id_text, NewAttribute::new(item(0x4200fc, Value::text_string("interop".to_owned()))?))),
        "Modify Attribute" => {
            let attribute = if case.case_id == "test-neg-modify-readonly-attribute" { item(0x42006a, Value::enumeration(2))? } else { item(0x4200fc, Value::text_string("updated".to_owned()))? };
            ClientRequest::ModifyAttribute(ModifyAttributeRequest::new(id_text, None, NewAttribute::new(attribute)))
        }
        "Delete Attribute" => ClientRequest::DeleteAttribute(DeleteAttributeRequest::new(id_text, None, Some(AttributeReference::tag(0x4200fc)))),
        "Adjust Attribute" => ClientRequest::AdjustAttribute(AdjustAttributeRequest::new(id_text, AttributeReference::tag(0x420001), AdjustmentType::from_raw(1), Some(Value::integer(1)))),
        "Hash" => ClientRequest::Hash(HashRequest::new(hash_parameters()?).with_data(data())),
        "RNG Retrieve" => ClientRequest::RngRetrieve(RngRetrieveRequest::new(32)),
        "RNG Seed" => ClientRequest::RngSeed(RngSeedRequest::new(SecretBytes::new(vec![0x41; 32]))),
        "Encrypt" => ClientRequest::Encrypt(EncryptRequest::new(Some(id()?), Some(data())).with_cryptographic_parameters(parameters()?).with_iv_counter_nonce(SecretBytes::new(vec![0; 16]))),
        "Decrypt" => ClientRequest::Decrypt(DecryptRequest::new(Some(id()?), Some(data())).with_cryptographic_parameters(parameters()?).with_iv_counter_nonce(SecretBytes::new(vec![0; 16]))),
        "MAC" => ClientRequest::Mac(MacRequest::new().with_unique_identifier(id()?).with_data(data()).with_cryptographic_parameters(hash_parameters()?)),
        "MAC Verify" => ClientRequest::MacVerify(MacVerifyRequest::new().with_unique_identifier(id()?).with_data(data()).with_mac_data(SecretBytes::new(vec![0;32])).with_cryptographic_parameters(hash_parameters()?)),
        "Sign" => ClientRequest::Sign(SignRequest::new().with_unique_identifier(id()?).with_data(data())),
        "Signature Verify" => ClientRequest::SignatureVerify(SignatureVerifyRequest::new().with_unique_identifier(id()?).with_data(data()).with_signature_data(SecretBytes::new(vec![0;256]))),
        "Derive Key" => ClientRequest::DeriveKey(DeriveKeyRequest::new(ObjectType::from_raw(2), vec![id()?], DerivationMethod::from_raw(2), DerivationParameters::new().with_cryptographic_parameters(hash_parameters()?), attributes(name, 3, 256)?)),
        _ => return Err("operation prerequisites are not represented by this connection contract"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    // KMIPKIT-0084-FR-003; each row's own exact OASIS references identify the contract.
    #[test]
    fn every_synchronous_operation_has_a_public_typed_builder() {
        let catalog: crate::models::Catalog = crate::models::load(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../shared/catalog/kmipkit-1.0-test-catalog.json")).expect("catalog");
        let mut absent = Vec::new();
        for case in &catalog.test_cases {
            if ["Poll", "Process", "Cancel", "Query Asynchronous Requests"].contains(&case.operation.as_str()) || case.execution.mode == "local_model" || case.requires["workflow_type"] == "server_to_client" { continue; }
            if !request(case, Some(UniqueIdentifier::from("owned-fixture")), "kmipkit-fixture").is_ok_and(|request| request.validation_report().is_valid()) { absent.push(&case.case_id); }
        }
        assert!(absent.is_empty(), "catalog rows without typed builders: {absent:?}");
    }
}
