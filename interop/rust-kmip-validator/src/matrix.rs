//! Execute only named public routes; pending recipes never produce call evidence.
use crate::models::{Result, hash};
use kmipkit::ttlv::{Item, Value, ValueView, codec::DecodeErrorKind};
use serde_json::{Value as Json, json};
use std::{
    collections::HashSet,
    io::{BufRead, BufReader},
    path::Path,
};

#[path = "route_calls_generated.rs"]
mod generated;

const KEY_BLOCK_TAG: u32 = 0x0042_0040;
const KEY_WRAPPING_CONSTRUCTOR_ROUTES: &[&str] = &[
    "kmipkit_protocol::EncryptionKeyInformation::try_new",
    "kmipkit_protocol::EncryptionKeyInformation::with_cryptographic_parameters",
    "kmipkit_protocol::EncryptionKeyInformation::with_typed_cryptographic_parameters",
    "kmipkit_protocol::KeyWrappingData::try_new",
    "kmipkit_protocol::KeyWrappingData::with_encoding_option",
    "kmipkit_protocol::KeyWrappingData::with_encryption_key_information",
    "kmipkit_protocol::KeyWrappingData::with_iv_counter_nonce",
    "kmipkit_protocol::KeyWrappingData::with_mac_signature",
    "kmipkit_protocol::KeyWrappingData::with_mac_signature_key_information",
    "kmipkit_protocol::MacSignatureKeyInformation::try_new",
    "kmipkit_protocol::MacSignatureKeyInformation::with_cryptographic_parameters",
    "kmipkit_protocol::MacSignatureKeyInformation::with_typed_cryptographic_parameters",
];

fn text<'a>(value: &'a Json, field: &str) -> Result<&'a str> {
    value[field].as_str().ok_or("matrix field missing")
}

fn operation_value(name: &str) -> Option<u32> {
    use kmipkit::ClientOperation as Operation;

    Some(match name {
        "Activate" => Operation::Activate.raw(),
        "Add Attribute" => Operation::AddAttribute.raw(),
        "Adjust Attribute" => Operation::AdjustAttribute.raw(),
        "Archive" => Operation::Archive.raw(),
        "Cancel" => Operation::Cancel.raw(),
        "Certify" => Operation::Certify.raw(),
        "Check" => Operation::Check.raw(),
        "Create" => Operation::Create.raw(),
        "Create Key Pair" => Operation::CreateKeyPair.raw(),
        "Create Split Key" => Operation::CreateSplitKey.raw(),
        "Decrypt" => Operation::Decrypt.raw(),
        "Delegated Login" => Operation::DelegatedLogin.raw(),
        "Delete Attribute" => Operation::DeleteAttribute.raw(),
        "Derive Key" => Operation::DeriveKey.raw(),
        "Destroy" => Operation::Destroy.raw(),
        "Discover Versions" => Operation::DiscoverVersions.raw(),
        "Encrypt" => Operation::Encrypt.raw(),
        "Export" => Operation::Export.raw(),
        "Get" => Operation::Get.raw(),
        "Get Attribute List" => Operation::GetAttributeList.raw(),
        "Get Attributes" => Operation::GetAttributes.raw(),
        "Get Constraints" => Operation::GetConstraints.raw(),
        "Get Usage Allocation" => Operation::GetUsageAllocation.raw(),
        "Hash" => Operation::Hash.raw(),
        "Import" => Operation::Import.raw(),
        "Interop" => Operation::Interop.raw(),
        "Join Split Key" => Operation::JoinSplitKey.raw(),
        "Locate" => Operation::Locate.raw(),
        "Log" => Operation::Log.raw(),
        "Login" => Operation::Login.raw(),
        "Logout" => Operation::Logout.raw(),
        "MAC" => Operation::Mac.raw(),
        "MAC Verify" => Operation::MacVerify.raw(),
        "Modify Attribute" => Operation::ModifyAttribute.raw(),
        "Notify" => 0x1b,
        "Obtain Lease" => Operation::ObtainLease.raw(),
        "PKCS#11" => Operation::Pkcs11.raw(),
        "Ping" => Operation::Ping.raw(),
        "Poll" => Operation::Poll.raw(),
        "Process" => Operation::Process.raw(),
        "Put" => 0x1c,
        "Query" => Operation::Query.raw(),
        "Query Asynchronous Requests" => Operation::QueryAsyncRequests.raw(),
        "RNG Retrieve" => Operation::RngRetrieve.raw(),
        "RNG Seed" => Operation::RngSeed.raw(),
        "Re-Provision" => Operation::ReProvision.raw(),
        "Re-certify" => Operation::Recertify.raw(),
        "Re-key" => Operation::Rekey.raw(),
        "Re-key Key Pair" => Operation::RekeyKeyPair.raw(),
        "Recover" => Operation::Recover.raw(),
        "Register" => Operation::Register.raw(),
        "Revoke" => Operation::Revoke.raw(),
        "Set Attribute" => Operation::SetAttribute.raw(),
        "Set Constraints" => Operation::SetConstraints.raw(),
        "Set Defaults" => Operation::SetDefaults.raw(),
        "Set Endpoint Role" => Operation::SetEndpointRole.raw(),
        "Sign" => Operation::Sign.raw(),
        "Signature Verify" => Operation::SignatureVerify.raw(),
        "Validate" => Operation::Validate.raw(),
        _ => return None,
    })
}

fn unsigned_wire_value(value: &str) -> Option<u32> {
    let digits = value
        .trim()
        .trim_start_matches("0x")
        .replace(['X', 'x'], "0");
    u32::from_str_radix(&digits, 16).ok()
}

fn wire_range_start(value: &str) -> Option<u32> {
    value
        .split(['-', '–', '—'])
        .next()
        .and_then(unsigned_wire_value)
}

fn target_wire_value(target: &Json) -> Result<u32> {
    let range = text(target, "wire_value")?;
    wire_range_start(range).ok_or("target range needs a concrete fixture recipe")
}

fn vendor_attribute_value() -> Result<Value> {
    let mut fields = kmipkit::ttlv::Structure::new();
    for field in [
        crate::driver::item(
            0x0042_009d,
            Value::text_string("KMIPKit.TestVendor_1".to_owned()),
        )?,
        crate::driver::item(
            0x0042_000a,
            Value::text_string("Opaque.Attribute".to_owned()),
        )?,
        crate::driver::item(0x0042_000b, Value::byte_string(vec![0, 0x80, 0xff]))?,
    ] {
        fields
            .try_push(field)
            .map_err(|_| "vendor attribute fixture rejected")?;
    }
    Ok(Value::structure(fields))
}

fn concrete_fixture_name(target_id: &str, kind: &str, name: &str) -> String {
    if target_id.contains("ANY-ATTRIBUTE-IN-4-OBJECT-ATTRIBUTES") {
        // Use one concrete assigned §4 attribute for the inventory's wildcard member.
        return "Cryptographic Algorithm".to_owned();
    }
    if kind == "message_field" {
        return name.split(',').next().unwrap_or(name).trim().to_owned();
    }
    if kind != "structure_member" {
        return name.to_owned();
    }
    let name = name
        .split('|')
        .next()
        .unwrap_or(name)
        .trim()
        .trim_end_matches(',')
        .trim();
    match name {
        "Alternative failover endpoints" => "Alternate Failover Endpoints".to_owned(),
        "Timestamp" => "Time Stamp".to_owned(),
        _ => name.to_owned(),
    }
}

fn unique_identifier_value_variant(value_type: &str) -> Result<Value> {
    match value_type {
        "text_string" => Ok(Value::text_string("unique-identifier-fixture".to_owned())),
        "enumeration" => Ok(Value::enumeration(1)),
        "integer" => Ok(Value::integer(256)),
        _ => Err("unsupported Unique Identifier value constructor"),
    }
}

#[allow(clippy::too_many_lines)] // Exact source tag/type selection remains auditable in one recipe.
fn sample(target: &Json, elements: &[Json]) -> Result<Item> {
    if let Some(fixture) = crate::local_credentials::fixture(target) {
        return fixture;
    }
    let kind = text(target, "kind")?;
    let mut name = text(target, "name")?.to_owned();
    if kind == "operation_payload_value_variant"
        && let Some(field_name) = target["field_name"].as_str()
    {
        field_name.clone_into(&mut name);
    }
    if ["enumeration_value", "bitmask_value", "object_type"].contains(&kind)
        && let Some(parent) = target["parent_element_ids"]
            .as_array()
            .and_then(|v| v.first())
            .and_then(Json::as_str)
        && let Some(element) = elements.iter().find(|e| e["element_id"] == parent)
    {
        element["name"]
            .as_str()
            .ok_or("target name missing")?
            .clone_into(&mut name);
    }
    let target_id = target["target_id"].as_str().unwrap_or_default();
    name = concrete_fixture_name(target_id, kind, &name);
    if kind == "operation_payload_field"
        && ["Any Object (Section 2)", "All Objects"].contains(&name.as_str())
    {
        return crate::fixtures::object("Symmetric Key").map(|(_, object, _)| object);
    }
    if kind == "message_field" && name == "Credential" {
        let authentication = message_authentication()?;
        let credential = authentication
            .as_structure()
            .ok_or("authentication fixture is not a Structure")?
            .children()
            .first()
            .ok_or("authentication fixture has no credential")?
            .try_clone()
            .map_err(|_| "credential fixture copy failed")?;
        return Ok(credential);
    }
    let operation = if kind == "operation" {
        operation_value(&name).ok_or("operation target needs a concrete public enum value")?
    } else {
        0
    };
    let tag = if kind == "operation" {
        0x0042_005c
    } else if target_id == "KMIPKIT-ELEM-ATTRIBUTE-VENDOR-ATTRIBUTE" {
        0x0042_0008
    } else if target_id == "KMIPKIT-ELEM-STRUCTURE-MEMBER-7-37-VENDOR-SPECIFIC" {
        // §7.37 permits a vendor-specific tag; this exercises the public generic extension range.
        0x0054_0000
    } else if kind == "tag_range" && target["allocation"] == "extension" {
        let wire_value = text(target, "wire_value")?;
        wire_range_start(wire_value).ok_or("tag range needs a concrete fixture")?
    } else if kind == "referenced_enum_value" {
        match text(target, "parent_name")? {
            "PKCS#11 Function" => 0x0042_015a,
            "PKCS#11 Return Code" => 0x0042_015d,
            _ => return Err("referenced enumeration needs a concrete tag"),
        }
    } else if kind == "tag" {
        unsigned_wire_value(text(target, "wire_value")?).ok_or("invalid target tag")?
    } else {
        elements
            .iter()
            .find(|element| {
                element["kind"] == "tag"
                    && element["allocation"] == "assigned"
                    && element["name"]
                        .as_str()
                        .is_some_and(|tag_name| tag_name.eq_ignore_ascii_case(&name))
            })
            .and_then(|e| e["wire_value"].as_str())
            .and_then(unsigned_wire_value)
            .or_else(|| {
                target["parent_element_ids"]
                    .as_array()?
                    .iter()
                    .find_map(|parent| {
                        u32::from_str_radix(parent.as_str()?.strip_prefix("KMIPKIT-ELEM-TAG-")?, 16)
                            .ok()
                    })
            })
            .or((kind == "enumeration_value" && name == "Item Type").then_some(0x0042_0001))
            .or((kind == "data_type").then_some(0x0042_0001))
            .ok_or("target tag needs a concrete fixture recipe")?
    };
    let wire_value = if kind == "operation" {
        operation
    } else {
        match target["wire_value"].as_str() {
            Some(_)
                if [
                    "enumeration_value",
                    "referenced_enum_value",
                    "bitmask_value",
                    "object_type",
                ]
                .contains(&kind) =>
            {
                target_wire_value(target)?
            }
            _ => 1,
        }
    };
    let encoding = if target_id.contains("ANY-ATTRIBUTE-IN-4-OBJECT-ATTRIBUTES")
        || target_id == "KMIPKIT-ELEM-STRUCTURE-MEMBER-7-22-OBJECT-TYPE-OBJECTTYPES"
    {
        // Exercise the concrete Enumeration branch of the attribute wildcard and wire union.
        "Enumeration"
    } else if kind == "message_field"
        && ["Request Payload", "Response Payload", "Credential"].contains(&name.as_str())
    {
        "Structure"
    } else if kind == "message_field" && name == "Ephemeral" {
        "Boolean"
    } else if kind == "message_field" && name == "Server Hashed Password" {
        "Byte String"
    } else if target_id == "KMIPKIT-ELEM-ATTRIBUTE-VENDOR-ATTRIBUTE" {
        "Structure"
    } else if target_id == "KMIPKIT-ELEM-STRUCTURE-MEMBER-7-23-OBJECT-GROUP"
        || target_id == "KMIPKIT-ELEM-STRUCTURE-MEMBER-7-37-VENDOR-SPECIFIC"
        || kind == "tag_range" && target["allocation"] == "extension"
    {
        "Text String"
    } else {
        target["source_encoding"]
            .as_str()
            .or_else(|| target["value_type"].as_str())
            .or_else(|| {
                elements
                    .iter()
                    .find(|element| element["kind"] == "attribute" && element["name"] == name)
                    .and_then(|element| element["source_encoding"].as_str())
            })
            .or_else(|| {
                elements
                    .iter()
                    .find(|element| {
                        element["name"] == name && element["source_encoding"].is_string()
                    })
                    .and_then(|element| element["source_encoding"].as_str())
            })
            .or_else(|| {
                elements.iter().find_map(|element| {
                    if element["name"] != name {
                        return None;
                    }
                    match element["kind"].as_str() {
                        Some("enumeration") => Some("Enumeration"),
                        Some("bitmask") => Some("Integer"),
                        _ => None,
                    }
                })
            })
            .unwrap_or_else(|| {
                if kind == "data_type" {
                    &name
                } else if [
                    "attribute_structure",
                    "operation_structure",
                    "object_structure",
                ]
                .contains(&kind)
                    || kind == "message_field" && name.contains("Structure")
                {
                    "Structure"
                } else if kind == "operation_payload_field" {
                    match name.as_str() {
                        "New Attribute"
                        | "Current Attribute"
                        | "Common Attributes"
                        | "Private Key Attributes"
                        | "Public Key Attributes"
                        | "Common Protection Storage Masks"
                        | "Private Protection Storage Masks"
                        | "Public Protection Storage Masks"
                        | "Any Object (Section 2)"
                        | "All Objects" => "Structure",
                        "Adjustment Type"
                        | "Derivation Method"
                        | "Key Wrap Type"
                        | "Object Group Member"
                        | "Validity Indicator"
                        | "Query Function"
                        | "Client Registration Method"
                        | "Endpoint Role"
                        | "Put Function" => "Enumeration",
                        "Adjustment Value"
                        | "Request Count"
                        | "Maximum Items"
                        | "Offset Items"
                        | "Storage Status Mask"
                        | "Located Items" => "Integer",
                        "Offset" => "Interval",
                        "Replace Existing" => "Boolean",
                        "Validity Date" => "Date Time",
                        "Digested Data" => "Byte String",
                        "Private Key Unique Identifier"
                        | "Public Key Unique Identifier"
                        | "Interop Identifier"
                        | "Certificate Request Unique Identifier"
                        | "Replaced Unique Identifier" => "Text String",
                        _ => "",
                    }
                } else if kind == "attribute_structure" {
                    "Structure"
                } else {
                    ""
                }
            })
    };
    let value = if target["target_id"] == "KMIPKIT-ELEM-TAG-420008"
        || target_id == "KMIPKIT-ELEM-ATTRIBUTE-VENDOR-ATTRIBUTE"
    {
        // KMIP 2.1 §4.60, Table 150 defines the required Vendor Attribute fields and order.
        vendor_attribute_value()?
    } else if [
        "enumeration_value",
        "referenced_enum_value",
        "object_type",
        "operation",
    ]
    .contains(&kind)
    {
        Value::enumeration(wire_value)
    } else if kind == "bitmask_value" {
        Value::integer(wire_value.cast_signed())
    } else {
        let normalized_encoding = encoding.to_ascii_lowercase();
        if normalized_encoding.contains("boolean") {
            Value::boolean(true)
        } else if normalized_encoding.contains("big integer") {
            Value::big_integer(vec![0; 8])
        } else if normalized_encoding.contains("long integer") {
            Value::long_integer(-17)
        } else if normalized_encoding.contains("byte string")
            || normalized_encoding.contains("bytestring")
        {
            Value::byte_string(vec![0x4b])
        } else if normalized_encoding.contains("date time extended")
            || normalized_encoding.contains("date-time extended")
        {
            Value::date_time_extended(1)
        } else if normalized_encoding.contains("date time")
            || normalized_encoding.contains("date-time")
        {
            Value::date_time(1)
        } else if normalized_encoding.contains("interval") {
            Value::interval(1)
        } else if normalized_encoding.contains("enumeration") {
            Value::enumeration(1)
        } else if normalized_encoding.contains("integer") {
            Value::integer(256)
        } else if normalized_encoding.contains("structure")
            || normalized_encoding.contains("rng parameters")
        {
            // §4.16 Table 59 permits every Cryptographic Parameters member to be absent.
            if tag == 0x0042_002b {
                Value::structure(kmipkit::ttlv::Structure::new())
            } else if kind == "data_type" {
                let mut structure = kmipkit::ttlv::Structure::new();
                structure
                    .try_push(crate::driver::item(0x0042_000d, Value::integer(1))?)
                    .map_err(|_| "local structure rejected")?;
                Value::structure(structure)
            } else {
                Value::structure(kmipkit::ttlv::Structure::new())
            }
        } else if normalized_encoding.contains("text string")
            || normalized_encoding.contains("textstring")
            || normalized_encoding.contains("any")
            || normalized_encoding.contains("varies")
            || kind == "tag"
        {
            Value::text_string("KMIPKit".to_owned())
        } else {
            return Err("target value needs a concrete fixture recipe");
        }
    };
    crate::driver::item(tag, value)
}

fn typed_attribute_marker(symbol: &str) -> Option<&str> {
    let marker = symbol.rsplit("typed::").next()?;
    marker.split(['>', ':']).next()
}

fn normalized_name(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn parent_structure_tag(target: &Json, elements: &[Json]) -> Option<u32> {
    let parent_id = target["parent_element_ids"].as_array()?.first()?.as_str()?;
    let parent = elements
        .iter()
        .find(|element| element["element_id"] == parent_id)?;
    if ![
        "object_structure",
        "attribute_structure",
        "operation_structure",
    ]
    .contains(&parent["kind"].as_str()?)
    {
        return None;
    }
    let parent_name = parent["name"].as_str()?;
    elements
        .iter()
        .find(|element| element["kind"] == "tag" && element["name"] == parent_name)?
        .get("wire_value")?
        .as_str()
        .and_then(unsigned_wire_value)
}

fn typed_item_route(symbol: &str) -> bool {
    symbol.contains("Attribute::<") && symbol.ends_with("::from_item")
        || symbol.contains("::try_push_typed::<")
        || symbol.contains("Attribute::from_typed::<")
}

fn key_format_type_request_payload(symbol: &str, raw: u32) -> Result<kmipkit::ttlv::Structure> {
    let key_format_type = kmipkit::protocol::KeyFormatType::from_raw(raw);
    match symbol {
        "kmipkit_protocol::GetRequest::with_key_format_type" => {
            kmipkit::protocol::GetRequest::new()
                .with_key_format_type(key_format_type)
                .to_ttlv_payload()
                .map_err(|_| "Key Format Type request fixture rejected")
        }
        "kmipkit_protocol::ExportRequest::with_key_format_type" => {
            kmipkit::protocol::ExportRequest::new()
                .with_key_format_type(key_format_type)
                .into_ttlv_payload()
                .map_err(|_| "Key Format Type request fixture rejected")
        }
        _ => Err("unsupported Key Format Type request route"),
    }
}

fn unique_identifier_request_payload(
    symbol: &str,
    identifier: kmipkit::protocol::UniqueIdentifier,
) -> Result<kmipkit::ttlv::Structure> {
    use kmipkit::protocol::{
        AttributeSet, CreateSplitKeyRequest, ObjectType, OperationData, RevocationReason,
        SecretBytes, SplitKeyMethod,
    };

    match symbol {
        "kmipkit_protocol::CertifyRequest::with_unique_identifier" => {
            kmipkit::protocol::CertifyRequest::new()
                .with_certificate_request_type(1)
                .with_certificate_request_value(kmipkit::protocol::SecretBytes::new(vec![0x30]))
                .with_unique_identifier(identifier)
                .into_ttlv_payload()
        }
        "kmipkit_protocol::CheckRequest::with_unique_identifier" => {
            kmipkit::protocol::CheckRequest::new()
                .with_unique_identifier(identifier)
                .into_ttlv_payload()
        }
        "kmipkit_protocol::CreateSplitKeyRequest::with_unique_identifier" => {
            CreateSplitKeyRequest::new(
                ObjectType::from_raw(2),
                2,
                1,
                SplitKeyMethod::from_raw(1),
                AttributeSet::new(),
            )
            .with_unique_identifier(identifier)
            .into_ttlv_payload()
        }
        "kmipkit_protocol::ExportRequest::with_unique_identifier" => {
            kmipkit::protocol::ExportRequest::new()
                .with_unique_identifier(identifier)
                .into_ttlv_payload()
        }
        "kmipkit_protocol::GetRequest::with_unique_identifier" => {
            kmipkit::protocol::GetRequest::new()
                .with_unique_identifier(identifier)
                .to_ttlv_payload()
        }
        "kmipkit_protocol::GetUsageAllocationRequest::with_unique_identifier" => {
            kmipkit::protocol::GetUsageAllocationRequest::new(1)
                .with_unique_identifier(identifier)
                .to_ttlv_payload()
        }
        "kmipkit_protocol::MacRequest::with_unique_identifier" => {
            kmipkit::protocol::MacRequest::new()
                .with_data(OperationData::ByteString(SecretBytes::new(vec![0x5a])))
                .with_unique_identifier(identifier)
                .to_ttlv_payload()
        }
        "kmipkit_protocol::MacVerifyRequest::with_unique_identifier" => {
            kmipkit::protocol::MacVerifyRequest::new()
                .with_mac_data(SecretBytes::new(vec![0x5a]))
                .with_unique_identifier(identifier)
                .to_ttlv_payload()
        }
        "kmipkit_protocol::ObtainLeaseRequest::with_unique_identifier" => {
            kmipkit::protocol::ObtainLeaseRequest::new()
                .with_unique_identifier(identifier)
                .into_ttlv_payload()
        }
        "kmipkit_protocol::RecertifyRequest::with_unique_identifier" => {
            kmipkit::protocol::RecertifyRequest::new()
                .with_certificate_request_type(1)
                .with_certificate_request_value(SecretBytes::new(vec![0x30]))
                .with_unique_identifier(identifier)
                .into_ttlv_payload()
        }
        "kmipkit_protocol::RekeyRequest::with_unique_identifier" => {
            kmipkit::protocol::RekeyRequest::new()
                .with_unique_identifier(identifier)
                .into_ttlv_payload()
        }
        "kmipkit_protocol::RevokeRequest::with_unique_identifier" => {
            kmipkit::protocol::RevokeRequest::new(RevocationReason::from_raw(1))
                .with_unique_identifier(identifier)
                .into_ttlv_payload()
        }
        "kmipkit_protocol::SignatureVerifyRequest::with_unique_identifier" => {
            kmipkit::protocol::SignatureVerifyRequest::new()
                .with_signature_data(SecretBytes::new(vec![0x5a]))
                .with_unique_identifier(identifier)
                .to_ttlv_payload()
        }
        "kmipkit_protocol::SignRequest::with_unique_identifier" => {
            kmipkit::protocol::SignRequest::new()
                .with_digested_data(SecretBytes::new(vec![0x5a]))
                .with_unique_identifier(identifier)
                .to_ttlv_payload()
        }
        "kmipkit_protocol::ValidateRequest::with_unique_identifier" => {
            kmipkit::protocol::ValidateRequest::new()
                .with_unique_identifier(identifier)
                .into_ttlv_payload()
        }
        _ => return Err("unsupported Unique Identifier request route"),
    }
    .map_err(|_| "Unique Identifier request fixture rejected")
}

fn request_payload_item(payload: kmipkit::ttlv::Structure) -> Result<Item> {
    crate::driver::item(0x0042_0079, Value::structure(payload))
}

const MANAGED_OBJECT_ROOTS: [&str; 9] = [
    "Certificate",
    "Certificate Request",
    "Opaque Object",
    "PGP Key",
    "Private Key",
    "Public Key",
    "Secret Data",
    "Split Key",
    "Symmetric Key",
];

fn typed_object_root(symbol: &str) -> Option<&'static str> {
    match symbol {
        "kmipkit_protocol::Certificate::try_from_item" => Some("Certificate"),
        "kmipkit_protocol::CertificateRequest::try_from_item" => Some("Certificate Request"),
        "kmipkit_protocol::OpaqueObject::try_from_item" => Some("Opaque Object"),
        "kmipkit_protocol::PgpKey::try_from_item" => Some("PGP Key"),
        "kmipkit_protocol::PrivateKey::try_from_item" => Some("Private Key"),
        "kmipkit_protocol::PublicKey::try_from_item" => Some("Public Key"),
        "kmipkit_protocol::SecretData::try_from_item" => Some("Secret Data"),
        "kmipkit_protocol::SplitKey::try_from_item" => Some("Split Key"),
        "kmipkit_protocol::SymmetricKey::try_from_item" => Some("Symmetric Key"),
        _ => None,
    }
}

fn typed_object_constructor_root(symbol: &str) -> Option<&'static str> {
    match symbol {
        "kmipkit_protocol::Certificate::try_new" => Some("Certificate"),
        "kmipkit_protocol::CertificateRequest::try_new" => Some("Certificate Request"),
        "kmipkit_protocol::OpaqueObject::try_new" => Some("Opaque Object"),
        "kmipkit_protocol::PgpKey::try_new" => Some("PGP Key"),
        "kmipkit_protocol::PrivateKey::try_new" => Some("Private Key"),
        "kmipkit_protocol::PublicKey::try_new" => Some("Public Key"),
        "kmipkit_protocol::SecretData::try_new" => Some("Secret Data"),
        "kmipkit_protocol::SplitKey::try_new" => Some("Split Key"),
        "kmipkit_protocol::SymmetricKey::try_new" => Some("Symmetric Key"),
        _ => None,
    }
}

fn key_material_format_for_target(target: &Json) -> Option<u32> {
    let section = target["source_refs"]
        .as_array()?
        .iter()
        .find_map(|source| source["section"].as_str())?;
    let raw = match section {
        "3.4" => 7,
        "3.5" => 8,
        "3.6" => 9,
        "3.7" => 10,
        "3.8" => 11,
        "3.9" => 12,
        "3.10" => 13,
        "3.11" => 20,
        "3.12" => 21,
        _ => return None,
    };
    Some(raw)
}

fn transparent_key_material(format: u32) -> Result<Item> {
    use kmipkit::protocol::{
        TransparentDhPrivateKey, TransparentDhPublicKey, TransparentDsaPrivateKey,
        TransparentDsaPublicKey, TransparentEcPrivateKey, TransparentEcPublicKey,
        TransparentRsaPrivateKey, TransparentRsaPublicKey, TransparentSymmetricKey,
    };
    let component = || vec![0x31, 0x41];
    match format {
        7 => TransparentSymmetricKey::try_new(vec![0x41; 16])
            .map(TransparentSymmetricKey::into_item)
            .map_err(|_| "transparent symmetric key fixture rejected"),
        8 => TransparentDsaPrivateKey::try_new(component(), component(), component(), component())
            .map(TransparentDsaPrivateKey::into_item)
            .map_err(|_| "transparent DSA private key fixture rejected"),
        9 => TransparentDsaPublicKey::try_new(component(), component(), component(), component())
            .map(TransparentDsaPublicKey::into_item)
            .map_err(|_| "transparent DSA public key fixture rejected"),
        10 => TransparentRsaPrivateKey::try_from_components(
            component(),
            Some(component()),
            Some(component()),
            Some(component()),
            Some(component()),
            Some(component()),
            Some(component()),
            Some(component()),
        )
        .map(TransparentRsaPrivateKey::into_item)
        .map_err(|_| "transparent RSA private key fixture rejected"),
        11 => TransparentRsaPublicKey::try_new(component(), component())
            .map(TransparentRsaPublicKey::into_item)
            .map_err(|_| "transparent RSA public key fixture rejected"),
        12 => TransparentDhPrivateKey::try_new(
            component(),
            component(),
            component(),
            Some(component()),
            Some(component()),
        )
        .map(TransparentDhPrivateKey::into_item)
        .map_err(|_| "transparent DH private key fixture rejected"),
        13 => TransparentDhPublicKey::try_new(
            component(),
            component(),
            component(),
            Some(component()),
            Some(component()),
        )
        .map(TransparentDhPublicKey::into_item)
        .map_err(|_| "transparent DH public key fixture rejected"),
        20 => TransparentEcPrivateKey::try_new(1, component())
            .map(TransparentEcPrivateKey::into_item)
            .map_err(|_| "transparent EC private key fixture rejected"),
        21 => TransparentEcPublicKey::try_new(1, component())
            .map(TransparentEcPublicKey::into_item)
            .map_err(|_| "transparent EC public key fixture rejected"),
        _ => Err("unsupported transparent Key Format Type fixture"),
    }
}

fn key_material_item_for_target(target: &Json, default_format: u32) -> Result<(u32, Item)> {
    use kmipkit::protocol::{KeyFormatType, KeyMaterial};

    let format = key_material_format_for_target(target).unwrap_or(default_format);
    let item = if matches!(format, 7..=13 | 20..=21) {
        transparent_key_material(format)?
    } else {
        KeyMaterial::byte_string(format, vec![0x41; 32])
            .map(KeyMaterial::into_item)
            .map_err(|_| "raw Key Material fixture rejected")?
    };
    let parsed = KeyMaterial::try_from_item(item, KeyFormatType::from_raw(format))
        .map_err(|_| "Key Material fixture rejected")?
        .into_item();
    Ok((format, parsed))
}

fn key_material_structure_item_for_target(target: &Json) -> Result<(u32, Item)> {
    use kmipkit::protocol::{KeyFormatType, KeyMaterial};

    let format = key_material_format_for_target(target).unwrap_or(7);
    let transparent = transparent_key_material(format)?;
    let structure = transparent.with_value(|value| match value {
        ValueView::Structure(structure) => structure
            .try_clone()
            .map_err(|_| "Key Material structure fixture copy failed"),
        _ => Err("transparent Key Material fixture is not a Structure"),
    })?;
    let item = KeyMaterial::structure(format, structure)
        .map(KeyMaterial::into_item)
        .map_err(|_| "Key Material structure fixture rejected")?;
    let parsed = KeyMaterial::try_from_item(item, KeyFormatType::from_raw(format))
        .map(KeyMaterial::into_item)
        .map_err(|_| "Key Material structure fixture rejected")?;
    Ok((format, parsed))
}

fn key_value_item_for_target(target: &Json) -> Result<Item> {
    use kmipkit::protocol::{AttributeSet, KeyFormatType, KeyMaterial, KeyValue};

    let (format, item) = key_material_item_for_target(target, 1)?;
    let material = KeyMaterial::try_from_item(item, KeyFormatType::from_raw(format))
        .map_err(|_| "Key Material fixture rejected")?;
    let has_attributes = target["target_id"]
        .as_str()
        .is_some_and(|id| id.contains("KEY-VALUE-ATTRIBUTES"));
    KeyValue::try_new(material, has_attributes.then(AttributeSet::new))
        .map(KeyValue::into_item)
        .map_err(|_| "Key Value fixture rejected")
}

fn key_block_for_target(target: &Json, secret_data: bool) -> Result<kmipkit::protocol::KeyBlock> {
    use kmipkit::protocol::{
        AttributeSet, EncryptionKeyInformation, KeyBlock, KeyBlockValue, KeyCompressionType,
        KeyFormatType, KeyMaterial, KeyValue, KeyWrappingData, MacSignatureKeyInformation,
        SecretBytes,
    };

    let format = key_material_format_for_target(target).unwrap_or(1);
    let material_item = if format == 1 {
        KeyMaterial::byte_string(format, vec![0x41; 32])
            .map_err(|_| "raw key material fixture rejected")?
            .into_item()
    } else {
        transparent_key_material(format)?
    };
    let material = KeyMaterial::try_from_item(material_item, KeyFormatType::from_raw(format))
        .map_err(|_| "Key Material fixture rejected")?;
    let target_id = target["target_id"].as_str().unwrap_or_default();
    let key_value_attributes = target_id.contains("KEY-VALUE-ATTRIBUTES");
    let attributes = key_value_attributes.then(AttributeSet::new);
    let value =
        KeyValue::try_new(material, attributes).map_err(|_| "Key Value fixture rejected")?;
    let mut block = KeyBlock::try_new(KeyFormatType::from_raw(format))
        .map_err(|_| "Key Format Type fixture rejected")?
        .with_key_value(KeyBlockValue::Plaintext(value));
    if target_id.contains("KEY-COMPRESSION-TYPE") || target_id == "KMIPKIT-ELEM-TAG-420041" {
        block = block.with_key_compression_type(KeyCompressionType::from_raw(1));
    }
    let include_cryptographic_properties =
        target_id.contains("CRYPTOGRAPHIC-ALGORITHM") || target_id.contains("CRYPTOGRAPHIC-LENGTH");
    if !secret_data || include_cryptographic_properties {
        block = block.with_cryptographic_properties(Some(3), Some(256));
    }
    if target_id.contains("KEY-WRAPPING-DATA")
        || target_id == "KMIPKIT-ELEM-TAG-420046"
        || target_id.contains("ENCRYPTION-KEY-INFORMATION")
        || target_id.contains("MAC-SIGNATURE-KEY-INFORMATION")
    {
        let encryption = EncryptionKeyInformation::try_new("kmipkit-encryption-key".to_owned())
            .and_then(|value| value.with_cryptographic_parameters(kmipkit::ttlv::Structure::new()))
            .map_err(|_| "encryption key information fixture rejected")?;
        let mac = MacSignatureKeyInformation::try_new("kmipkit-mac-key".to_owned())
            .and_then(|value| value.with_cryptographic_parameters(kmipkit::ttlv::Structure::new()))
            .map_err(|_| "MAC key information fixture rejected")?;
        let wrapping = KeyWrappingData::try_new(1)
            .map_err(|_| "key wrapping fixture rejected")?
            .with_encryption_key_information(encryption)
            .with_mac_signature_key_information(mac)
            .with_mac_signature(vec![0x5a])
            .with_iv_counter_nonce(vec![0x41])
            .with_encoding_option(1);
        block = KeyBlock::try_new(KeyFormatType::from_raw(format))
            .map_err(|_| "Key Format Type fixture rejected")?
            .with_key_value(KeyBlockValue::Wrapped(SecretBytes::new(vec![0x41; 16])))
            .with_key_wrapping_data(wrapping);
    }
    Ok(block)
}

fn construct_typed_object(symbol: &str, target: &Json) -> Option<Result<Item>> {
    use kmipkit::protocol::{
        Certificate, CertificateRequest, OpaqueObject, PgpKey, PrivateKey, PublicKey, SecretData,
        SplitKey, SplitKeyMethod, SymmetricKey,
    };

    let result = match symbol {
        "kmipkit_protocol::Certificate::try_new" => {
            Certificate::try_new(1, crate::fixtures::CERTIFICATE.to_vec())
                .map(Certificate::into_item)
                .map_err(|_| "Certificate constructor rejected")
        }
        "kmipkit_protocol::CertificateRequest::try_new" => {
            CertificateRequest::try_new(1, crate::fixtures::CSR.to_vec())
                .map(CertificateRequest::into_item)
                .map_err(|_| "Certificate Request constructor rejected")
        }
        "kmipkit_protocol::OpaqueObject::try_new" => {
            OpaqueObject::try_new(1, b"dedicated fixture".to_vec())
                .map(OpaqueObject::into_item)
                .map_err(|_| "Opaque Object constructor rejected")
        }
        "kmipkit_protocol::PgpKey::try_new" => key_block_for_target(target, false)
            .and_then(|key_block| PgpKey::try_new(4, key_block).map_err(|_| "invalid PGP key"))
            .map(PgpKey::into_item)
            .map_err(|_| "PGP Key constructor rejected"),
        "kmipkit_protocol::PrivateKey::try_new" => key_block_for_target(target, false)
            .and_then(|key_block| PrivateKey::try_new(key_block).map_err(|_| "invalid private key"))
            .map(PrivateKey::into_item)
            .map_err(|_| "Private Key constructor rejected"),
        "kmipkit_protocol::PublicKey::try_new" => key_block_for_target(target, false)
            .and_then(|key_block| PublicKey::try_new(key_block).map_err(|_| "invalid public key"))
            .map(PublicKey::into_item)
            .map_err(|_| "Public Key constructor rejected"),
        "kmipkit_protocol::SecretData::try_new" => key_block_for_target(target, true)
            .and_then(|key_block| {
                SecretData::try_new(1, key_block).map_err(|_| "invalid secret data")
            })
            .map(SecretData::into_item)
            .map_err(|_| "Secret Data constructor rejected"),
        "kmipkit_protocol::SplitKey::try_new" => key_block_for_target(target, false)
            .and_then(|key_block| {
                let prime_field_size = target["target_id"]
                    .as_str()
                    .is_some_and(|id| id.contains("PRIME-FIELD-SIZE"))
                    .then(|| vec![0x11]);
                SplitKey::try_new(
                    key_block,
                    1,
                    SplitKeyMethod::from_raw(1),
                    2,
                    2,
                    prime_field_size,
                )
                .map_err(|_| "invalid split key")
            })
            .map(SplitKey::into_item)
            .map_err(|_| "Split Key constructor rejected"),
        "kmipkit_protocol::SymmetricKey::try_new" => key_block_for_target(target, false)
            .and_then(|key_block| {
                SymmetricKey::try_new(key_block).map_err(|_| "invalid symmetric key")
            })
            .map(SymmetricKey::into_item)
            .map_err(|_| "Symmetric Key constructor rejected"),
        _ => return None,
    };
    Some(result)
}

fn invoke_typed_object_parser(symbol: &str, object: Item) -> Option<Result<Item>> {
    let result = match symbol {
        "kmipkit_protocol::Certificate::try_from_item" => {
            kmipkit::protocol::Certificate::try_from_item(object)
                .map(kmipkit::protocol::Certificate::into_item)
        }
        "kmipkit_protocol::CertificateRequest::try_from_item" => {
            kmipkit::protocol::CertificateRequest::try_from_item(object)
                .map(kmipkit::protocol::CertificateRequest::into_item)
        }
        "kmipkit_protocol::OpaqueObject::try_from_item" => {
            kmipkit::protocol::OpaqueObject::try_from_item(object)
                .map(kmipkit::protocol::OpaqueObject::into_item)
        }
        "kmipkit_protocol::PgpKey::try_from_item" => {
            kmipkit::protocol::PgpKey::try_from_item(object)
                .map(kmipkit::protocol::PgpKey::into_item)
        }
        "kmipkit_protocol::PrivateKey::try_from_item" => {
            kmipkit::protocol::PrivateKey::try_from_item(object)
                .map(kmipkit::protocol::PrivateKey::into_item)
        }
        "kmipkit_protocol::PublicKey::try_from_item" => {
            kmipkit::protocol::PublicKey::try_from_item(object)
                .map(kmipkit::protocol::PublicKey::into_item)
        }
        "kmipkit_protocol::SecretData::try_from_item" => {
            kmipkit::protocol::SecretData::try_from_item(object)
                .map(kmipkit::protocol::SecretData::into_item)
        }
        "kmipkit_protocol::SplitKey::try_from_item" => {
            kmipkit::protocol::SplitKey::try_from_item(object)
                .map(kmipkit::protocol::SplitKey::into_item)
        }
        "kmipkit_protocol::SymmetricKey::try_from_item" => {
            kmipkit::protocol::SymmetricKey::try_from_item(object)
                .map(kmipkit::protocol::SymmetricKey::into_item)
        }
        _ => return None,
    };
    Some(result.map_err(|_| "typed object parser rejected"))
}

fn managed_object_root_name(target: &Json, elements: &[Json]) -> Option<&'static str> {
    let mut pending = target["parent_element_ids"]
        .as_array()?
        .iter()
        .filter_map(Json::as_str)
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if let Some(name) = target["name"].as_str()
        && let Some(root) = MANAGED_OBJECT_ROOTS.iter().find(|root| **root == name)
    {
        return Some(root);
    }
    while let Some(element_id) = pending.pop() {
        let element = elements
            .iter()
            .find(|element| element["element_id"] == element_id)?;
        let name = element["name"].as_str()?;
        if element["kind"] == "object_structure"
            && let Some(root) = MANAGED_OBJECT_ROOTS.iter().find(|root| **root == name)
        {
            return Some(root);
        }
        if let Some(parents) = element["parent_element_ids"].as_array() {
            pending.extend(parents.iter().filter_map(Json::as_str).map(str::to_owned));
        }
    }
    None
}

fn collect_tagged_items(item: &Item, raw_tag: u32, matches: &mut Vec<Item>) -> Result<()> {
    if item.tag().raw() == raw_tag {
        matches.push(item.try_clone().map_err(|_| "object fixture copy failed")?);
    }
    if let Some(structure) = item.as_structure() {
        for child in structure.children() {
            collect_tagged_items(child, raw_tag, matches)?;
        }
    }
    Ok(())
}

fn collect_direct_tagged_items(item: &Item, raw_tag: u32, matches: &mut Vec<Item>) -> Result<()> {
    item.with_value(|value| {
        let ValueView::Structure(structure) = value else {
            return Ok(());
        };
        for child in structure.children() {
            if child.tag().raw() == raw_tag {
                matches.push(child.try_clone().map_err(|_| "fixture item copy failed")?);
            }
        }
        Ok(())
    })
}

fn target_parent_tag(target: &Json, elements: &[Json]) -> Option<u32> {
    let parent_id = target["parent_element_ids"].as_array()?.first()?.as_str()?;
    let parent_name = elements
        .iter()
        .find(|element| element["element_id"] == parent_id)?["name"]
        .as_str()?;
    let tag = elements
        .iter()
        .find(|element| element["kind"] == "tag" && element["name"] == parent_name)?;
    u32::from_str_radix(tag["wire_value"].as_str()?.trim_start_matches("0x"), 16).ok()
}

fn object_root_fixture(
    root_name: &str,
    target: &Json,
    elements: &[Json],
) -> Result<(u32, Item, Item)> {
    let (object_type, fallback_object, _) = crate::fixtures::object(root_name)?;
    let constructor = match root_name {
        "Certificate" => "kmipkit_protocol::Certificate::try_new",
        "Certificate Request" => "kmipkit_protocol::CertificateRequest::try_new",
        "Opaque Object" => "kmipkit_protocol::OpaqueObject::try_new",
        "PGP Key" => "kmipkit_protocol::PgpKey::try_new",
        "Private Key" => "kmipkit_protocol::PrivateKey::try_new",
        "Public Key" => "kmipkit_protocol::PublicKey::try_new",
        "Secret Data" => "kmipkit_protocol::SecretData::try_new",
        "Split Key" => "kmipkit_protocol::SplitKey::try_new",
        "Symmetric Key" => "kmipkit_protocol::SymmetricKey::try_new",
        _ => return Err("unsupported managed object fixture"),
    };
    let object = if let Some(object) = construct_typed_object(constructor, target) {
        object?
    } else {
        fallback_object
    };
    let expected = if target["kind"] == "object_structure" && target["name"] == root_name {
        object
            .try_clone()
            .map_err(|_| "Managed Object fixture copy failed")?
    } else {
        let target_item = sample(target, elements)?;
        let mut matching_items = Vec::new();
        if target["kind"] == "structure_member"
            && let Some(parent_tag) = target_parent_tag(target, elements)
        {
            let mut parents = Vec::new();
            collect_tagged_items(&object, parent_tag, &mut parents)?;
            for parent in &parents {
                collect_direct_tagged_items(parent, target_item.tag().raw(), &mut matching_items)?;
            }
        } else {
            collect_tagged_items(&object, target_item.tag().raw(), &mut matching_items)?;
        }
        match matching_items.as_slice() {
            [matched] => matched
                .try_clone()
                .map_err(|_| "Managed Object target copy failed")?,
            [] => return Err("Managed Object fixture does not contain the target"),
            _ => return Err("Managed Object target is ambiguous in fixture"),
        }
    };
    let expected_type = sample(target, elements)?.item_type();
    let source_encoding = target["source_encoding"].as_str().unwrap_or_default();
    let encoding_is_union = target["name"] == "Key Value"
        && source_encoding.contains("Byte String")
        && source_encoding.contains("Structure");
    if !matches!(target["kind"].as_str(), Some("tag" | "object_type"))
        && !encoding_is_union
        && expected.item_type() != expected_type
    {
        return Err("managed object fixture does not match target Item Type");
    }
    Ok((object_type.raw(), object, expected))
}

fn managed_object_fixture(target: &Json, elements: &[Json]) -> Result<(u32, Item, Item)> {
    let root_name = managed_object_root_name(target, elements)
        .ok_or("Managed Object fixture target has no object root")?;
    object_root_fixture(root_name, target, elements)
}

fn key_block_fixture(target: &Json, elements: &[Json]) -> Result<(Item, Item)> {
    let (_, object, expected) = managed_object_fixture(target, elements)?;
    let mut key_blocks = Vec::new();
    collect_tagged_items(&object, KEY_BLOCK_TAG, &mut key_blocks)?;
    match key_blocks.as_slice() {
        [key_block] => Ok((
            key_block
                .try_clone()
                .map_err(|_| "Key Block fixture copy failed")?,
            expected,
        )),
        [] => Err("Managed Object fixture has no Key Block"),
        _ => Err("Managed Object fixture has ambiguous Key Blocks"),
    }
}

fn construct_key_block_fixture(target: &Json, elements: &[Json]) -> Result<Item> {
    let secret_data = managed_object_root_name(target, elements) == Some("Secret Data");
    key_block_for_target(target, secret_data)?
        .try_into_item()
        .map_err(|_| "Key Block constructor rejected")
}

fn metadata_key_information(
    mac: bool,
    include_parameters: bool,
    typed_parameters: bool,
) -> Result<Item> {
    use kmipkit::protocol::{EncryptionKeyInformation, MacSignatureKeyInformation};

    if mac {
        let mut info = MacSignatureKeyInformation::try_new("kmipkit-mac-key".to_owned())
            .map_err(|_| "MAC key information fixture rejected")?;
        if include_parameters {
            if typed_parameters {
                let parameters = kmipkit::protocol::CryptographicParametersBuilder::new()
                    .build()
                    .map_err(|_| "typed MAC cryptographic parameters fixture rejected")?;
                info = info
                    .with_typed_cryptographic_parameters(parameters)
                    .map_err(|_| "typed MAC cryptographic parameters fixture rejected")?;
            } else {
                info = info
                    .with_cryptographic_parameters(kmipkit::ttlv::Structure::new())
                    .map_err(|_| "MAC cryptographic parameters fixture rejected")?;
            }
        }
        Ok(info.into_item())
    } else {
        let mut info = EncryptionKeyInformation::try_new("kmipkit-encryption-key".to_owned())
            .map_err(|_| "encryption key information fixture rejected")?;
        if include_parameters {
            if typed_parameters {
                let parameters = kmipkit::protocol::CryptographicParametersBuilder::new()
                    .build()
                    .map_err(|_| "typed encryption cryptographic parameters fixture rejected")?;
                info = info
                    .with_typed_cryptographic_parameters(parameters)
                    .map_err(|_| "typed encryption cryptographic parameters fixture rejected")?;
            } else {
                info = info
                    .with_cryptographic_parameters(kmipkit::ttlv::Structure::new())
                    .map_err(|_| "encryption cryptographic parameters fixture rejected")?;
            }
        }
        Ok(info.into_item())
    }
}

fn key_wrapping_metadata_model(symbol: &str, target: &Json) -> Result<Item> {
    use kmipkit::protocol::KeyWrappingData;

    let target_id = text(target, "target_id")?;
    let target_name = text(target, "name")?;
    let wrapping_root = symbol == "kmipkit_protocol::KeyWrappingData::try_from_item";
    let encryption_target = target_id.contains("ENCRYPTION-KEY-INFORMATION")
        || target_name == "Encryption Key Information";
    let mac_target = target_id.contains("MAC-SIGNATURE-KEY-INFORMATION")
        || target_name == "MAC/Signature Key Information";
    let full_wrapping_root = target_name == "Key Wrapping Data";
    let generic_unique_identifier_tag = target_id == "KMIPKIT-ELEM-TAG-420094";
    let include_encryption = !wrapping_root || !mac_target || full_wrapping_root;
    let include_mac =
        mac_target || full_wrapping_root || !wrapping_root && !generic_unique_identifier_tag;
    let include_parameters = target_name == "Cryptographic Parameters";
    if symbol.starts_with("kmipkit_protocol::EncryptionKeyInformation::") {
        return metadata_key_information(
            false,
            include_parameters,
            symbol.ends_with("::with_typed_cryptographic_parameters"),
        );
    }
    if symbol.starts_with("kmipkit_protocol::MacSignatureKeyInformation::") {
        return metadata_key_information(
            true,
            include_parameters,
            symbol.ends_with("::with_typed_cryptographic_parameters"),
        );
    }

    let mut data = KeyWrappingData::try_new(1).map_err(|_| "key wrapping fixture rejected")?;
    if include_encryption {
        let info =
            kmipkit::protocol::EncryptionKeyInformation::try_from_item(metadata_key_information(
                false,
                include_parameters && (encryption_target || !mac_target),
                false,
            )?)
            .map_err(|_| "encryption key information fixture rejected")?;
        data = data.with_encryption_key_information(info);
    }
    if include_mac {
        let info = kmipkit::protocol::MacSignatureKeyInformation::try_from_item(
            metadata_key_information(true, include_parameters && mac_target, false)?,
        )
        .map_err(|_| "MAC key information fixture rejected")?;
        data = data.with_mac_signature_key_information(info);
    }
    if full_wrapping_root {
        data = data
            .with_mac_signature(vec![0x5a])
            .with_iv_counter_nonce(vec![0x41])
            .with_encoding_option(1);
    } else {
        match target_name {
            "MAC/Signature" => data = data.with_mac_signature(vec![0x5a]),
            "IV/Counter/Nonce" => data = data.with_iv_counter_nonce(vec![0x41]),
            "Encoding Option" => data = data.with_encoding_option(1),
            _ => {}
        }
    }
    data.into_item()
        .map_err(|_| "key wrapping data fixture rejected")
}

fn metadata_expected_target(model: &Item, target: &Json, elements: &[Json]) -> Result<Item> {
    let expected_tag = if target["kind"] == "message_field" {
        message_field_tag(target, elements)?
    } else {
        sample(target, elements)?.tag().raw()
    };
    if model.tag().raw() == expected_tag {
        return model
            .try_clone()
            .map_err(|_| "metadata fixture copy failed");
    }
    let mut tagged_items = Vec::new();
    if target["kind"] == "structure_member"
        && let Some(parent_tag) = target_parent_tag(target, elements)
    {
        let mut parents = Vec::new();
        collect_tagged_items(model, parent_tag, &mut parents)?;
        for parent in &parents {
            collect_direct_tagged_items(parent, expected_tag, &mut tagged_items)?;
        }
    } else {
        collect_tagged_items(model, expected_tag, &mut tagged_items)?;
    }
    match tagged_items.as_slice() {
        [found_item] => found_item
            .try_clone()
            .map_err(|_| "metadata target copy failed"),
        [] => Err("metadata fixture target is missing"),
        _ => Err("metadata fixture target is ambiguous"),
    }
}

fn message_field_tag(target: &Json, elements: &[Json]) -> Result<u32> {
    let name = text(target, "name")?.trim_end_matches(", MAY be repeated");
    elements
        .iter()
        .find(|element| element["kind"] == "tag" && element["name"] == name)
        .and_then(|element| element["wire_value"].as_str())
        .and_then(unsigned_wire_value)
        .ok_or("message field has no assigned TTLV tag")
}

fn message_target_tag(target: &Json, elements: &[Json]) -> Result<Option<u32>> {
    match target["kind"].as_str() {
        Some("message_field") => message_field_tag(target, elements).map(Some),
        Some("tag") => sample(target, elements).map(|item| Some(item.tag().raw())),
        _ => Ok(None),
    }
}

fn message_item(tag: u32, value: Value) -> Result<Item> {
    crate::driver::item(tag, value)
}

fn message_structure(tag: u32, children: Vec<Item>) -> Result<Item> {
    let structure = crate::driver::structure(tag, children)?;
    message_item(tag, Value::structure(structure))
}

fn message_scalar(tag: u32, value: Value, target: &Json, elements: &[Json]) -> Result<Item> {
    if message_target_tag(target, elements)? == Some(tag) {
        // Some §8 targets omit an explicit encoding in the inventory; the
        // field table already supplies its type through this fixture recipe.
        sample(target, elements).or_else(|_| message_item(tag, value))
    } else {
        message_item(tag, value)
    }
}

fn message_extension(target: &Json, elements: &[Json]) -> Result<Item> {
    let target_tag = message_target_tag(target, elements)?;
    let vendor_id = target_tag == Some(0x0042_009D);
    let criticality = target_tag == Some(0x0042_0026);
    let vendor_identification = message_item(
        0x0042_009D,
        if vendor_id {
            sample(target, elements)?.with_value(value_to_owned)?
        } else {
            Value::text_string("KMIPKit".to_owned())
        },
    )?;
    let criticality_indicator = message_item(
        0x0042_0026,
        if criticality {
            sample(target, elements)?.with_value(value_to_owned)?
        } else {
            Value::boolean(true)
        },
    )?;
    let vendor_extension_item = message_structure(
        0x0042_009C,
        vec![message_item(
            0x0054_0001,
            Value::text_string("fixture".to_owned()),
        )?],
    )?;
    let extension = message_structure(
        0x0042_0051,
        vec![
            vendor_identification,
            criticality_indicator,
            vendor_extension_item,
        ],
    )?;
    Ok(extension)
}

fn message_protocol_version(
    target: &Json,
    elements: &[Json],
    protocol_version: &str,
) -> Result<Item> {
    let (major, minor) = match protocol_version {
        "2.0" => (2, 0),
        "2.1" => (2, 1),
        _ => return Err("unsupported protocol version in message fixture"),
    };
    message_structure(
        0x0042_0069,
        vec![
            message_scalar(0x0042_006A, Value::integer(major), target, elements)?,
            message_scalar(0x0042_006B, Value::integer(minor), target, elements)?,
        ],
    )
}

fn message_authentication() -> Result<Item> {
    let credential_type = message_item(0x0042_0024, Value::enumeration(1))?;
    let credential_value = message_structure(
        0x0042_0025,
        vec![
            message_item(0x0042_0099, Value::text_string("interop-user".to_owned()))?,
            message_item(
                0x0042_00A1,
                Value::text_string("fixture-password".to_owned()),
            )?,
        ],
    )?;
    let credential = message_structure(0x0042_0023, vec![credential_type, credential_value])?;
    message_structure(0x0042_000C, vec![credential])
}

fn message_nonce(target: &Json, elements: &[Json]) -> Result<Item> {
    message_structure(
        0x0042_00C8,
        vec![
            message_scalar(
                0x0042_00C9,
                Value::byte_string(vec![0x4e]),
                target,
                elements,
            )?,
            message_scalar(
                0x0042_00CA,
                Value::byte_string(vec![0x56]),
                target,
                elements,
            )?,
        ],
    )
}

fn message_batch_extension(target: &Json, elements: &[Json]) -> Result<Option<Item>> {
    let target_tag = message_target_tag(target, elements)?;
    if target_tag
        .is_some_and(|tag| [0x0042_0051, 0x0042_0026, 0x0042_009C, 0x0042_009D].contains(&tag))
    {
        Ok(Some(message_extension(target, elements)?))
    } else {
        Ok(None)
    }
}

fn request_batch_item(index: usize, target: &Json, elements: &[Json]) -> Result<Item> {
    let operation_tag = 0x0042_005C;
    let operation = message_scalar(
        operation_tag,
        Value::enumeration(operation_value("Query").ok_or("Query operation is unavailable")?),
        target,
        elements,
    )?;
    let ephemeral = message_scalar(0x0042_0154, Value::boolean(false), target, elements)?;
    let unique_id = if message_target_tag(target, elements)? == Some(0x0042_0093) {
        sample(target, elements)?
    } else {
        let identifier = u8::try_from(index + 1).map_err(|_| "too many request batch items")?;
        message_item(0x0042_0093, Value::byte_string(vec![0x4b, identifier]))?
    };
    let payload = message_structure(0x0042_0079, Vec::new())?;
    let mut fields = vec![operation, ephemeral, unique_id, payload];
    if let Some(extension) = message_batch_extension(target, elements)? {
        fields.push(extension);
    }
    message_structure(0x0042_000F, fields)
}

fn request_message_fixture(
    target: &Json,
    elements: &[Json],
    protocol_version: &str,
) -> Result<Item> {
    let target_tag = message_target_tag(target, elements)?;
    let multiple = target_tag.is_some_and(|tag| [0x0042_000E, 0x0042_0010].contains(&tag));
    let count: usize = if multiple { 2 } else { 1 };
    let version = message_protocol_version(target, elements, protocol_version)?;
    let mut header = vec![version];
    header.push(message_scalar(
        0x0042_0050,
        Value::integer(4096),
        target,
        elements,
    )?);
    header.push(message_scalar(
        0x0042_0105,
        Value::text_string("client-correlation".to_owned()),
        target,
        elements,
    )?);
    header.push(message_scalar(
        0x0042_0106,
        Value::text_string("server-correlation".to_owned()),
        target,
        elements,
    )?);
    header.push(message_scalar(
        0x0042_0007,
        Value::enumeration(1),
        target,
        elements,
    )?);
    header.push(message_scalar(
        0x0042_00D3,
        Value::boolean(true),
        target,
        elements,
    )?);
    header.push(message_scalar(
        0x0042_00C7,
        Value::enumeration(1),
        target,
        elements,
    )?);
    header.push(message_authentication()?);
    if target_tag == Some(0x0042_000E) {
        header.push(sample(target, elements)?);
    }
    if target_tag == Some(0x0042_0010) {
        header.push(sample(target, elements)?);
    }
    header.push(message_scalar(
        0x0042_0092,
        Value::date_time(1),
        target,
        elements,
    )?);
    header.push(message_item(
        0x0042_000D,
        Value::integer(i32::try_from(count).map_err(|_| "too many request batch items")?),
    )?);
    let request_header = message_structure(0x0042_0077, header)?;
    let mut children = vec![request_header];
    for index in 0..count {
        children.push(request_batch_item(index, target, elements)?);
    }
    message_structure(0x0042_0078, children)
}

fn response_batch_item(target: &Json, elements: &[Json]) -> Result<Item> {
    let target_tag = message_target_tag(target, elements)?;
    let pending = target_tag == Some(0x0042_0006);
    let failed = matches!(target_tag, Some(0x0042_007D | 0x0042_007E));
    let operation = if pending {
        operation_value("Poll").ok_or("Poll operation is unavailable")?
    } else {
        operation_value("Query").ok_or("Query operation is unavailable")?
    };
    let mut fields = vec![
        message_scalar(0x0042_005C, Value::enumeration(operation), target, elements)?,
        message_scalar(
            0x0042_0093,
            Value::byte_string(vec![0x4b, 1]),
            target,
            elements,
        )?,
        message_item(
            0x0042_007F,
            Value::enumeration(if pending { 2 } else { u32::from(failed) }),
        )?,
    ];
    if failed {
        fields.push(message_scalar(
            0x0042_007E,
            Value::enumeration(1),
            target,
            elements,
        )?);
        fields.push(message_scalar(
            0x0042_007D,
            Value::text_string("controlled fixture failure".to_owned()),
            target,
            elements,
        )?);
    } else if pending {
        fields.push(message_scalar(
            0x0042_0006,
            Value::byte_string(vec![0x41, 0x53, 0x59, 0x4e, 0x43]),
            target,
            elements,
        )?);
    } else {
        fields.push(message_structure(0x0042_007C, Vec::new())?);
    }
    if let Some(extension) = message_batch_extension(target, elements)? {
        fields.push(extension);
    }
    message_structure(0x0042_000F, fields)
}

fn response_message_fixture(
    target: &Json,
    elements: &[Json],
    protocol_version: &str,
) -> Result<Item> {
    let version = message_protocol_version(target, elements, protocol_version)?;
    let nonce = message_nonce(target, elements)?;
    let response_header = message_structure(
        0x0042_007A,
        vec![
            version,
            message_scalar(0x0042_0092, Value::date_time(1), target, elements)?,
            nonce,
            message_scalar(
                0x0042_0155,
                Value::byte_string(vec![0x50]),
                target,
                elements,
            )?,
            message_scalar(0x0042_00C7, Value::enumeration(1), target, elements)?,
            message_scalar(
                0x0042_0105,
                Value::text_string("client-correlation".to_owned()),
                target,
                elements,
            )?,
            message_scalar(
                0x0042_0106,
                Value::text_string("server-correlation".to_owned()),
                target,
                elements,
            )?,
            message_item(0x0042_000D, Value::integer(1))?,
        ],
    )?;
    message_structure(
        0x0042_007B,
        vec![response_header, response_batch_item(target, elements)?],
    )
}

fn message_fixture_model(
    symbol: &str,
    target: &Json,
    elements: &[Json],
    protocol_version: &str,
) -> Result<Item> {
    match symbol {
        "kmipkit_protocol::RequestMessage::try_from_ttlv" => {
            request_message_fixture(target, elements, protocol_version)
        }
        "kmipkit_protocol::ResponseMessage::try_from_ttlv" => {
            response_message_fixture(target, elements, protocol_version)
        }
        _ => Err("unsupported message parser route"),
    }
}

#[cfg(test)]
fn sample_for_route(symbol: &str, target: &Json, elements: &[Json]) -> Result<Item> {
    sample_for_route_at_version(symbol, target, elements, "2.1")
}

#[allow(clippy::too_many_lines)] // Keep explicit public route-to-fixture dispatch auditable.
fn sample_for_route_at_version(
    symbol: &str,
    target: &Json,
    elements: &[Json],
    protocol_version: &str,
) -> Result<Item> {
    if [
        "kmipkit_protocol::RequestMessage::try_from_ttlv",
        "kmipkit_protocol::ResponseMessage::try_from_ttlv",
    ]
    .contains(&symbol)
    {
        let model = message_fixture_model(symbol, target, elements, protocol_version)?;
        return metadata_expected_target(&model, target, elements);
    }
    if KEY_WRAPPING_CONSTRUCTOR_ROUTES.contains(&symbol) {
        let model = key_wrapping_metadata_model(symbol, target)?;
        return metadata_expected_target(&model, target, elements);
    }
    if [
        "kmipkit_protocol::KeyValue::try_new",
        "kmipkit_protocol::KeyValue::try_from_item",
    ]
    .contains(&symbol)
    {
        let model = key_value_item_for_target(target)?;
        return metadata_expected_target(&model, target, elements);
    }
    if symbol == "kmipkit_protocol::KeyMaterial::byte_string" {
        let (format, _) = key_material_item_for_target(target, 1)?;
        let model = kmipkit::protocol::KeyMaterial::byte_string(format, vec![0x41; 32])
            .map(kmipkit::protocol::KeyMaterial::into_item)
            .map_err(|_| "byte-string Key Material fixture rejected")?;
        return metadata_expected_target(&model, target, elements);
    }
    if symbol == "kmipkit_protocol::KeyMaterial::structure" {
        let (_, model) = key_material_structure_item_for_target(target)?;
        return metadata_expected_target(&model, target, elements);
    }
    if symbol == "kmipkit_protocol::KeyMaterial::try_from_item" {
        let (_, model) = key_material_item_for_target(target, 1)?;
        return metadata_expected_target(&model, target, elements);
    }
    if symbol == "kmipkit_protocol::KeyBlock::try_from_item" {
        return key_block_fixture(target, elements).map(|(_, expected)| expected);
    }
    if symbol == "kmipkit_protocol::KeyBlock::try_new" {
        let model = construct_key_block_fixture(target, elements)?;
        return metadata_expected_target(&model, target, elements);
    }
    if [
        "kmipkit_protocol::KeyBlock::with_key_value",
        "kmipkit_protocol::KeyBlock::with_key_compression_type",
        "kmipkit_protocol::KeyBlock::with_key_wrapping_data",
    ]
    .contains(&symbol)
    {
        let model = construct_key_block_fixture(target, elements)?;
        return metadata_expected_target(&model, target, elements);
    }
    if symbol == "kmipkit_protocol::ManagedObject::try_from_item"
        || typed_object_root(symbol).is_some()
    {
        let (_, _, expected) = if let Some(root_name) = typed_object_root(symbol) {
            object_root_fixture(root_name, target, elements)?
        } else {
            managed_object_fixture(target, elements)?
        };
        return Ok(expected);
    }
    if let Some(root_name) = typed_object_constructor_root(symbol) {
        let (_, _, expected) = object_root_fixture(root_name, target, elements)?;
        return Ok(expected);
    }
    if [
        "kmipkit_protocol::EncryptionKeyInformation::try_from_item",
        "kmipkit_protocol::MacSignatureKeyInformation::try_from_item",
        "kmipkit_protocol::KeyWrappingData::try_from_item",
    ]
    .contains(&symbol)
    {
        let model = key_wrapping_metadata_model(symbol, target)?;
        return metadata_expected_target(&model, target, elements);
    }
    if symbol == "kmipkit_protocol::QueryResponse::to_ttlv_payload"
        && target["kind"] == "operation_payload_field"
        && target["name"] == "Profile Information"
    {
        let source = sample(target, elements)?;
        let profile_name = crate::driver::item(0x0042_00ec, Value::enumeration(0x0000_012a))?;
        let mut profile = kmipkit::ttlv::Structure::new();
        profile
            .try_push(profile_name)
            .map_err(|_| "Profile Information fixture rejected")?;
        return crate::driver::item(source.tag().raw(), Value::structure(profile));
    }
    if symbol == "kmipkit_protocol::ManagedObject::try_from_item"
        && target["kind"] == "object_structure"
    {
        return crate::fixtures::object(text(target, "name")?).map(|(_, item, _)| item);
    }
    if let Some(value_type) = symbol.strip_prefix("kmipkit_ttlv::Value::")
        && target["source_encoding"]
            .as_str()
            .is_some_and(|encoding| encoding.contains("Text String, Enumeration or Integer"))
    {
        let source = sample(target, elements)?;
        let value = unique_identifier_value_variant(value_type)?;
        return Item::new(source.tag(), value).map_err(|_| "Unique Identifier fixture rejected");
    }
    if !typed_item_route(symbol) {
        return sample(target, elements);
    }
    let marker = typed_attribute_marker(symbol).ok_or("typed attribute marker missing")?;
    let attribute = elements
        .iter()
        .find(|element| {
            element["kind"] == "attribute"
                && normalized_name(element["name"].as_str().unwrap_or_default())
                    == normalized_name(marker)
        })
        .ok_or("typed attribute definition missing")?;
    let attribute_id = text(attribute, "element_id")?;
    let tag = attribute["parent_element_ids"]
        .as_array()
        .and_then(|parents| parents.first())
        .and_then(Json::as_str)
        .and_then(|parent| parent.strip_prefix("KMIPKIT-ELEM-TAG-"))
        .and_then(|raw| u32::from_str_radix(raw, 16).ok())
        .ok_or("typed attribute tag missing")?;
    let value = if attribute["source_encoding"] == "Structure" {
        if normalized_name(marker) == "vendorattribute" {
            vendor_attribute_value()?
        } else {
            let mut fields = kmipkit::ttlv::Structure::new();
            for member in elements.iter().filter(|element| {
                element["kind"] == "structure_member"
                    && element["parent_element_ids"]
                        .as_array()
                        .is_some_and(|parents| parents.iter().any(|parent| parent == attribute_id))
            }) {
                fields
                    .try_push(sample(member, elements)?)
                    .map_err(|_| "typed attribute structure rejected")?;
            }
            Value::structure(fields)
        }
    } else {
        let source = if target["kind"] == "attribute" {
            sample(attribute, elements)?
        } else {
            sample(target, elements)?
        };
        source.with_value(value_to_owned)?
    };
    crate::driver::item(tag, value)
}

fn invoke_client_batch_route(symbol: &str, sample: &Item) -> Option<Result<Item>> {
    const CLIENT_BATCH_ROUTES: &[&str] = &[
        "kmipkit_client::ClientBatch::with_asynchronous_indicator",
        "kmipkit_client::ClientBatch::with_authentication",
        "kmipkit_client::ClientBatch::with_batch_error_continuation_option",
        "kmipkit_client::ClientBatch::with_batch_order_option",
        "kmipkit_client::ClientBatch::with_client_correlation_value",
        "kmipkit_client::ClientBatch::with_request_time_stamp",
        "kmipkit_client::ClientBatchItem::with_unique_batch_item_id",
    ];
    if !CLIENT_BATCH_ROUTES.contains(&symbol) {
        return None;
    }
    Some((|| {
        match symbol {
            "kmipkit_client::ClientBatch::with_asynchronous_indicator" => {
                let value = sample.with_value(|value| match value {
                    ValueView::Enumeration(value) => Ok(*value),
                    _ => Err("Asynchronous Indicator fixture is not an Enumeration"),
                })?;
                let _batch = kmipkit::ClientBatch::empty().with_asynchronous_indicator(value);
            }
            "kmipkit_client::ClientBatch::with_authentication" => {
                let structure = sample
                    .as_structure()
                    .ok_or("Authentication fixture is not a Structure")?
                    .try_clone()
                    .map_err(|_| "Authentication fixture copy failed")?;
                let authentication = kmipkit::protocol::Authentication::try_from_ttlv(structure)
                    .map_err(|_| "Authentication fixture rejected")?;
                let _batch = kmipkit::ClientBatch::empty().with_authentication(authentication);
            }
            "kmipkit_client::ClientBatch::with_batch_error_continuation_option" => {
                let value = sample.with_value(|value| match value {
                    ValueView::Enumeration(value) => Ok(*value),
                    _ => Err("Batch Error Continuation Option fixture is not an Enumeration"),
                })?;
                let _batch =
                    kmipkit::ClientBatch::empty().with_batch_error_continuation_option(value);
            }
            "kmipkit_client::ClientBatch::with_batch_order_option" => {
                let value = sample.with_value(|value| match value {
                    ValueView::Boolean(value) => Ok(*value),
                    _ => Err("Batch Order Option fixture is not a Boolean"),
                })?;
                let _batch = kmipkit::ClientBatch::empty().with_batch_order_option(value);
            }
            "kmipkit_client::ClientBatch::with_client_correlation_value" => {
                let value = sample.with_value(|value| match value {
                    ValueView::TextString(value) => Ok(value.to_owned()),
                    _ => Err("Client Correlation Value fixture is not a Text String"),
                })?;
                let _batch = kmipkit::ClientBatch::empty().with_client_correlation_value(value);
            }
            "kmipkit_client::ClientBatch::with_request_time_stamp" => {
                let value = sample.with_value(|value| match value {
                    ValueView::DateTime(value) => Ok(*value),
                    _ => Err("Request Time Stamp fixture is not a Date-Time"),
                })?;
                let _batch = kmipkit::ClientBatch::empty().with_request_time_stamp(value);
            }
            "kmipkit_client::ClientBatchItem::with_unique_batch_item_id" => {
                let value = sample.with_value(|value| match value {
                    ValueView::ByteString(value) => Ok(value.to_vec()),
                    _ => Err("Unique Batch Item ID fixture is not a Byte String"),
                })?;
                let item =
                    kmipkit::ClientBatchItem::ping().with_unique_batch_item_id(value.clone());
                if item.unique_batch_item_id() != Some(value.as_slice()) {
                    return Err("Unique Batch Item ID was not preserved");
                }
            }
            _ => return Err("unsupported ClientBatch route"),
        }
        sample.try_clone().map_err(|_| "local copy failed")
    })())
}

#[allow(clippy::too_many_lines)] // Explicit public route dispatch stays together for review.
fn invoke(
    symbol: &str,
    sample: &Item,
    target: &Json,
    elements: &[Json],
    protocol_version: &str,
) -> Option<Result<Item>> {
    if let Some(result) = invoke_client_batch_route(symbol, sample) {
        return Some(result);
    }
    if let Some(result) = crate::local_credentials::invoke(symbol, sample, target, elements) {
        return Some(result);
    }
    if [
        "kmipkit_protocol::RequestMessage::try_from_ttlv",
        "kmipkit_protocol::ResponseMessage::try_from_ttlv",
    ]
    .contains(&symbol)
    {
        return Some((|| {
            let model = message_fixture_model(symbol, target, elements, protocol_version)?;
            let structure = model
                .as_structure()
                .ok_or("message fixture root is not a Structure")?
                .try_clone()
                .map_err(|_| "message fixture copy failed")?;
            let (tag, tree) = match symbol {
                "kmipkit_protocol::RequestMessage::try_from_ttlv" => (
                    0x0042_0078,
                    kmipkit::protocol::RequestMessage::try_from_ttlv(structure)
                        .map(kmipkit::protocol::RequestMessage::into_ttlv)
                        .map_err(|_| "Request Message parser rejected"),
                ),
                "kmipkit_protocol::ResponseMessage::try_from_ttlv" => (
                    0x0042_007B,
                    kmipkit::protocol::ResponseMessage::try_from_ttlv(structure)
                        .map(kmipkit::protocol::ResponseMessage::into_ttlv)
                        .map_err(|_| "Response Message parser rejected"),
                ),
                _ => return Err("unsupported message parser route"),
            };
            message_item(tag, Value::structure(tree?))
        })());
    }
    if [
        "kmipkit_protocol::GetRequest::with_key_format_type",
        "kmipkit_protocol::ExportRequest::with_key_format_type",
    ]
    .contains(&symbol)
    {
        return Some((|| {
            let raw = sample.with_value(|value| match value {
                ValueView::Enumeration(value) => Ok(*value),
                _ => Err("Key Format Type fixture is not an Enumeration"),
            })?;
            let payload = key_format_type_request_payload(symbol, raw)?;
            request_payload_item(payload)
        })());
    }
    if symbol.ends_with("::with_unique_identifier") {
        return Some((|| {
            let identifier = sample.with_value(|value| match value {
                ValueView::TextString(value) => Ok(
                    kmipkit::protocol::UniqueIdentifier::TextString(value.to_owned()),
                ),
                ValueView::Integer(value) => {
                    Ok(kmipkit::protocol::UniqueIdentifier::Integer(*value))
                }
                ValueView::Enumeration(value) => {
                    Ok(kmipkit::protocol::UniqueIdentifier::Enumeration(*value))
                }
                _ => Err("Unique Identifier fixture has an unsupported wire type"),
            })?;
            let payload = unique_identifier_request_payload(symbol, identifier)?;
            request_payload_item(payload)
        })());
    }
    if KEY_WRAPPING_CONSTRUCTOR_ROUTES.contains(&symbol) {
        return Some(
            key_wrapping_metadata_model(symbol, target).and_then(|model| {
                crate::local_wire::roundtrip(&model)?;
                metadata_expected_target(&model, target, elements)
            }),
        );
    }
    if symbol == "kmipkit_protocol::KeyValue::try_new" {
        return Some(key_value_item_for_target(target).and_then(|model| {
            crate::local_wire::roundtrip(&model)?;
            metadata_expected_target(&model, target, elements)
        }));
    }
    if symbol == "kmipkit_protocol::KeyValue::try_from_item" {
        return Some(key_value_item_for_target(target).and_then(|item| {
            let format = key_material_format_for_target(target).unwrap_or(1);
            kmipkit::protocol::KeyValue::try_from_item(
                item,
                kmipkit::protocol::KeyFormatType::from_raw(format),
            )
            .map(kmipkit::protocol::KeyValue::into_item)
            .map_err(|_| "Key Value parser rejected")
            .and_then(|model| {
                crate::local_wire::roundtrip(&model)?;
                metadata_expected_target(&model, target, elements)
            })
        }));
    }
    if symbol == "kmipkit_protocol::KeyMaterial::byte_string" {
        let format = key_material_format_for_target(target).unwrap_or(1);
        return Some(
            kmipkit::protocol::KeyMaterial::byte_string(format, vec![0x41; 32])
                .map(kmipkit::protocol::KeyMaterial::into_item)
                .map_err(|_| "byte-string Key Material constructor rejected")
                .and_then(|model| {
                    crate::local_wire::roundtrip(&model)?;
                    metadata_expected_target(&model, target, elements)
                }),
        );
    }
    if symbol == "kmipkit_protocol::KeyMaterial::structure" {
        return Some(
            key_material_structure_item_for_target(target).and_then(|(_, model)| {
                crate::local_wire::roundtrip(&model)?;
                metadata_expected_target(&model, target, elements)
            }),
        );
    }
    if symbol == "kmipkit_protocol::KeyMaterial::try_from_item" {
        return Some(
            key_material_item_for_target(target, 1).and_then(|(format, item)| {
                kmipkit::protocol::KeyMaterial::try_from_item(
                    item,
                    kmipkit::protocol::KeyFormatType::from_raw(format),
                )
                .map(kmipkit::protocol::KeyMaterial::into_item)
                .map_err(|_| "Key Material parser rejected")
                .and_then(|model| {
                    crate::local_wire::roundtrip(&model)?;
                    metadata_expected_target(&model, target, elements)
                })
            }),
        );
    }
    if symbol == "kmipkit_protocol::ManagedObject::try_from_item" {
        let (object_type, object, _) = managed_object_fixture(target, elements).ok()?;
        return Some(
            kmipkit::protocol::ManagedObject::try_from_item(
                kmipkit::protocol::ObjectType::from_raw(object_type),
                object,
            )
            .map(kmipkit::protocol::ManagedObject::into_item)
            .map_err(|_| "managed object parser rejected"),
        );
    }
    if symbol == "kmipkit_protocol::KeyBlock::try_from_item" {
        let (key_block, _) = key_block_fixture(target, elements).ok()?;
        return Some(
            kmipkit::protocol::KeyBlock::try_from_item(key_block)
                .and_then(kmipkit::protocol::KeyBlock::try_into_item)
                .map_err(|_| "Key Block parser rejected")
                .and_then(|model| {
                    crate::local_wire::roundtrip(&model)?;
                    metadata_expected_target(&model, target, elements)
                }),
        );
    }
    if symbol == "kmipkit_protocol::KeyBlock::try_new" {
        return Some(
            construct_key_block_fixture(target, elements).and_then(|model| {
                crate::local_wire::roundtrip(&model)?;
                metadata_expected_target(&model, target, elements)
            }),
        );
    }
    if [
        "kmipkit_protocol::KeyBlock::with_key_value",
        "kmipkit_protocol::KeyBlock::with_key_compression_type",
        "kmipkit_protocol::KeyBlock::with_key_wrapping_data",
    ]
    .contains(&symbol)
    {
        return Some(
            construct_key_block_fixture(target, elements).and_then(|model| {
                crate::local_wire::roundtrip(&model)?;
                metadata_expected_target(&model, target, elements)
            }),
        );
    }
    if let Some(result) = construct_typed_object(symbol, target) {
        return Some(result.and_then(|object| metadata_expected_target(&object, target, elements)));
    }
    if let Some(root) = typed_object_root(symbol) {
        let (_, object, _) = object_root_fixture(root, target, elements).ok()?;
        return invoke_typed_object_parser(symbol, object).map(|result| {
            result.and_then(|model| {
                crate::local_wire::roundtrip(&model)?;
                metadata_expected_target(&model, target, elements)
            })
        });
    }
    if [
        "kmipkit_protocol::EncryptionKeyInformation::try_from_item",
        "kmipkit_protocol::MacSignatureKeyInformation::try_from_item",
        "kmipkit_protocol::KeyWrappingData::try_from_item",
    ]
    .contains(&symbol)
    {
        let item = key_wrapping_metadata_model(symbol, target).ok()?;
        let result = match symbol {
            "kmipkit_protocol::EncryptionKeyInformation::try_from_item" => {
                kmipkit::protocol::EncryptionKeyInformation::try_from_item(item)
                    .map(kmipkit::protocol::EncryptionKeyInformation::into_item)
            }
            "kmipkit_protocol::MacSignatureKeyInformation::try_from_item" => {
                kmipkit::protocol::MacSignatureKeyInformation::try_from_item(item)
                    .map(kmipkit::protocol::MacSignatureKeyInformation::into_item)
            }
            "kmipkit_protocol::KeyWrappingData::try_from_item" => {
                kmipkit::protocol::KeyWrappingData::try_from_item(item)
                    .and_then(kmipkit::protocol::KeyWrappingData::into_item)
            }
            _ => return None,
        }
        .map_err(|_| "key wrapping metadata parser rejected");
        return Some(result);
    }
    if symbol.starts_with("kmipkit_ttlv::Value::") {
        let expected = match sample.item_type() {
            kmipkit::ttlv::ItemType::Structure => "structure",
            kmipkit::ttlv::ItemType::Integer => "integer",
            kmipkit::ttlv::ItemType::LongInteger => "long_integer",
            kmipkit::ttlv::ItemType::BigInteger => "big_integer",
            kmipkit::ttlv::ItemType::Enumeration => "enumeration",
            kmipkit::ttlv::ItemType::Boolean => "boolean",
            kmipkit::ttlv::ItemType::TextString => "text_string",
            kmipkit::ttlv::ItemType::ByteString => "byte_string",
            kmipkit::ttlv::ItemType::DateTime => "date_time",
            kmipkit::ttlv::ItemType::Interval => "interval",
            kmipkit::ttlv::ItemType::DateTimeExtended => "date_time_extended",
            _ => return None,
        };
        if symbol.rsplit("::").next() != Some(expected) {
            return None;
        }
        return Some(
            sample.with_value(value_to_owned).and_then(|value| {
                Item::new(sample.tag(), value).map_err(|_| "local value rejected")
            }),
        );
    }
    if [
        "kmipkit_ttlv::RawTag::new",
        "kmipkit_ttlv::RawTag::try_checked",
    ]
    .contains(&symbol)
    {
        return Some((|| {
            let raw =
                kmipkit::ttlv::RawTag::new(sample.tag().raw()).map_err(|_| "raw tag rejected")?;
            let checked = raw.try_checked().map_err(|_| "checked tag rejected")?;
            Item::new(checked, sample.with_value(value_to_owned)?)
                .map_err(|_| "local item rejected")
        })());
    }
    if [
        "kmipkit_ttlv::Structure::new",
        "kmipkit_ttlv::Structure::try_push",
    ]
    .contains(&symbol)
    {
        if symbol == "kmipkit_ttlv::Structure::new" && sample.as_structure().is_none() {
            return Some((|| {
                let mut structure = kmipkit::ttlv::Structure::new();
                structure
                    .try_push(sample.try_clone().map_err(|_| "local copy failed")?)
                    .map_err(|_| "structure insertion rejected")?;
                let container_tag = parent_structure_tag(target, elements).unwrap_or(0x0042_0079);
                crate::driver::item(container_tag, Value::structure(structure))
            })());
        }
        if symbol.ends_with("::try_push")
            && sample
                .as_structure()
                .is_none_or(|children| children.children().is_empty())
        {
            return Some((|| {
                let mut structure = kmipkit::ttlv::Structure::new();
                structure
                    .try_push(sample.try_clone().map_err(|_| "local copy failed")?)
                    .map_err(|_| "structure insertion rejected")?;
                // A Request Payload is the stable generic envelope for top-level structures.
                let container_tag = parent_structure_tag(target, elements).unwrap_or(0x0042_0079);
                crate::driver::item(container_tag, Value::structure(structure))
            })());
        }
        let children = sample.as_structure()?;
        return Some((|| {
            let mut structure = kmipkit::ttlv::Structure::new();
            for child in children.children() {
                structure
                    .try_push(child.try_clone().map_err(|_| "local copy failed")?)
                    .map_err(|_| "structure insertion rejected")?;
            }
            crate::driver::item(sample.tag().raw(), Value::structure(structure))
        })());
    }
    if symbol.contains("typed::CryptographicParameters>") {
        let context = crate::local_models::parameter_context(sample).ok()?;
        return generated::execute(symbol, &context);
    }
    if let Some(result) = crate::local_models::invoke(symbol, sample, protocol_version) {
        return Some(result);
    }
    if let Some(result) = generated::execute(symbol, sample) {
        return Some(result);
    }
    if ![
        "kmipkit_ttlv::Item::new",
        "kmipkit_ttlv::codec::decode",
        "kmipkit_ttlv::Item::with_value",
        "kmipkit_protocol::AttributeSet::try_new",
        "kmipkit_protocol::AttributeSet::try_push",
        "kmipkit_protocol::CurrentAttribute::new",
        "kmipkit_protocol::NewAttribute::new",
    ]
    .contains(&symbol)
    {
        return None;
    }
    Some((|| match symbol {
        "kmipkit_ttlv::Item::new" => {
            let tag = sample.tag();
            let value = sample.with_value(value_to_owned)?;
            Item::new(tag, value).map_err(|_| "local item rejected")
        }
        "kmipkit_ttlv::codec::decode" => {
            kmipkit::ttlv::codec::decode(&crate::local_wire::encode(sample)?)
                .map_err(|_| "local decode rejected")
        }
        "kmipkit_ttlv::Item::with_value" => sample
            .with_value(value_to_owned)
            .and_then(|v| Item::new(sample.tag(), v).map_err(|_| "local item rejected")),
        "kmipkit_protocol::AttributeSet::try_new" => {
            let set = kmipkit::protocol::AttributeSet::try_new([sample
                .try_clone()
                .map_err(|_| "local copy failed")?])
            .map_err(|_| "attribute set rejected")?;
            set.as_items()[0]
                .try_clone()
                .map_err(|_| "local copy failed")
        }
        "kmipkit_protocol::AttributeSet::try_push" => {
            let mut set = kmipkit::protocol::AttributeSet::new();
            set.try_push(sample.try_clone().map_err(|_| "local copy failed")?)
                .map_err(|_| "attribute set rejected")?;
            set.as_items()[0]
                .try_clone()
                .map_err(|_| "local copy failed")
        }
        "kmipkit_protocol::CurrentAttribute::new" => kmipkit::protocol::CurrentAttribute::new(
            sample.try_clone().map_err(|_| "local copy failed")?,
        )
        .item()
        .try_clone()
        .map_err(|_| "local copy failed"),
        "kmipkit_protocol::NewAttribute::new" => kmipkit::protocol::NewAttribute::new(
            sample.try_clone().map_err(|_| "local copy failed")?,
        )
        .item()
        .try_clone()
        .map_err(|_| "local copy failed"),
        _ => Err("pending local recipe"),
    })())
}

fn value_to_owned(value: ValueView<'_>) -> Result<Value> {
    Ok(match value {
        ValueView::Structure(v) => {
            Value::structure(v.try_clone().map_err(|_| "local copy failed")?)
        }
        ValueView::Integer(v) => Value::integer(*v),
        ValueView::LongInteger(v) => Value::long_integer(*v),
        ValueView::BigInteger(v) => Value::big_integer(v.to_vec()),
        ValueView::Enumeration(v) => Value::enumeration(*v),
        ValueView::Boolean(v) => Value::boolean(*v),
        ValueView::TextString(v) => Value::text_string(v.to_owned()),
        ValueView::ByteString(v) => Value::byte_string(v.to_vec()),
        ValueView::DateTime(v) => Value::date_time(*v),
        ValueView::Interval(v) => Value::interval(*v),
        ValueView::DateTimeExtended(v) => Value::date_time_extended(*v),
        _ => return Err("unknown local value type"),
    })
}

fn expected_tag_decode_rejection(
    row: &Json,
    target: &Json,
    catalog: &Json,
) -> Option<(u32, &'static str)> {
    if row["action"] != "decode" || !row["route_id"].as_str()?.contains("generic-ttlv") {
        return None;
    }
    let symbol: String = text(row, "public_api_symbol")
        .ok()?
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .flat_map(char::to_lowercase)
        .collect();
    if !(symbol.contains("codecdecode") || symbol.ends_with("ttlvdecode")) {
        return None;
    }
    let _case = catalog["test_cases"]
        .as_array()?
        .iter()
        .find(|case| case["case_id"] == row["case_id"])?;
    // An enum inventory case can expand to tag-range targets even though its own `requires`
    // block describes the enum. Use the resolved normative target's allocation here.
    if !["tag", "tag_range"].contains(&target["kind"].as_str()?) {
        return None;
    }
    let rejection_code = match target["allocation"].as_str()? {
        "reserved" => "tag_reserved",
        "unused" => "tag_not_allocated",
        _ => return None,
    };
    Some((target_wire_start(target)?, rejection_code))
}

fn target_wire_start(target: &Json) -> Option<u32> {
    wire_range_start(text(target, "wire_value").ok()?)
}

fn raw_tag_route_event(row: &Json, target: &Json, invocation: usize) -> Option<Json> {
    let symbol = row["public_api_symbol"].as_str()?;
    if ![
        "kmipkit_ttlv::RawTag::new",
        "kmipkit_ttlv::RawTag::try_checked",
    ]
    .contains(&symbol)
        || target["kind"] != "tag"
    {
        return None;
    }
    let raw = target_wire_start(target)?;
    let tag = kmipkit::ttlv::RawTag::new(raw).ok()?;
    let (expected, result_value) = if symbol == "kmipkit_ttlv::RawTag::new" {
        (true, json!(["RawTag", tag.raw()]))
    } else {
        let should_be_checked = matches!(target["allocation"].as_str()?, "assigned" | "extension");
        let checked = tag.try_checked();
        let matched = match checked {
            Ok(checked) => should_be_checked && checked.raw() == raw,
            Err(_) => !should_be_checked,
        };
        let value_kind = if should_be_checked { "Tag" } else { "rejected" };
        (
            matched,
            json!(["RawTag::try_checked", value_kind, tag.raw()]),
        )
    };
    let digest = hash(&serde_json::to_vec(&result_value).ok()?);
    let mut event = json!({
        "case_id": row["case_id"], "route_id": row["route_id"], "target_id": row["target_id"],
        "public_api_symbol": symbol, "execution_route_id": null, "execution_api_symbol": null,
        "invocation_id": format!("rust-local-{invocation}"), "action": row["action"],
        "protocol_version": row["protocol_version"], "encoding": "TTLV", "transport": null,
        "status": if expected { "passed" } else { "failed" }, "invocation_count": 1,
        "request_tree_sha256": digest
    });
    for field in [
        "workflow_step_index",
        "workflow_operation",
        "workflow_item_index",
    ] {
        if let Some(value) = row.get(field) {
            event[field] = value.clone();
        }
    }
    if !expected {
        event["failure_code"] = "tag_allocation_result_mismatch".into();
    }
    Some(event)
}

fn reserved_tag_ttlv(raw_tag: u32) -> [u8; 16] {
    let tag = raw_tag.to_be_bytes();
    [
        tag[1], tag[2], tag[3], 5, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0,
    ]
}

#[allow(clippy::too_many_lines)] // Keep row validation and invocation bookkeeping adjacent for auditing.
pub(crate) fn run(matrix: &Path, output: &Path) -> Result<bool> {
    let catalog_bytes = include_bytes!("../../shared/catalog/kmipkit-1.0-test-catalog.json");
    let routes_bytes = include_bytes!("../../shared/coverage/route-capabilities.json");
    let catalog: Json = serde_json::from_slice(catalog_bytes).map_err(|_| "catalog invalid")?;
    let manifest: Json =
        serde_json::from_slice(routes_bytes).map_err(|_| "route manifest invalid")?;
    if manifest["library_commit"] != env!("KMIPKIT_LIBRARY_COMMIT") {
        return Err("route artifact commit mismatch");
    }
    let rows: Vec<Json> =
        BufReader::new(std::fs::File::open(matrix).map_err(|_| "matrix unavailable")?)
            .lines()
            .map(|line| {
                serde_json::from_str(&line.map_err(|_| "matrix read failed")?)
                    .map_err(|_| "matrix row invalid")
            })
            .collect::<Result<_>>()?;
    let canonical_rows: HashSet<_> =
        include_str!(concat!(env!("OUT_DIR"), "/route-cell-hashes.txt"))
            .lines()
            .collect();
    let mut seen = HashSet::new();
    for row in &rows {
        if row["language"] != "rust" || row["mode"] != "local" {
            return Err("local matrix requires Rust local cells");
        }
        let canonical = serde_json::to_vec(row).map_err(|_| "matrix normalization failed")?;
        if !canonical_rows.contains(hash(&canonical).as_str()) {
            return Err("matrix row does not match canonical shared obligations");
        }
        let key = json!([
            row["case_id"],
            row["route_id"],
            row["target_id"],
            row["workflow_step_index"],
            row["workflow_item_index"],
            row["protocol_version"],
            row["action"]
        ])
        .to_string();
        if !seen.insert(key) {
            return Err("duplicate matrix cell");
        }
    }
    let elements: Json =
        serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
            .map_err(|_| "invalid target metadata")?;
    let elements = elements["elements"]
        .as_array()
        .ok_or("invalid target metadata")?;
    let mut events = Vec::new();
    let mut versions = std::collections::BTreeSet::new();
    let mut pending = 0;
    let mut pending_routes = std::collections::BTreeMap::<(String, String), usize>::new();
    for row in &rows {
        let target = catalog["normative_targets"]
            .as_array()
            .ok_or("catalog invalid")?
            .iter()
            .find(|t| t["target_id"] == row["target_id"])
            .ok_or("unknown normative target")?;
        let symbol = text(row, "public_api_symbol")?;
        versions.insert(text(row, "protocol_version")?.to_owned());
        if let Some((raw_tag, rejection_code)) =
            expected_tag_decode_rejection(row, target, &catalog)
        {
            let wire = reserved_tag_ttlv(raw_tag);
            let result = kmipkit::ttlv::codec::decode(&wire);
            let rejected = matches!(result, Err(error) if matches!(
                error.kind(), DecodeErrorKind::ReservedTag | DecodeErrorKind::UnallocatedTag));
            let mut event = json!({"case_id":row["case_id"],"route_id":row["route_id"],
                "target_id":row["target_id"],"public_api_symbol":symbol,
                "execution_route_id":null,"execution_api_symbol":null,
                "invocation_id":format!("rust-local-{}",events.len()),"action":"decode",
                "protocol_version":row["protocol_version"],"encoding":"TTLV","transport":null,
                "status":if rejected {"passed"} else {"failed"},"invocation_count":1,
                "decode_outcome":"rejected","decode_input_sha256":hash(&wire)});
            if rejected {
                event["decode_rejection_code"] = rejection_code.into();
            } else {
                event["failure_code"] = "expected_tag_rejection_not_observed".into();
            }
            events.push(event);
            continue;
        }
        if let Some(event) = raw_tag_route_event(row, target, events.len()) {
            events.push(event);
            continue;
        }
        let sample = match sample_for_route_at_version(
            symbol,
            target,
            elements,
            text(row, "protocol_version")?,
        ) {
            Ok(sample) => sample,
            Err(error) => {
                pending += 1;
                *pending_routes
                    .entry((symbol.to_owned(), error.to_owned()))
                    .or_default() += 1;
                continue;
            }
        };
        let Some(result) = invoke(
            symbol,
            &sample,
            target,
            elements,
            text(row, "protocol_version")?,
        ) else {
            pending += 1;
            *pending_routes
                .entry((symbol.to_owned(), "no invocation route".to_owned()))
                .or_default() += 1;
            continue;
        };
        let result = result.and_then(|item| crate::local_wire::target_hash(&item, &sample));
        let mut event = json!({"case_id":row["case_id"],"route_id":row["route_id"],"target_id":row["target_id"],
            "public_api_symbol":symbol,"execution_route_id":null,"execution_api_symbol":null,
            "invocation_id":format!("rust-local-{}",events.len()),"action":row["action"],
            "protocol_version":row["protocol_version"],"encoding":"TTLV","transport":null,
            "status":if result.is_ok() {"passed"} else {"failed"},"invocation_count":1});
        for field in [
            "workflow_step_index",
            "workflow_operation",
            "workflow_item_index",
        ] {
            if let Some(value) = row.get(field) {
                event[field] = value.clone();
            }
        }
        if let Ok(hash) = result {
            event[if row["action"] == "decode" {
                "response_tree_sha256"
            } else {
                "request_tree_sha256"
            }] = hash.into();
        } else {
            event["failure_code"] = "local_roundtrip_failed".into();
        }
        events.push(event);
    }
    let success = pending == 0 && events.iter().all(|event| event["status"] == "passed");
    let report = json!({"schema_version":"1.0","run_id":format!("rust-local-{}",uuid::Uuid::new_v4()),"language":"rust",
        "server":{"id":"controlled-fixture","product":"controlled-fixture","version":"offline-1","configuration_fingerprint":hash(b"local-model")},
        "library_commit":env!("KMIPKIT_LIBRARY_COMMIT"),"artifact_sha256":env!("KMIPKIT_ARTIFACT_SHA256"),
        "catalog_sha256":hash(catalog_bytes),"route_manifest_sha256":hash(routes_bytes),
        "protocol_versions":versions,"network_combinations":[{"encoding":"TTLV","transport":"raw_tls"}],"events":events});
    std::fs::write(
        output,
        serde_json::to_vec_pretty(&report).map_err(|_| "matrix report failed")?,
    )
    .map_err(|_| "matrix output unavailable")?;
    if pending > 0 {
        eprintln!("pending_recipe: {pending} matrix cells have no executable Rust recipe");
        for ((symbol, reason), count) in pending_routes {
            eprintln!("pending_recipe_detail: {count} {symbol}: {reason}");
        }
    }
    Ok(success)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key_material_target(catalog: &Json) -> &Json {
        catalog["normative_targets"]
            .as_array()
            .unwrap()
            .iter()
            .find(|target| target["target_id"] == "KMIPKIT-ELEM-OBJECT-STRUCTURE-3-10-KEY-MATERIAL")
            .unwrap()
    }

    #[test]
    fn key_block_parser_route_roundtrips_the_normative_key_material_target() {
        // OASIS KMIP 2.0/2.1 §3.10 and §3.11 define Key Material and Key Block.
        let catalog: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let specification: Json =
            serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
                .unwrap();
        let elements = specification["elements"].as_array().unwrap();
        let target = key_material_target(&catalog);
        let symbol = "kmipkit_protocol::KeyBlock::try_from_item";
        let expected = sample_for_route(symbol, target, elements).unwrap();
        let result = invoke(symbol, &expected, target, elements, "2.1");
        assert!(result.is_some(), "KeyBlock parser route is not executable");
        let result = result.unwrap().unwrap();
        assert!(crate::local_wire::target_hash(&result, &expected).is_ok());
    }

    #[test]
    fn generic_structure_constructor_wraps_a_primitive_normative_target() {
        let catalog: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let specification: Json =
            serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
                .unwrap();
        let elements = specification["elements"].as_array().unwrap();
        let target = catalog["normative_targets"]
            .as_array()
            .unwrap()
            .iter()
            .find(|target| {
                target["target_id"] == "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-10-DATA-LENGTH"
            })
            .unwrap();
        let symbol = "kmipkit_ttlv::Structure::new";
        let expected = sample_for_route(symbol, target, elements).unwrap();
        assert_ne!(expected.item_type(), kmipkit::ttlv::ItemType::Structure);

        let result = invoke(symbol, &expected, target, elements, "2.1")
            .expect("Structure::new route is executable")
            .expect("Structure::new accepts the fixture child");
        assert!(crate::local_wire::target_hash(&result, &expected).is_ok());
    }

    #[test]
    fn client_batch_builder_routes_invoke_the_named_public_methods() {
        let catalog: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let specification: Json =
            serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
                .unwrap();
        let elements = specification["elements"].as_array().unwrap();
        let routes = [
            (
                "kmipkit_client::ClientBatch::with_asynchronous_indicator",
                "KMIPKIT-ELEM-ENUM-VALUE-ASYNCHRONOUS-INDICATOR-EXTENSIONS-8XXXXXXX",
            ),
            (
                "kmipkit_client::ClientBatch::with_authentication",
                "KMIPKIT-ELEM-MESSAGE-FIELD-8-2-AUTHENTICATION",
            ),
            (
                "kmipkit_client::ClientBatch::with_batch_error_continuation_option",
                "KMIPKIT-ELEM-ENUM-VALUE-BATCH-ERROR-CONTINUATION-OPTION-CONTINUE-00000001",
            ),
            (
                "kmipkit_client::ClientBatch::with_batch_order_option",
                "KMIPKIT-ELEM-MESSAGE-FIELD-8-2-BATCH-ORDER-OPTION",
            ),
            (
                "kmipkit_client::ClientBatch::with_client_correlation_value",
                "KMIPKIT-ELEM-MESSAGE-FIELD-8-2-CLIENT-CORRELATION-VALUE",
            ),
            (
                "kmipkit_client::ClientBatch::with_request_time_stamp",
                "KMIPKIT-ELEM-MESSAGE-FIELD-8-2-TIME-STAMP",
            ),
            (
                "kmipkit_client::ClientBatchItem::with_unique_batch_item_id",
                "KMIPKIT-ELEM-MESSAGE-FIELD-8-3-UNIQUE-BATCH-ITEM-ID",
            ),
        ];

        for (symbol, target_id) in routes {
            let target = catalog["normative_targets"]
                .as_array()
                .unwrap()
                .iter()
                .find(|target| target["target_id"] == target_id)
                .unwrap();
            let expected = sample_for_route_at_version(symbol, target, elements, "2.1").unwrap();
            let result = invoke(symbol, &expected, target, elements, "2.1")
                .unwrap_or_else(|| panic!("{symbol} route is not executable"))
                .unwrap_or_else(|error| panic!("{symbol} rejected its fixture: {error}"));
            assert!(
                crate::local_wire::target_hash(&result, &expected).is_ok(),
                "{symbol} changed the normative value"
            );
        }
    }

    #[test]
    fn attribute_reference_constructors_execute_name_and_raw_tag_forms() {
        let catalog: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let specification: Json =
            serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
                .unwrap();
        let elements = specification["elements"].as_array().unwrap();
        let target = catalog["normative_targets"]
            .as_array()
            .unwrap()
            .iter()
            .find(|target| {
                target["target_id"] == "KMIPKIT-ELEM-ATTRIBUTE-STRUCTURE-5-5-ATTRIBUTE-REFERENCE"
            })
            .unwrap();

        for symbol in [
            "kmipkit_protocol::AttributeReference::name",
            "kmipkit_protocol::AttributeReference::tag",
        ] {
            let expected = sample_for_route_at_version(symbol, target, elements, "2.1").unwrap();
            let result = invoke(symbol, &expected, target, elements, "2.1")
                .unwrap_or_else(|| panic!("{symbol} route is not executable"))
                .unwrap_or_else(|error| panic!("{symbol} rejected its fixture: {error}"));
            assert!(
                crate::local_wire::target_hash(&result, &expected).is_ok(),
                "{symbol} changed the normative value"
            );
        }
    }

    #[test]
    fn transparent_key_material_fixture_uses_its_normative_structure_type() {
        let catalog: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let specification: Json =
            serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
                .unwrap();
        let elements = specification["elements"].as_array().unwrap();
        let target = catalog["normative_targets"]
            .as_array()
            .unwrap()
            .iter()
            .find(|target| target["target_id"] == "KMIPKIT-ELEM-OBJECT-STRUCTURE-3-4-KEY-MATERIAL")
            .unwrap();
        let (_, _, expected) = object_root_fixture("PGP Key", target, elements).unwrap();
        assert_eq!(expected.item_type(), kmipkit::ttlv::ItemType::Structure);
    }

    #[test]
    fn key_block_constructor_route_builds_the_normative_key_material_target() {
        // OASIS KMIP 2.0/2.1 §3.10 and §3.11 define Key Material and Key Block.
        let catalog: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let specification: Json =
            serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
                .unwrap();
        let elements = specification["elements"].as_array().unwrap();
        let target = key_material_target(&catalog);
        let symbol = "kmipkit_protocol::KeyBlock::try_new";
        let expected = sample_for_route(symbol, target, elements).unwrap();
        let result = invoke(symbol, &expected, target, elements, "2.1");
        assert!(
            result.is_some(),
            "KeyBlock constructor route is not executable"
        );
        let result = result.unwrap().unwrap();
        assert!(crate::local_wire::target_hash(&result, &expected).is_ok());
    }

    #[test]
    fn key_block_constructor_preserves_secret_data_key_block_shape() {
        // OASIS KMIP 2.0/2.1 §2.7 and §3.1 define Secret Data and Key Block.
        let catalog: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let specification: Json =
            serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
                .unwrap();
        let elements = specification["elements"].as_array().unwrap();
        let target = catalog["normative_targets"]
            .as_array()
            .unwrap()
            .iter()
            .find(|target| {
                target["target_id"] == "KMIPKIT-ELEM-STRUCTURE-MEMBER-2-7-SECRET-DATA-KEY-BLOCK"
            })
            .unwrap();
        let symbol = "kmipkit_protocol::KeyBlock::try_new";
        let expected = sample_for_route(symbol, target, elements).unwrap();
        let result = invoke(symbol, &expected, target, elements, "2.1")
            .unwrap()
            .unwrap();
        assert!(crate::local_wire::target_hash(&result, &expected).is_ok());
    }

    /// OASIS KMIP 2.0/2.1 §3.2, Tables 13–14 define Key Value and Key Material.
    #[test]
    fn key_block_key_value_setter_executes_every_normative_target() {
        let catalog: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let manifest: Json = serde_json::from_str(include_str!(
            "../../shared/coverage/route-capabilities.json"
        ))
        .unwrap();
        let specification: Json =
            serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
                .unwrap();
        let elements = specification["elements"].as_array().unwrap();
        let route = manifest["routes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|route| {
                route["public_api_symbol"] == "kmipkit_protocol::KeyBlock::with_key_value"
            })
            .unwrap();
        for target_id in route["normative_target_ids"].as_array().unwrap() {
            let target_id = target_id.as_str().unwrap();
            let target = catalog["normative_targets"]
                .as_array()
                .unwrap()
                .iter()
                .find(|target| target["target_id"] == target_id)
                .unwrap();
            let fixture = sample_for_route(
                "kmipkit_protocol::KeyBlock::with_key_value",
                target,
                elements,
            );
            assert!(fixture.is_ok(), "{target_id}: {fixture:?}");
            let fixture = fixture.unwrap();
            let result = invoke(
                "kmipkit_protocol::KeyBlock::with_key_value",
                &fixture,
                target,
                elements,
                "2.1",
            );
            assert!(result.is_some(), "{target_id}: setter route is not invoked");
            let result = result.unwrap();
            assert!(result.is_ok(), "{target_id}: {result:?}");
            assert!(crate::local_wire::target_hash(&result.unwrap(), &fixture).is_ok());
        }
    }

    /// OASIS KMIP 2.0/2.1 §3.1 Table 11 defines optional Key Block fields.
    #[test]
    fn key_block_optional_setters_execute_every_normative_target() {
        let catalog: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let manifest: Json = serde_json::from_str(include_str!(
            "../../shared/coverage/route-capabilities.json"
        ))
        .unwrap();
        let specification: Json =
            serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
                .unwrap();
        let elements = specification["elements"].as_array().unwrap();
        for symbol in [
            "kmipkit_protocol::KeyBlock::with_key_compression_type",
            "kmipkit_protocol::KeyBlock::with_key_wrapping_data",
        ] {
            let route = manifest["routes"]
                .as_array()
                .unwrap()
                .iter()
                .find(|route| route["public_api_symbol"] == symbol)
                .unwrap();
            for target_id in route["normative_target_ids"].as_array().unwrap() {
                let target_id = target_id.as_str().unwrap();
                let target = catalog["normative_targets"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|target| target["target_id"] == target_id)
                    .unwrap();
                let fixture = sample_for_route(symbol, target, elements);
                assert!(fixture.is_ok(), "{symbol} {target_id}: {fixture:?}");
                let fixture = fixture.unwrap();
                let result = invoke(symbol, &fixture, target, elements, "2.1");
                assert!(
                    result.is_some(),
                    "{symbol} {target_id}: route is not invoked"
                );
                let result = result.unwrap();
                assert!(result.is_ok(), "{symbol} {target_id}: {result:?}");
                assert!(crate::local_wire::target_hash(&result.unwrap(), &fixture).is_ok());
            }
        }
    }

    /// OASIS KMIP 2.0/2.1 §§3.2–3.12 define the typed Key Value and Key Material shapes.
    #[test]
    fn key_value_and_material_routes_execute_every_normative_target() {
        let catalog: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let manifest: Json = serde_json::from_str(include_str!(
            "../../shared/coverage/route-capabilities.json"
        ))
        .unwrap();
        let specification: Json =
            serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
                .unwrap();
        let elements = specification["elements"].as_array().unwrap();
        for symbol in [
            "kmipkit_protocol::KeyValue::try_new",
            "kmipkit_protocol::KeyValue::try_from_item",
            "kmipkit_protocol::KeyMaterial::byte_string",
            "kmipkit_protocol::KeyMaterial::structure",
            "kmipkit_protocol::KeyMaterial::try_from_item",
        ] {
            let route = manifest["routes"]
                .as_array()
                .unwrap()
                .iter()
                .find(|route| route["public_api_symbol"] == symbol)
                .unwrap();
            for target_id in route["normative_target_ids"].as_array().unwrap() {
                let target_id = target_id.as_str().unwrap();
                let target = catalog["normative_targets"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|target| target["target_id"] == target_id)
                    .unwrap();
                let fixture = sample_for_route(symbol, target, elements);
                assert!(fixture.is_ok(), "{symbol} {target_id}: {fixture:?}");
                let fixture = fixture.unwrap();
                let result = invoke(symbol, &fixture, target, elements, "2.1");
                assert!(
                    result.is_some(),
                    "{symbol} {target_id}: route is not invoked"
                );
                let result = result.unwrap();
                assert!(result.is_ok(), "{symbol} {target_id}: {result:?}");
                assert!(crate::local_wire::target_hash(&result.unwrap(), &fixture).is_ok());
            }
        }
    }

    #[test]
    fn key_format_type_request_routes_construct_each_normative_value() {
        // OASIS KMIP 2.0/2.1 §11.25 defines Key Format Type values.
        let catalog: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let manifest: Json = serde_json::from_str(include_str!(
            "../../shared/coverage/route-capabilities.json"
        ))
        .unwrap();
        let specification: Json =
            serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
                .unwrap();
        let elements = specification["elements"].as_array().unwrap();
        let targets = catalog["normative_targets"].as_array().unwrap();
        for symbol in [
            "kmipkit_protocol::GetRequest::with_key_format_type",
            "kmipkit_protocol::ExportRequest::with_key_format_type",
        ] {
            let route = manifest["routes"]
                .as_array()
                .unwrap()
                .iter()
                .find(|route| route["public_api_symbol"] == symbol)
                .unwrap();
            for target_id in route["normative_target_ids"].as_array().unwrap() {
                let Some(target_id) = target_id.as_str() else {
                    continue;
                };
                let Some(target) = targets
                    .iter()
                    .find(|target| target["target_id"] == target_id)
                else {
                    continue;
                };
                if target["kind"] != "enumeration_value"
                    || !target["parent_element_ids"]
                        .as_array()
                        .is_some_and(|parents| {
                            parents
                                .iter()
                                .any(|parent| parent == "KMIPKIT-ELEM-ENUMERATION-KEY-FORMAT-TYPE")
                        })
                {
                    continue;
                }
                let expected = sample(target, elements).unwrap();
                let result = invoke(symbol, &expected, target, elements, "2.1");
                assert!(result.is_some(), "{symbol} {target_id}: no request route");
                let result = result.unwrap();
                assert!(result.is_ok(), "{symbol} {target_id}: {result:?}");
                let projection = crate::local_wire::target_hash(&result.unwrap(), &expected);
                assert!(projection.is_ok(), "{symbol} {target_id}: {projection:?}");
            }
        }
    }

    #[test]
    fn unique_identifier_request_routes_construct_each_normative_value() {
        // OASIS KMIP 2.0/2.1 §11.38, Table 187 defines Unique Identifier wire forms.
        let catalog: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let manifest: Json = serde_json::from_str(include_str!(
            "../../shared/coverage/route-capabilities.json"
        ))
        .unwrap();
        let specification: Json =
            serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
                .unwrap();
        let elements = specification["elements"].as_array().unwrap();
        let targets = catalog["normative_targets"].as_array().unwrap();
        for route in manifest["routes"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|route| {
                route["language"] == "rust"
                    && route["public_api_symbol"]
                        .as_str()
                        .is_some_and(|symbol| symbol.ends_with("::with_unique_identifier"))
            })
        {
            let symbol = route["public_api_symbol"].as_str().unwrap();
            for target_id in route["normative_target_ids"].as_array().unwrap() {
                let Some(target_id) = target_id.as_str() else {
                    continue;
                };
                let Some(target) = targets
                    .iter()
                    .find(|target| target["target_id"] == target_id)
                else {
                    continue;
                };
                if target["kind"] != "enumeration_value"
                    || !target["parent_element_ids"]
                        .as_array()
                        .is_some_and(|parents| {
                            parents.iter().any(|parent| {
                                parent == "KMIPKIT-ELEM-ENUMERATION-UNIQUE-IDENTIFIER"
                            })
                        })
                {
                    continue;
                }
                let expected = sample(target, elements).unwrap();
                let result = invoke(symbol, &expected, target, elements, "2.1");
                assert!(result.is_some(), "{symbol} {target_id}: no request route");
                let result = result.unwrap();
                assert!(result.is_ok(), "{symbol} {target_id}: {result:?}");
                let projection = crate::local_wire::target_hash(&result.unwrap(), &expected);
                assert!(projection.is_ok(), "{symbol} {target_id}: {projection:?}");
            }
        }
    }

    #[test]
    fn reserved_enumeration_ranges_have_a_concrete_boundary_fixture() {
        let catalog: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let specification: Json =
            serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
                .unwrap();
        let elements = specification["elements"].as_array().unwrap();
        let targets = catalog["normative_targets"].as_array().unwrap();
        let mut checked = 0;
        for target in targets.iter().filter(|target| {
            target["kind"] == "enumeration_value" && target["allocation"] == "reserved"
        }) {
            let fixture = sample(target, elements);
            assert!(
                fixture.is_ok(),
                "{}: {fixture:?}",
                target["target_id"].as_str().unwrap_or("unknown target")
            );
            let fixture = fixture.unwrap();
            let expected = target_wire_value(target).unwrap();
            let raw = fixture.with_value(|value| match value {
                kmipkit::ttlv::ValueView::Enumeration(raw) => Some(*raw),
                _ => None,
            });
            assert_eq!(raw, Some(expected));
            checked += 1;
        }
        assert!(checked > 0, "catalog has no reserved Enumeration ranges");
    }

    #[test]
    fn key_wrapping_metadata_recipes_build_and_parse_each_manifest_target() {
        let catalog: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let manifest: Json = serde_json::from_str(include_str!(
            "../../shared/coverage/route-capabilities.json"
        ))
        .unwrap();
        let specification: Json =
            serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
                .unwrap();
        let elements = specification["elements"].as_array().unwrap();
        for symbol in [
            "kmipkit_protocol::EncryptionKeyInformation::try_from_item",
            "kmipkit_protocol::MacSignatureKeyInformation::try_from_item",
            "kmipkit_protocol::KeyWrappingData::try_from_item",
        ] {
            let route = manifest["routes"]
                .as_array()
                .unwrap()
                .iter()
                .find(|route| route["public_api_symbol"] == symbol)
                .unwrap();
            for target_id in route["normative_target_ids"].as_array().unwrap() {
                let target_id = target_id.as_str().unwrap();
                let target = catalog["normative_targets"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|target| target["target_id"] == target_id)
                    .unwrap();
                let expected = key_wrapping_metadata_model(symbol, target)
                    .and_then(|model| metadata_expected_target(&model, target, elements));
                assert!(expected.is_ok(), "{symbol} {target_id}: {expected:?}");
                let expected = expected.unwrap();
                let result = invoke(symbol, &expected, target, elements, "2.1");
                assert!(result.is_some(), "{symbol} {target_id}: no parser route");
                let result = result.unwrap();
                assert!(result.is_ok(), "{symbol} {target_id}: {result:?}");
                let projection = crate::local_wire::target_hash(&result.unwrap(), &expected);
                assert!(projection.is_ok(), "{symbol} {target_id}: {projection:?}");
            }
        }
    }

    /// OASIS KMIP 2.0/2.1 §3.3 Tables 14–17 define key wrapping metadata.
    #[test]
    fn key_wrapping_constructors_execute_every_normative_target() {
        let catalog: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let manifest: Json = serde_json::from_str(include_str!(
            "../../shared/coverage/route-capabilities.json"
        ))
        .unwrap();
        let specification: Json =
            serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
                .unwrap();
        let elements = specification["elements"].as_array().unwrap();
        for symbol in [
            "kmipkit_protocol::EncryptionKeyInformation::try_new",
            "kmipkit_protocol::EncryptionKeyInformation::with_cryptographic_parameters",
            "kmipkit_protocol::EncryptionKeyInformation::with_typed_cryptographic_parameters",
            "kmipkit_protocol::KeyWrappingData::try_new",
            "kmipkit_protocol::KeyWrappingData::with_encoding_option",
            "kmipkit_protocol::KeyWrappingData::with_encryption_key_information",
            "kmipkit_protocol::KeyWrappingData::with_iv_counter_nonce",
            "kmipkit_protocol::KeyWrappingData::with_mac_signature",
            "kmipkit_protocol::KeyWrappingData::with_mac_signature_key_information",
            "kmipkit_protocol::MacSignatureKeyInformation::try_new",
            "kmipkit_protocol::MacSignatureKeyInformation::with_cryptographic_parameters",
            "kmipkit_protocol::MacSignatureKeyInformation::with_typed_cryptographic_parameters",
        ] {
            let route = manifest["routes"]
                .as_array()
                .unwrap()
                .iter()
                .find(|route| route["public_api_symbol"] == symbol)
                .unwrap();
            for target_id in route["normative_target_ids"].as_array().unwrap() {
                let target_id = target_id.as_str().unwrap();
                let target = catalog["normative_targets"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|target| target["target_id"] == target_id)
                    .unwrap();
                let fixture = key_wrapping_metadata_model(symbol, target)
                    .and_then(|model| metadata_expected_target(&model, target, elements));
                assert!(fixture.is_ok(), "{symbol} {target_id}: {fixture:?}");
                let fixture = fixture.unwrap();
                let result = invoke(symbol, &fixture, target, elements, "2.1");
                assert!(
                    result.is_some(),
                    "{symbol} {target_id}: route is not invoked"
                );
                let result = result.unwrap();
                assert!(result.is_ok(), "{symbol} {target_id}: {result:?}");
                assert!(crate::local_wire::target_hash(&result.unwrap(), &fixture).is_ok());
            }
        }
    }

    /// OASIS KMIP 2.0/2.1 §§8.1–8.6 and §§9.1–9.21 define the message parser fields.
    #[test]
    fn message_parsers_execute_every_normative_target() {
        let catalog: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let manifest: Json = serde_json::from_str(include_str!(
            "../../shared/coverage/route-capabilities.json"
        ))
        .unwrap();
        let specification: Json =
            serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
                .unwrap();
        let elements = specification["elements"].as_array().unwrap();
        for protocol_version in ["2.0", "2.1"] {
            for symbol in [
                "kmipkit_protocol::RequestMessage::try_from_ttlv",
                "kmipkit_protocol::ResponseMessage::try_from_ttlv",
            ] {
                let route = manifest["routes"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|route| route["public_api_symbol"] == symbol)
                    .unwrap();
                for target_id in route["normative_target_ids"].as_array().unwrap() {
                    let target_id = target_id.as_str().unwrap();
                    let target = catalog["normative_targets"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|target| target["target_id"] == target_id)
                        .unwrap();
                    let fixture =
                        sample_for_route_at_version(symbol, target, elements, protocol_version);
                    assert!(
                        fixture.is_ok(),
                        "{symbol} {protocol_version} {target_id}: {fixture:?}"
                    );
                    let fixture = fixture.unwrap();
                    let result = invoke(symbol, &fixture, target, elements, protocol_version);
                    assert!(
                        result.is_some(),
                        "{symbol} {protocol_version} {target_id}: no parser route"
                    );
                    let result = result.unwrap();
                    assert!(
                        result.is_ok(),
                        "{symbol} {protocol_version} {target_id}: {result:?}"
                    );
                    let projection = crate::local_wire::target_hash(&result.unwrap(), &fixture);
                    assert!(
                        projection.is_ok(),
                        "{symbol} {protocol_version} {target_id}: {projection:?}"
                    );
                }
            }
        }
    }

    /// OASIS KMIP 2.0/2.1 §§5.1–5.7, 7.12, 7.22, 7.23, 7.37, 9.11, and 11.56.
    #[test]
    fn generic_ttlv_routes_build_concrete_member_and_extension_fixtures() {
        let catalog: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let specification: Json =
            serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
                .unwrap();
        let elements = specification["elements"].as_array().unwrap();
        let cases = [
            (
                "KMIPKIT-ELEM-STRUCTURE-MEMBER-5-1-ANY-ATTRIBUTE-IN-4-OBJECT-ATTRIBUTES",
                0x0042_0028,
            ),
            (
                "KMIPKIT-ELEM-STRUCTURE-MEMBER-5-2-ANY-ATTRIBUTE-IN-4-OBJECT-ATTRIBUTES",
                0x0042_0028,
            ),
            (
                "KMIPKIT-ELEM-STRUCTURE-MEMBER-5-3-ANY-ATTRIBUTE-IN-4-OBJECT-ATTRIBUTES",
                0x0042_0028,
            ),
            (
                "KMIPKIT-ELEM-STRUCTURE-MEMBER-5-4-ANY-ATTRIBUTE-IN-4-OBJECT-ATTRIBUTES",
                0x0042_0028,
            ),
            (
                "KMIPKIT-ELEM-STRUCTURE-MEMBER-5-6-ANY-ATTRIBUTE-IN-4-OBJECT-ATTRIBUTES",
                0x0042_0028,
            ),
            (
                "KMIPKIT-ELEM-STRUCTURE-MEMBER-5-7-ANY-ATTRIBUTE-IN-4-OBJECT-ATTRIBUTES",
                0x0042_0028,
            ),
            (
                "KMIPKIT-ELEM-STRUCTURE-MEMBER-7-12-CRYPTOGRAPHIC-PARAMETERS",
                0x0042_002b,
            ),
            (
                "KMIPKIT-ELEM-STRUCTURE-MEMBER-7-22-OBJECT-TYPE-OBJECTTYPES",
                0x0042_0057,
            ),
            (
                "KMIPKIT-ELEM-STRUCTURE-MEMBER-7-23-OBJECT-GROUP",
                0x0042_0056,
            ),
            (
                "KMIPKIT-ELEM-STRUCTURE-MEMBER-7-37-ALTERNATIVE-FAILOVER-ENDPOINTS",
                0x0042_0135,
            ),
            (
                "KMIPKIT-ELEM-STRUCTURE-MEMBER-7-37-VENDOR-SPECIFIC",
                0x0054_0000,
            ),
            (
                "KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-HASHED-PASSWORD-TIMESTAMP",
                0x0042_0092,
            ),
            ("KMIPKIT-RANGE-004", 0x0054_0000),
        ];
        for (target_id, expected_tag) in cases {
            let target = catalog["normative_targets"]
                .as_array()
                .unwrap()
                .iter()
                .find(|target| target["target_id"] == target_id)
                .unwrap();
            let sample = sample_for_route("kmipkit_ttlv::Item::new", target, elements);
            assert!(sample.is_ok(), "{target_id}: {sample:?}");
            let sample = sample.unwrap();
            assert_eq!(sample.tag().raw(), expected_tag, "{target_id}");
            let value_constructor = match sample.item_type() {
                kmipkit::ttlv::ItemType::Structure => "kmipkit_ttlv::Value::structure",
                kmipkit::ttlv::ItemType::Enumeration => "kmipkit_ttlv::Value::enumeration",
                kmipkit::ttlv::ItemType::TextString => "kmipkit_ttlv::Value::text_string",
                kmipkit::ttlv::ItemType::DateTimeExtended => {
                    "kmipkit_ttlv::Value::date_time_extended"
                }
                item_type => panic!("unexpected generic fixture type: {item_type:?}"),
            };
            let mut symbols = vec![
                "kmipkit_ttlv::Item::new",
                "kmipkit_ttlv::Item::with_value",
                "kmipkit_ttlv::codec::decode",
            ];
            symbols.push(value_constructor);
            for protocol_version in target["protocol_versions"].as_array().unwrap() {
                let protocol_version = protocol_version.as_str().unwrap();
                for symbol in &symbols {
                    let result = invoke(symbol, &sample, target, elements, protocol_version);
                    assert!(result.is_some(), "{symbol} omitted {target_id}");
                    let result = result.unwrap();
                    assert!(result.is_ok(), "{symbol} {target_id}: {result:?}");
                    assert!(crate::local_wire::target_hash(&result.unwrap(), &sample).is_ok());
                }
            }
        }
    }

    /// OASIS KMIP 2.0/2.1 §§2.1–2.2, 4.60, 6.1.19, 6.1.24, 6.1.39, 8.3, 8.5–8.6, and 9.4.
    #[test]
    #[allow(clippy::too_many_lines)] // Keep versioned fixture cases and route assertions auditable together.
    fn generic_ttlv_routes_cover_nested_objects_and_message_fields() {
        let catalog: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let specification: Json =
            serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
                .unwrap();
        let elements = specification["elements"].as_array().unwrap();
        let cases = [
            ("KMIPKIT-ELEM-ATTRIBUTE-VENDOR-ATTRIBUTE", 0x0042_0008),
            ("KMIPKIT-ELEM-MESSAGE-FIELD-8-3-EPHEMERAL", 0x0042_0154),
            (
                "KMIPKIT-ELEM-MESSAGE-FIELD-8-3-REQUEST-PAYLOAD",
                0x0042_0079,
            ),
            (
                "KMIPKIT-ELEM-MESSAGE-FIELD-8-5-SERVER-HASHED-PASSWORD",
                0x0042_0155,
            ),
            (
                "KMIPKIT-ELEM-MESSAGE-FIELD-8-6-RESPONSE-PAYLOAD",
                0x0042_007c,
            ),
            (
                "KMIPKIT-ELEM-MESSAGE-FIELD-9-4-CREDENTIAL-MAY-BE-REPEATED",
                0x0042_0023,
            ),
            ("KMIPKIT-PAYLOAD-CLIENT-EXPORT-RESPONSE-20-004", 0x0042_008f),
            ("KMIPKIT-PAYLOAD-CLIENT-EXPORT-RESPONSE-21-004", 0x0042_008f),
            ("KMIPKIT-PAYLOAD-CLIENT-GET-RESPONSE-20-003", 0x0042_008f),
            ("KMIPKIT-PAYLOAD-CLIENT-GET-RESPONSE-21-003", 0x0042_008f),
            ("KMIPKIT-PAYLOAD-CLIENT-IMPORT-REQUEST-20-006", 0x0042_008f),
            ("KMIPKIT-PAYLOAD-CLIENT-IMPORT-REQUEST-21-006", 0x0042_008f),
            (
                "KMIPKIT-PAYLOAD-CLIENT-REGISTER-REQUEST-20-003",
                0x0042_008f,
            ),
            (
                "KMIPKIT-PAYLOAD-CLIENT-REGISTER-REQUEST-21-003",
                0x0042_008f,
            ),
            ("KMIPKIT-PAYLOAD-SERVER-PUT-REQUEST-20-004", 0x0042_008f),
            ("KMIPKIT-PAYLOAD-SERVER-PUT-REQUEST-21-004", 0x0042_008f),
        ];
        for (target_id, expected_tag) in cases {
            let target = catalog["normative_targets"]
                .as_array()
                .unwrap()
                .iter()
                .find(|target| target["target_id"] == target_id)
                .unwrap();
            let sample = sample_for_route("kmipkit_ttlv::Item::new", target, elements);
            assert!(sample.is_ok(), "{target_id}: {sample:?}");
            let sample = sample.unwrap();
            assert_eq!(sample.tag().raw(), expected_tag, "{target_id}");
            for protocol_version in target["protocol_versions"].as_array().unwrap() {
                let protocol_version = protocol_version.as_str().unwrap();
                for symbol in [
                    "kmipkit_ttlv::Item::new",
                    "kmipkit_ttlv::Item::with_value",
                    "kmipkit_ttlv::codec::decode",
                ] {
                    let result = invoke(symbol, &sample, target, elements, protocol_version);
                    assert!(result.is_some(), "{symbol} omitted {target_id}");
                    let result = result.unwrap();
                    assert!(result.is_ok(), "{symbol} {target_id}: {result:?}");
                    assert!(crate::local_wire::target_hash(&result.unwrap(), &sample).is_ok());
                }
            }
        }
        for target_id in [
            "KMIPKIT-ELEM-ATTRIBUTE-UNIQUE-IDENTIFIER",
            "KMIPKIT-ELEM-STRUCTURE-MEMBER-7-21-UNIQUE-IDENTIFIER",
        ] {
            let target = catalog["normative_targets"]
                .as_array()
                .unwrap()
                .iter()
                .find(|target| target["target_id"] == target_id)
                .unwrap();
            for (symbol, expected_type) in [
                (
                    "kmipkit_ttlv::Value::text_string",
                    kmipkit::ttlv::ItemType::TextString,
                ),
                (
                    "kmipkit_ttlv::Value::enumeration",
                    kmipkit::ttlv::ItemType::Enumeration,
                ),
                (
                    "kmipkit_ttlv::Value::integer",
                    kmipkit::ttlv::ItemType::Integer,
                ),
            ] {
                let sample = sample_for_route(symbol, target, elements);
                assert!(sample.is_ok(), "{symbol} {target_id}: {sample:?}");
                let sample = sample.unwrap();
                assert_eq!(sample.item_type(), expected_type, "{symbol} {target_id}");
                let result = invoke(symbol, &sample, target, elements, "2.1");
                assert!(result.is_some(), "{symbol} omitted {target_id}");
                let result = result.unwrap();
                assert!(result.is_ok(), "{symbol} {target_id}: {result:?}");
                assert!(crate::local_wire::target_hash(&result.unwrap(), &sample).is_ok());
            }
        }
    }

    #[test]
    fn vendor_attribute_target_uses_a_table_150_structure_fixture() {
        let cases: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let specification: Json =
            serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
                .unwrap();
        let target = cases["normative_targets"]
            .as_array()
            .unwrap()
            .iter()
            .find(|target| target["target_id"] == "KMIPKIT-ELEM-TAG-420008")
            .unwrap();
        let elements = specification["elements"].as_array().unwrap();
        let fixture = sample(target, elements).unwrap();
        assert_eq!(fixture.item_type(), kmipkit::ttlv::ItemType::Structure);
        let result = invoke(
            "kmipkit_protocol::AttributeSet::try_new",
            &fixture,
            target,
            elements,
            "2.1",
        )
        .unwrap()
        .unwrap();
        let projected = crate::local_wire::target_hash(&result, &fixture);
        assert!(projected.is_ok(), "{projected:?}");
    }

    #[test]
    fn structure_try_push_projects_an_empty_object_structure_child() {
        let cases: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let specification: Json =
            serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
                .unwrap();
        let target = cases["normative_targets"]
            .as_array()
            .unwrap()
            .iter()
            .find(|target| target["target_id"] == "KMIPKIT-ELEM-OBJECT-STRUCTURE-CERTIFICATE")
            .unwrap();
        let elements = specification["elements"].as_array().unwrap();
        let fixture = sample(target, elements).unwrap();
        let result = invoke(
            "kmipkit_ttlv::Structure::try_push",
            &fixture,
            target,
            elements,
            "2.1",
        )
        .unwrap()
        .unwrap();
        let projected = crate::local_wire::target_hash(&result, &fixture);
        assert!(projected.is_ok(), "{projected:?}");
    }

    /// OASIS KMIP 2.1 §4.21, Table 69 defines Digest as an attribute Structure.
    #[test]
    fn typed_attribute_route_wraps_each_member_in_its_attribute_structure() {
        let cases: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let specification: Json =
            serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
                .unwrap();
        let target = cases["normative_targets"]
            .as_array()
            .unwrap()
            .iter()
            .find(|target| target["target_id"] == "KMIPKIT-ELEM-STRUCTURE-MEMBER-4-21-DIGEST-VALUE")
            .unwrap();
        let elements = specification["elements"].as_array().unwrap();
        let fixture = sample_for_route(
            "kmipkit_protocol::attribute::Attribute::<kmipkit_protocol::attribute::typed::Digest>::from_item",
            target,
            elements,
        )
        .unwrap();
        let member = sample(target, elements).unwrap();
        assert!(crate::local_wire::target_hash(&fixture, &member).is_ok());
        let result = invoke(
            "kmipkit_protocol::attribute::Attribute::<kmipkit_protocol::attribute::typed::Digest>::from_item",
            &fixture,
            target,
            elements,
            "2.1",
        )
        .unwrap()
        .unwrap();

        assert!(crate::local_wire::target_hash(&result, &fixture).is_ok());
    }

    /// OASIS KMIP 2.1 §4.21 and §§5.6–5.7 preserve the direct Digest item in typed wrappers.
    #[test]
    fn typed_attribute_wrapper_routes_resolve_the_attribute_marker() {
        let cases: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let specification: Json =
            serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
                .unwrap();
        let target = cases["normative_targets"]
            .as_array()
            .unwrap()
            .iter()
            .find(|target| target["target_id"] == "KMIPKIT-ELEM-ATTRIBUTE-DIGEST")
            .unwrap();
        let elements = specification["elements"].as_array().unwrap();
        for symbol in [
            "kmipkit_protocol::AttributeSet::try_push_typed::<kmipkit_protocol::attribute::typed::Digest>",
            "kmipkit_protocol::CurrentAttribute::from_typed::<kmipkit_protocol::attribute::typed::Digest>",
            "kmipkit_protocol::NewAttribute::from_typed::<kmipkit_protocol::attribute::typed::Digest>",
        ] {
            let fixture = sample_for_route(symbol, target, elements).unwrap();
            let result = invoke(symbol, &fixture, target, elements, "2.1")
                .unwrap()
                .unwrap();
            assert!(crate::local_wire::target_hash(&result, &fixture).is_ok());
        }
    }

    /// OASIS KMIP 2.1 §9.11, Tables 411–416 and §7.39 define credential field shapes.
    #[test]
    fn credential_routes_preserve_the_target_field_in_a_valid_variant() {
        let cases: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let specification: Json =
            serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
                .unwrap();
        let elements = specification["elements"].as_array().unwrap();
        for (target_id, symbol) in [
            (
                "KMIPKIT-ELEM-TAG-42000C",
                "kmipkit_protocol::Authentication::new",
            ),
            (
                "KMIPKIT-ELEM-TAG-42000C",
                "kmipkit_protocol::Authentication::try_from_ttlv",
            ),
            (
                "KMIPKIT-ELEM-TAG-420023",
                "kmipkit_protocol::Credential::try_from_ttlv",
            ),
            (
                "KMIPKIT-ELEM-TAG-420025",
                "kmipkit_protocol::CredentialValue::try_from_ttlv",
            ),
            (
                "KMIPKIT-ELEM-TAG-4200C8",
                "kmipkit_protocol::Nonce::try_from_ttlv",
            ),
            (
                "KMIPKIT-ELEM-TAG-4200C8",
                "kmipkit_protocol::AttestationCredential::try_from_ttlv",
            ),
            (
                "KMIPKIT-ELEM-TAG-4200C8",
                "kmipkit_protocol::CredentialValue::try_from_ttlv",
            ),
            (
                "KMIPKIT-ELEM-TAG-420092",
                "kmipkit_protocol::CredentialValue::try_from_ttlv",
            ),
            (
                "KMIPKIT-ELEM-TAG-420092",
                "kmipkit_protocol::HashedPasswordCredential::try_from_ttlv",
            ),
            (
                "KMIPKIT-ELEM-TAG-420149",
                "kmipkit_protocol::CredentialValue::try_from_ttlv",
            ),
            (
                "KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-TICKET-TICKET",
                "kmipkit_protocol::CredentialValue::try_from_ttlv",
            ),
            (
                "KMIPKIT-ELEM-TAG-420156",
                "kmipkit_protocol::CredentialValue::try_from_ttlv",
            ),
            (
                "KMIPKIT-ELEM-TAG-420157",
                "kmipkit_protocol::CredentialValue::try_from_ttlv",
            ),
            (
                "KMIPKIT-ELEM-TAG-420157",
                "kmipkit_protocol::HashedPasswordCredential::try_from_ttlv",
            ),
        ] {
            let target = cases["normative_targets"]
                .as_array()
                .unwrap()
                .iter()
                .find(|target| target["target_id"] == target_id)
                .unwrap();
            let fixture = sample(target, elements).unwrap();
            let result = invoke(symbol, &fixture, target, elements, "2.1").unwrap();
            assert!(
                result.is_ok(),
                "{target_id} via {symbol}: {:?}",
                result.err()
            );
            assert!(crate::local_wire::target_hash(&result.unwrap(), &fixture).is_ok());
        }
    }

    /// OASIS KMIP 2.0/2.1 §11.56 requires unallocated tag ranges to remain decodable as rejections.
    #[test]
    fn enum_tag_decode_routes_reject_unallocated_ranges_from_target_metadata() {
        let catalog: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let target = catalog["normative_targets"]
            .as_array()
            .unwrap()
            .iter()
            .find(|target| target["target_id"] == "KMIPKIT-RANGE-001")
            .unwrap();
        let row = json!({
            "action": "decode",
            "case_id": "test-enum-tag",
            "public_api_symbol": "kmipkit_ttlv::codec::decode",
            "route_id": "rust.generic-ttlv.codec.decode",
            "target_id": "KMIPKIT-RANGE-001",
            "protocol_version": "2.1"
        });
        let Some((raw_tag, rejection_code)) = expected_tag_decode_rejection(&row, target, &catalog)
        else {
            panic!("unallocated tag target did not produce a rejection recipe");
        };
        assert_eq!(raw_tag, 0);
        assert_eq!(rejection_code, "tag_not_allocated");
        let decoded = kmipkit::ttlv::codec::decode(&reserved_tag_ttlv(raw_tag));
        assert!(matches!(decoded, Err(error) if error.kind() == DecodeErrorKind::UnallocatedTag));
    }
}
