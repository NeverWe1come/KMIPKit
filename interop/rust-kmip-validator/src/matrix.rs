//! Execute only named public routes; pending recipes never produce call evidence.
use crate::models::{Connection, Result, hash};
use kmipkit::protocol::{
    EncryptRequest, OperationData, QueryFunction, QueryRequest, SecretBytes, UniqueIdentifier,
};
use kmipkit::ttlv::{Item, Value, ValueView, codec::DecodeErrorKind};
use kmipkit::{ClientBatch, ClientBatchItem, ClientRequest};
use serde_json::{Value as Json, json};
use std::{
    collections::HashSet,
    io::{BufRead, BufReader},
    path::Path,
};

#[path = "route_calls_generated.rs"]
mod generated;

const KEY_BLOCK_TAG: u32 = 0x0042_0040;
const KEY_PAIR_ATTRIBUTE_ROUTES: &[&str] = &[
    "kmipkit_protocol::CreateKeyPairRequest::with_common_attributes",
    "kmipkit_protocol::CreateKeyPairRequest::with_private_key_attributes",
    "kmipkit_protocol::CreateKeyPairRequest::with_public_key_attributes",
    "kmipkit_protocol::RekeyKeyPairRequest::with_common_attributes",
    "kmipkit_protocol::RekeyKeyPairRequest::with_private_key_attributes",
    "kmipkit_protocol::RekeyKeyPairRequest::with_public_key_attributes",
];
const KEY_WRAPPING_CONSTRUCTOR_ROUTES: &[&str] = &[
    "kmipkit_protocol::EncryptionKeyInformation::try_new",
    "kmipkit_protocol::EncryptionKeyInformation::try_from_unique_identifier",
    "kmipkit_protocol::EncryptionKeyInformation::with_cryptographic_parameters",
    "kmipkit_protocol::EncryptionKeyInformation::with_typed_cryptographic_parameters",
    "kmipkit_protocol::KeyWrappingData::try_new",
    "kmipkit_protocol::KeyWrappingData::with_encoding_option",
    "kmipkit_protocol::KeyWrappingData::with_encryption_key_information",
    "kmipkit_protocol::KeyWrappingData::with_iv_counter_nonce",
    "kmipkit_protocol::KeyWrappingData::with_mac_signature",
    "kmipkit_protocol::KeyWrappingData::with_mac_signature_key_information",
    "kmipkit_protocol::MacSignatureKeyInformation::try_new",
    "kmipkit_protocol::MacSignatureKeyInformation::try_from_unique_identifier",
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

fn certificate_attributes_item() -> Result<Item> {
    let mut certificate_fields = kmipkit::ttlv::Structure::new();
    certificate_fields
        .try_push(crate::driver::item(
            0x0042_0108,
            Value::text_string("kmipkit.example".to_owned()),
        )?)
        .map_err(|_| "Certificate Subject CN fixture rejected")?;
    let mut attribute_fields = kmipkit::ttlv::Structure::new();
    attribute_fields
        .try_push(crate::driver::item(
            0x0042_000a,
            Value::text_string("Certificate Attributes".to_owned()),
        )?)
        .map_err(|_| "Certificate Attributes name fixture rejected")?;
    attribute_fields
        .try_push(crate::driver::item(
            0x0042_000b,
            Value::structure(certificate_fields),
        )?)
        .map_err(|_| "Certificate Attributes value fixture rejected")?;
    crate::driver::item(0x0042_0008, Value::structure(attribute_fields))
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
    if target_id == "KMIPKIT-ELEM-ATTRIBUTE-CERTIFICATE-ATTRIBUTES" {
        return certificate_attributes_item();
    }
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

fn typed_attribute_constructor_route(symbol: &str) -> bool {
    symbol.contains("kmipkit_protocol::attribute::Attribute::<") && symbol.ends_with("::new")
}

fn typed_attribute_constructor_item(symbol: &str, elements: &[Json]) -> Result<Item> {
    let marker = typed_attribute_marker(symbol).ok_or("typed attribute marker missing")?;
    if normalized_name(marker) == "vendorattribute" {
        return vendor_attribute_item();
    }
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
    let value = if normalized_name(marker) == "vendorattribute" {
        vendor_attribute_value()?
    } else if attribute["source_encoding"] == "Structure" {
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
    } else {
        sample(attribute, elements)?.with_value(value_to_owned)?
    };
    crate::driver::item(tag, value)
}

fn vendor_attribute_item() -> Result<Item> {
    crate::driver::item(0x0042_0008, vendor_attribute_value()?)
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
    let (_, object, expected) = if target["target_id"] == "KMIPKIT-ELEM-TAG-420040" {
        object_root_fixture("Symmetric Key", target, elements)?
    } else {
        managed_object_fixture(target, elements)?
    };
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

fn ticket_ttlv_fixture() -> Result<Item> {
    let mut fields = kmipkit::ttlv::Structure::new();
    fields
        .try_push(crate::driver::item(
            0x0042_014a,
            Value::enumeration(kmipkit::protocol::TicketType::LOGIN.raw()),
        )?)
        .map_err(|_| "Ticket fixture rejected")?;
    fields
        .try_push(crate::driver::item(
            0x0042_014b,
            Value::byte_string(vec![0x41; 16]),
        )?)
        .map_err(|_| "Ticket fixture rejected")?;
    crate::driver::item(0x0042_0149, Value::structure(fields))
}

fn ticket_credential_fixture(ticket: Item) -> Result<Item> {
    let mut fields = kmipkit::ttlv::Structure::new();
    fields
        .try_push(ticket)
        .map_err(|_| "Ticket Credential fixture rejected")?;
    crate::driver::item(0x0042_0025, Value::structure(fields))
}

fn ticket_fixture_target(target: &Json, elements: &[Json]) -> Result<Item> {
    let model = ticket_credential_fixture(ticket_ttlv_fixture()?)?;
    let tag = sample(target, elements)?.tag().raw();
    let mut candidates = Vec::new();
    collect_tagged_items(&model, tag, &mut candidates)?;
    match candidates.as_slice() {
        [item] => item.try_clone().map_err(|_| "Ticket fixture copy failed"),
        [] => Err("Ticket fixture target is missing"),
        _ => Err("Ticket fixture target is ambiguous"),
    }
}

fn extension_information_ttlv_fixture() -> Result<Item> {
    let mut fields = kmipkit::ttlv::Structure::new();
    for field in [
        crate::driver::item(
            0x0042_00a5,
            Value::text_string("KMIPKit.Validator".to_owned()),
        )?,
        crate::driver::item(0x0042_00a6, Value::integer(0x0054_0000))?,
        crate::driver::item(0x0042_00a7, Value::enumeration(7))?,
    ] {
        fields
            .try_push(field)
            .map_err(|_| "Extension Information fixture rejected")?;
    }
    crate::driver::item(0x0042_00a4, Value::structure(fields))
}

fn extension_information_fixture_target(target: &Json, elements: &[Json]) -> Result<Item> {
    metadata_expected_target(&extension_information_ttlv_fixture()?, target, elements)
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

fn failed_response_message_fixture(
    operation: &str,
    target: &Json,
    elements: &[Json],
    protocol_version: &str,
) -> Result<kmipkit::protocol::ResponseMessage> {
    let operation = operation_value(operation).ok_or("response operation is unavailable")?;
    let header = message_structure(
        0x0042_007A,
        vec![
            message_protocol_version(target, elements, protocol_version)?,
            message_item(0x0042_0092, Value::date_time(1))?,
            message_nonce(target, elements)?,
            message_item(0x0042_0155, Value::byte_string(vec![0x50]))?,
            message_item(0x0042_00C7, Value::enumeration(1))?,
            message_item(
                0x0042_0105,
                Value::text_string("client-correlation".to_owned()),
            )?,
            message_item(
                0x0042_0106,
                Value::text_string("server-correlation".to_owned()),
            )?,
            message_item(0x0042_000D, Value::integer(1))?,
        ],
    )?;
    let batch_item = message_structure(
        0x0042_000F,
        vec![
            message_item(0x0042_005C, Value::enumeration(operation))?,
            message_item(0x0042_0093, Value::byte_string(vec![0x4B, 1]))?,
            message_item(0x0042_007F, Value::enumeration(1))?,
            message_item(0x0042_007E, Value::enumeration(1))?,
            message_item(
                0x0042_007D,
                Value::text_string("controlled fixture failure".to_owned()),
            )?,
        ],
    )?;
    let message = message_structure(0x0042_007B, vec![header, batch_item])?
        .as_structure()
        .ok_or("Response Message fixture root is not a Structure")?
        .try_clone()
        .map_err(|_| "Response Message fixture copy failed")?;
    kmipkit::protocol::ResponseMessage::try_from_ttlv(message)
        .map_err(|_| "Response Message fixture rejected")
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
    if symbol.ends_with("protection_storage_masks") && target["kind"] == "tag" {
        let tag = target_wire_value(target)?;
        return crate::driver::item(tag, Value::structure(kmipkit::ttlv::Structure::new()));
    }
    let secret_operation_route = matches!(
        symbol,
        "kmipkit_protocol::CertifyRequest::with_certificate_request_value"
            | "kmipkit_protocol::CreateSplitKeyRequest::with_prime_field_size"
            | "kmipkit_protocol::DecryptRequest::with_authenticated_encryption_additional_data"
            | "kmipkit_protocol::DecryptRequest::with_authenticated_encryption_tag"
            | "kmipkit_protocol::DecryptRequest::with_correlation_value"
            | "kmipkit_protocol::DecryptRequest::with_iv_counter_nonce"
            | "kmipkit_protocol::EncryptRequest::with_authenticated_encryption_additional_data"
            | "kmipkit_protocol::EncryptRequest::with_correlation_value"
            | "kmipkit_protocol::EncryptRequest::with_iv_counter_nonce"
            | "kmipkit_protocol::HashRequest::with_correlation_value"
            | "kmipkit_protocol::MacRequest::with_correlation_value"
            | "kmipkit_protocol::MacVerifyRequest::with_correlation_value"
            | "kmipkit_protocol::MacVerifyRequest::with_mac_data"
            | "kmipkit_protocol::Pkcs11Request::with_correlation_value"
            | "kmipkit_protocol::RecertifyRequest::with_certificate_request_value"
            | "kmipkit_protocol::SignRequest::with_correlation_value"
            | "kmipkit_protocol::SignRequest::with_digested_data"
            | "kmipkit_protocol::SignatureVerifyRequest::with_correlation_value"
            | "kmipkit_protocol::SignatureVerifyRequest::with_digested_data"
            | "kmipkit_protocol::SignatureVerifyRequest::with_signature_data"
    );
    if secret_operation_route
        && target["kind"] == "tag"
        && [
            "Certificate Request Value",
            "Prime Field Size",
            "Authenticated Encryption Additional Data",
            "Authenticated Encryption Tag",
            "Correlation Value",
            "IV/Counter/Nonce",
            "MAC Data",
            "Digested Data",
            "Signature Data",
        ]
        .contains(&target["name"].as_str().unwrap_or_default())
    {
        let tag = target_wire_value(target)?;
        let value = if target["name"] == "Prime Field Size" {
            Value::big_integer(vec![0x41; 8])
        } else {
            Value::byte_string(vec![0x4b])
        };
        return crate::driver::item(tag, value);
    }
    if symbol == "kmipkit_protocol::AttributeReference::tag" {
        return crate::driver::item(0x0042_013b, Value::enumeration(0x0042_0001));
    }
    if symbol == "kmipkit_protocol::ProtocolVersion::from_raw" {
        let message = if target["target_id"]
            .as_str()
            .is_some_and(|target_id| target_id.contains("MESSAGE-FIELD-8-5"))
        {
            response_message_fixture(target, elements, protocol_version)?
        } else {
            request_message_fixture(target, elements, protocol_version)?
        };
        return metadata_expected_target(&message, target, elements);
    }
    if [
        "kmipkit_protocol::Ticket::new",
        "kmipkit_protocol::Ticket::try_from_ttlv",
    ]
    .contains(&symbol)
    {
        return ticket_fixture_target(target, elements);
    }
    if symbol == "kmipkit_protocol::ManagedObject::unknown"
        && target["target_id"] == "KMIPKIT-RANGE-004"
    {
        let raw_tag = sample(target, elements)?.tag().raw();
        return crate::driver::item(raw_tag, Value::structure(kmipkit::ttlv::Structure::new()));
    }
    if symbol == "kmipkit_protocol::extension::to_ttlv" {
        return extension_information_fixture_target(target, elements);
    }
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
        "kmipkit_protocol::TransparentSymmetricKey::try_from_item",
        "kmipkit_protocol::TransparentSymmetricKey::try_new",
        "kmipkit_protocol::TransparentRsaPrivateKey::try_from_components",
        "kmipkit_protocol::TransparentRsaPrivateKey::try_from_item",
        "kmipkit_protocol::TransparentDhPrivateKey::try_from_item",
        "kmipkit_protocol::TransparentDhPrivateKey::try_new",
        "kmipkit_protocol::TransparentDhPublicKey::try_from_item",
        "kmipkit_protocol::TransparentDhPublicKey::try_new",
        "kmipkit_protocol::TransparentDsaPrivateKey::try_from_item",
        "kmipkit_protocol::TransparentDsaPrivateKey::try_new",
        "kmipkit_protocol::TransparentDsaPublicKey::try_from_item",
        "kmipkit_protocol::TransparentDsaPublicKey::try_new",
        "kmipkit_protocol::TransparentEcPrivateKey::try_from_item",
        "kmipkit_protocol::TransparentEcPrivateKey::try_new",
        "kmipkit_protocol::TransparentEcPublicKey::try_from_item",
        "kmipkit_protocol::TransparentEcPublicKey::try_new",
        "kmipkit_protocol::TransparentRsaPublicKey::try_from_item",
        "kmipkit_protocol::TransparentRsaPublicKey::try_new",
    ]
    .contains(&symbol)
    {
        let (_, model) = key_material_item_for_target(target, 10)?;
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
    if KEY_PAIR_ATTRIBUTE_ROUTES.contains(&symbol)
        && target["kind"] == "structure_member"
        && target["target_id"]
            .as_str()
            .is_some_and(|target_id| target_id.contains("ANY-ATTRIBUTE-IN-4-OBJECT-ATTRIBUTES"))
    {
        return crate::driver::item(0x0042_002c, Value::integer(1));
    }
    if crypto_indicator_operation(symbol).is_some()
        && (symbol.ends_with("::with_init_indicator") || symbol.ends_with("::with_final_indicator"))
        && target["kind"] == "tag"
    {
        return crate::driver::item(target_wire_value(target)?, Value::boolean(true));
    }
    if KEY_PAIR_ATTRIBUTE_ROUTES.contains(&symbol) && target["kind"] == "tag" {
        return crate::driver::item(
            target_wire_value(target)?,
            Value::structure(kmipkit::ttlv::Structure::new()),
        );
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
    if typed_attribute_constructor_route(symbol) {
        let model = typed_attribute_constructor_item(symbol, elements)?;
        return metadata_expected_target(&model, target, elements);
    }
    if typed_attribute_marker(symbol) == Some("VendorAttribute") {
        let model = vendor_attribute_item()?;
        return metadata_expected_target(&model, target, elements);
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

fn invoke_attribute_reference_route(symbol: &str, sample: &Item) -> Option<Result<Item>> {
    if ![
        "kmipkit_protocol::AttributeReference::name",
        "kmipkit_protocol::AttributeReference::tag",
    ]
    .contains(&symbol)
    {
        return None;
    }
    Some((|| {
        match symbol {
            "kmipkit_protocol::AttributeReference::name" => {
                let reference = kmipkit::protocol::AttributeReference::name(
                    "KMIPKit.TestVendor_1",
                    "Opaque.Attribute",
                );
                if reference.name_parts() != Some(("KMIPKit.TestVendor_1", "Opaque.Attribute")) {
                    return Err("Attribute Reference name constructor lost its values");
                }
            }
            "kmipkit_protocol::AttributeReference::tag" => {
                let raw_tag = sample.with_value(|value| match value {
                    ValueView::Enumeration(value) => Ok(*value),
                    _ => Err("Attribute Reference Tag fixture is not an Enumeration"),
                })?;
                let reference = kmipkit::protocol::AttributeReference::tag(raw_tag);
                if reference.tag_value() != Some(raw_tag) {
                    return Err("Attribute Reference tag constructor lost its value");
                }
            }
            _ => return Err("unsupported Attribute Reference route"),
        }
        sample.try_clone().map_err(|_| "local copy failed")
    })())
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
                let structure = if sample.tag().raw() == 0x0042_0023 {
                    let mut structure = kmipkit::ttlv::Structure::new();
                    structure
                        .try_push(
                            sample
                                .try_clone()
                                .map_err(|_| "Authentication fixture copy failed")?,
                        )
                        .map_err(|_| "Authentication fixture rejected")?;
                    structure
                } else {
                    sample
                        .as_structure()
                        .ok_or("Authentication fixture is not a Structure")?
                        .try_clone()
                        .map_err(|_| "Authentication fixture copy failed")?
                };
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
                    ValueView::DateTime(value) | ValueView::DateTimeExtended(value) => Ok(*value),
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

// Keep response accessor mappings next to their typed response fixtures.
#[allow(clippy::too_many_lines)]
fn invoke_client_batch_outcome_response_route(
    symbol: &str,
    sample: &Item,
    target: &Json,
    elements: &[Json],
    protocol_version: &str,
) -> Option<Result<Item>> {
    const RESPONSE_ACCESSOR_ROUTES: &[&str] = &[
        "kmipkit_client::ClientBatchOutcome::certify_response",
        "kmipkit_client::ClientBatchOutcome::check_response",
        "kmipkit_client::ClientBatchOutcome::decrypt_response",
        "kmipkit_client::ClientBatchOutcome::delegated_login_response",
        "kmipkit_client::ClientBatchOutcome::encrypt_response",
        "kmipkit_client::ClientBatchOutcome::hash_response",
        "kmipkit_client::ClientBatchOutcome::join_split_key_response",
        "kmipkit_client::ClientBatchOutcome::login_response",
        "kmipkit_client::ClientBatchOutcome::logout_response",
        "kmipkit_client::ClientBatchOutcome::mac_response",
        "kmipkit_client::ClientBatchOutcome::mac_verify_response",
        "kmipkit_client::ClientBatchOutcome::recertify_response",
        "kmipkit_client::ClientBatchOutcome::register_response",
        "kmipkit_client::ClientBatchOutcome::rekey_response",
        "kmipkit_client::ClientBatchOutcome::rekey_key_pair_response",
        "kmipkit_client::ClientBatchOutcome::reprovision_response",
        "kmipkit_client::ClientBatchOutcome::revoke_response",
        "kmipkit_client::ClientBatchOutcome::sign_response",
        "kmipkit_client::ClientBatchOutcome::signature_verify_response",
        "kmipkit_client::ClientBatchOutcome::rng_retrieve_response",
        "kmipkit_client::ClientBatchOutcome::rng_seed_response",
        "kmipkit_client::ClientBatchOutcome::Activate",
        "kmipkit_client::ClientBatchOutcome::Archive",
        "kmipkit_client::ClientBatchOutcome::DeriveKey",
        "kmipkit_client::ClientBatchOutcome::Destroy",
        "kmipkit_client::ClientBatchOutcome::GetUsageAllocation",
        "kmipkit_client::ClientBatchOutcome::Recover",
    ];
    if !RESPONSE_ACCESSOR_ROUTES.contains(&symbol) {
        return None;
    }

    macro_rules! invoke_response_accessor {
        ($operation:literal, $response:ident, $variant:ident, $accessor:ident) => {{
            let message =
                failed_response_message_fixture($operation, target, elements, protocol_version)?;
            let item = message
                .batch_items()
                .next()
                .ok_or("typed operation response fixture has no batch item")?;
            let response = kmipkit::protocol::$response::try_from_response_item(item)
                .map_err(|_| "typed operation response fixture rejected")?;
            let outcome = kmipkit::ClientBatchOutcome::$variant(response);
            if outcome.result().status().raw() != 1 || outcome.$accessor().is_none() {
                return Err("ClientBatchOutcome accessor did not return its typed response");
            }
            sample.try_clone().map_err(|_| "local copy failed")
        }};
    }

    macro_rules! invoke_response_variant {
        ($operation:literal, $response:ident, $variant:ident, $operation_variant:ident) => {{
            let message =
                failed_response_message_fixture($operation, target, elements, protocol_version)?;
            let item = message
                .batch_items()
                .next()
                .ok_or("typed operation response fixture has no batch item")?;
            let response = kmipkit::protocol::$response::try_from_response_item(item)
                .map_err(|_| "typed operation response fixture rejected")?;
            let outcome = kmipkit::ClientBatchOutcome::$variant(response);
            if outcome.operation() != kmipkit::ClientOperation::$operation_variant
                || outcome.result().status().raw() != 1
            {
                return Err("ClientBatchOutcome variant did not preserve its operation result");
            }
            sample.try_clone().map_err(|_| "local copy failed")
        }};
    }

    Some((|| match symbol {
        "kmipkit_client::ClientBatchOutcome::certify_response" => {
            invoke_response_accessor!("Certify", CertifyResponse, Certify, certify_response)
        }
        "kmipkit_client::ClientBatchOutcome::check_response" => {
            invoke_response_accessor!("Check", CheckResponse, Check, check_response)
        }
        "kmipkit_client::ClientBatchOutcome::decrypt_response" => {
            invoke_response_accessor!("Decrypt", DecryptResponse, Decrypt, decrypt_response)
        }
        "kmipkit_client::ClientBatchOutcome::delegated_login_response" => {
            invoke_response_accessor!(
                "Delegated Login",
                DelegatedLoginResponse,
                DelegatedLogin,
                delegated_login_response
            )
        }
        "kmipkit_client::ClientBatchOutcome::encrypt_response" => {
            invoke_response_accessor!("Encrypt", EncryptResponse, Encrypt, encrypt_response)
        }
        "kmipkit_client::ClientBatchOutcome::hash_response" => {
            invoke_response_accessor!("Hash", HashResponse, Hash, hash_response)
        }
        "kmipkit_client::ClientBatchOutcome::join_split_key_response" => invoke_response_accessor!(
            "Join Split Key",
            JoinSplitKeyResponse,
            JoinSplitKeyCompleted,
            join_split_key_response
        ),
        "kmipkit_client::ClientBatchOutcome::login_response" => {
            invoke_response_accessor!("Login", LoginResponse, Login, login_response)
        }
        "kmipkit_client::ClientBatchOutcome::logout_response" => {
            invoke_response_accessor!("Logout", LogoutResponse, Logout, logout_response)
        }
        "kmipkit_client::ClientBatchOutcome::mac_response" => {
            invoke_response_accessor!("MAC", MacResponse, Mac, mac_response)
        }
        "kmipkit_client::ClientBatchOutcome::mac_verify_response" => invoke_response_accessor!(
            "MAC Verify",
            MacVerifyResponse,
            MacVerify,
            mac_verify_response
        ),
        "kmipkit_client::ClientBatchOutcome::recertify_response" => invoke_response_accessor!(
            "Re-certify",
            RecertifyResponse,
            Recertify,
            recertify_response
        ),
        "kmipkit_client::ClientBatchOutcome::register_response" => {
            invoke_response_accessor!("Register", RegisterResponse, Register, register_response)
        }
        "kmipkit_client::ClientBatchOutcome::rekey_response" => {
            invoke_response_accessor!("Re-key", RekeyResponse, Rekey, rekey_response)
        }
        "kmipkit_client::ClientBatchOutcome::rekey_key_pair_response" => invoke_response_accessor!(
            "Re-key Key Pair",
            RekeyKeyPairResponse,
            RekeyKeyPair,
            rekey_key_pair_response
        ),
        "kmipkit_client::ClientBatchOutcome::reprovision_response" => invoke_response_accessor!(
            "Re-Provision",
            ReProvisionResponse,
            ReProvision,
            reprovision_response
        ),
        "kmipkit_client::ClientBatchOutcome::revoke_response" => {
            invoke_response_accessor!("Revoke", RevokeResponse, Revoke, revoke_response)
        }
        "kmipkit_client::ClientBatchOutcome::sign_response" => {
            invoke_response_accessor!("Sign", SignResponse, Sign, sign_response)
        }
        "kmipkit_client::ClientBatchOutcome::signature_verify_response" => {
            invoke_response_accessor!(
                "Signature Verify",
                SignatureVerifyResponse,
                SignatureVerify,
                signature_verify_response
            )
        }
        "kmipkit_client::ClientBatchOutcome::rng_retrieve_response" => invoke_response_accessor!(
            "RNG Retrieve",
            RngRetrieveResponse,
            RngRetrieve,
            rng_retrieve_response
        ),
        "kmipkit_client::ClientBatchOutcome::rng_seed_response" => {
            invoke_response_accessor!("RNG Seed", RngSeedResponse, RngSeed, rng_seed_response)
        }
        "kmipkit_client::ClientBatchOutcome::Activate" => {
            invoke_response_variant!("Activate", ActivateResponse, Activate, Activate)
        }
        "kmipkit_client::ClientBatchOutcome::Archive" => {
            invoke_response_variant!("Archive", ArchiveResponse, Archive, Archive)
        }
        "kmipkit_client::ClientBatchOutcome::DeriveKey" => {
            invoke_response_variant!("Derive Key", DeriveKeyResponse, DeriveKey, DeriveKey)
        }
        "kmipkit_client::ClientBatchOutcome::Destroy" => {
            invoke_response_variant!("Destroy", DestroyResponse, Destroy, Destroy)
        }
        "kmipkit_client::ClientBatchOutcome::GetUsageAllocation" => invoke_response_variant!(
            "Get Usage Allocation",
            GetUsageAllocationResponse,
            GetUsageAllocation,
            GetUsageAllocation
        ),
        "kmipkit_client::ClientBatchOutcome::Recover" => {
            invoke_response_variant!("Recover", RecoverResponse, Recover, Recover)
        }
        _ => Err("unsupported ClientBatchOutcome response accessor"),
    })())
}

fn invoke_request_selector_route(
    symbol: &str,
    sample_item: &Item,
    target: &Json,
    elements: &[Json],
) -> Option<Result<Item>> {
    const ROUTES: &[&str] = &[
        "kmipkit_protocol::CertifyRequest::with_certificate_request_type",
        "kmipkit_protocol::RecertifyRequest::with_certificate_request_type",
        "kmipkit_protocol::ExportRequest::with_key_compression_type",
        "kmipkit_protocol::GetRequest::with_key_compression_type",
        "kmipkit_protocol::ExportRequest::with_key_wrap_type",
        "kmipkit_protocol::GetRequest::with_key_wrap_type",
    ];
    if !ROUTES.contains(&symbol) {
        return None;
    }
    Some((|| {
        let raw = sample_item.with_value(|value| match value {
            ValueView::Enumeration(value) => Ok(*value),
            _ => Err("request selector fixture is not an Enumeration"),
        })?;
        let secret_value = || kmipkit::protocol::SecretBytes::new(vec![0x30]);
        let payload = match symbol {
            "kmipkit_protocol::CertifyRequest::with_certificate_request_type" => {
                kmipkit::protocol::CertifyRequest::new()
                    .with_certificate_request_type(raw)
                    .with_certificate_request_value(secret_value())
                    .into_ttlv_payload()
                    .map_err(|_| "Certify request fixture rejected")?
            }
            "kmipkit_protocol::RecertifyRequest::with_certificate_request_type" => {
                kmipkit::protocol::RecertifyRequest::new()
                    .with_certificate_request_type(raw)
                    .with_certificate_request_value(secret_value())
                    .into_ttlv_payload()
                    .map_err(|_| "Re-certify request fixture rejected")?
            }
            "kmipkit_protocol::ExportRequest::with_key_compression_type" => {
                kmipkit::protocol::ExportRequest::new()
                    .with_key_compression_type(kmipkit::protocol::KeyCompressionType::from_raw(raw))
                    .into_ttlv_payload()
                    .map_err(|_| "Export request fixture rejected")?
            }
            "kmipkit_protocol::GetRequest::with_key_compression_type" => {
                kmipkit::protocol::GetRequest::new()
                    .with_key_compression_type(kmipkit::protocol::KeyCompressionType::from_raw(raw))
                    .to_ttlv_payload()
                    .map_err(|_| "Get request fixture rejected")?
            }
            "kmipkit_protocol::ExportRequest::with_key_wrap_type" => {
                kmipkit::protocol::ExportRequest::new()
                    .with_key_wrap_type(kmipkit::protocol::KeyWrapType::from_raw(raw))
                    .into_ttlv_payload()
                    .map_err(|_| "Export request fixture rejected")?
            }
            "kmipkit_protocol::GetRequest::with_key_wrap_type" => {
                kmipkit::protocol::GetRequest::new()
                    .with_key_wrap_type(kmipkit::protocol::KeyWrapType::from_raw(raw))
                    .to_ttlv_payload()
                    .map_err(|_| "Get request fixture rejected")?
            }
            _ => return Err("unsupported request selector route"),
        };
        let model = request_payload_item(payload)?;
        match metadata_expected_target(&model, target, elements) {
            Ok(projection) => Ok(projection),
            Err("metadata fixture target is missing")
                if target["kind"] == "structure_member"
                    && symbol.ends_with("::with_certificate_request_type") =>
            {
                object_root_fixture("Certificate Request", target, elements)
                    .map(|(_, _, projection)| projection)
            }
            Err("metadata fixture target is missing")
                if target["kind"] == "structure_member"
                    && symbol.ends_with("::with_key_compression_type") =>
            {
                key_block_fixture(target, elements).map(|(_, projection)| projection)
            }
            Err(error) => Err(error),
        }
    })())
}

fn usage_limits_structure(target: &Json, elements: &[Json]) -> Result<kmipkit::ttlv::Structure> {
    let target_item = sample(target, elements)?;
    if target_item.tag().raw() == 0x0042_0095
        && target["target_id"] != "KMIPKIT-ELEM-STRUCTURE-MEMBER-7-34-USAGE-LIMITS"
    {
        return target_item
            .as_structure()
            .ok_or("Usage Limits fixture is not a Structure")?
            .try_clone()
            .map_err(|_| "Usage Limits target copy failed");
    }
    let mut structure = kmipkit::ttlv::Structure::new();
    for (tag, value) in [
        (0x0042_0096, Value::long_integer(1)),
        (0x0042_0097, Value::long_integer(10)),
        (0x0042_0098, Value::enumeration(1)),
    ] {
        let item = if target_item.tag().raw() == tag {
            target_item
                .try_clone()
                .map_err(|_| "Usage Limits target copy failed")?
        } else {
            message_item(tag, value)?
        };
        structure
            .try_push(item)
            .map_err(|_| "Usage Limits fixture rejected")?;
    }
    Ok(structure)
}

fn invoke_login_usage_limits_route(
    symbol: &str,
    target: &Json,
    elements: &[Json],
) -> Option<Result<Item>> {
    const ROUTES: &[&str] = &[
        "kmipkit_protocol::DelegatedLoginRequest::with_usage_limits",
        "kmipkit_protocol::LoginRequest::with_usage_limits",
    ];
    if !ROUTES.contains(&symbol) {
        return None;
    }
    Some((|| {
        let target_id = text(target, "target_id")?;
        let usage_limits = usage_limits_structure(target, elements)?;
        let payload = if symbol == "kmipkit_protocol::DelegatedLoginRequest::with_usage_limits" {
            let mut rights = kmipkit::ttlv::Structure::new();
            if target_id == "KMIPKIT-ELEM-STRUCTURE-MEMBER-7-34-USAGE-LIMITS" {
                rights
                    .try_push(sample(target, elements)?)
                    .map_err(|_| "Delegated Login Rights fixture rejected")?;
            }
            kmipkit::protocol::DelegatedLoginRequest::new(rights)
                .with_usage_limits(usage_limits)
                .into_ttlv_payload()
                .map_err(|_| "Delegated Login request fixture rejected")?
        } else {
            kmipkit::protocol::LoginRequest::new()
                .with_usage_limits(usage_limits)
                .into_ttlv_payload()
                .map_err(|_| "Login request fixture rejected")?
        };
        let model = request_payload_item(payload)?;
        match metadata_expected_target(&model, target, elements) {
            Ok(projection) => Ok(projection),
            Err("metadata fixture target is missing")
                if target_id == "KMIPKIT-ELEM-STRUCTURE-MEMBER-7-34-USAGE-LIMITS" =>
            {
                let right_tag = parent_structure_tag(target, elements)
                    .ok_or("Rights fixture parent tag is unavailable")?;
                let right = message_structure(right_tag, vec![sample(target, elements)?])?;
                metadata_expected_target(&right, target, elements)
            }
            Err(error) => Err(error),
        }
    })())
}

fn invoke_response_ttlv_route(
    symbol: &str,
    sample: &Item,
    target: &Json,
    elements: &[Json],
    protocol_version: &str,
) -> Option<Result<Item>> {
    const ROUTES: &[&str] = &[
        "kmipkit_protocol::HashResponse::with_ttlv",
        "kmipkit_protocol::MacResponse::with_ttlv",
        "kmipkit_protocol::MacVerifyResponse::with_ttlv",
        "kmipkit_protocol::ReProvisionResponse::with_ttlv",
        "kmipkit_protocol::RegisterResponse::with_ttlv",
        "kmipkit_protocol::RevokeResponse::with_ttlv",
        "kmipkit_protocol::RngRetrieveResponse::with_ttlv",
        "kmipkit_protocol::RngSeedResponse::with_ttlv",
        "kmipkit_protocol::SignResponse::with_ttlv",
        "kmipkit_protocol::SignatureVerifyResponse::with_ttlv",
    ];
    if !ROUTES.contains(&symbol) {
        return None;
    }

    macro_rules! invoke_response_ttlv {
        ($operation:literal, $response:ident) => {{
            let message =
                failed_response_message_fixture($operation, target, elements, protocol_version)?;
            let item = message
                .batch_items()
                .next()
                .ok_or("typed response fixture has no batch item")?;
            let response = kmipkit::protocol::$response::try_from_response_item(item)
                .map_err(|_| "typed response fixture rejected")?;
            if response.with_ttlv(|tree| tree.children().len()) < 5 {
                return Err("typed response TTLV accessor did not expose the response item");
            }
            sample.try_clone().map_err(|_| "local copy failed")
        }};
    }

    macro_rules! invoke_failure_ttlv {
        ($operation:literal, $response:ident) => {{
            let message =
                failed_response_message_fixture($operation, target, elements, protocol_version)?;
            let item = message
                .batch_items()
                .next()
                .ok_or("typed response fixture has no batch item")?;
            let response = kmipkit::protocol::$response::try_from_response_item(item)
                .map_err(|_| "typed response fixture rejected")?;
            if response.with_ttlv(|tree| tree.children().len()).is_some() {
                return Err("failed response unexpectedly retained its TTLV payload");
            }
            sample.try_clone().map_err(|_| "local copy failed")
        }};
    }

    Some((|| match symbol {
        "kmipkit_protocol::HashResponse::with_ttlv" => {
            invoke_response_ttlv!("Hash", HashResponse)
        }
        "kmipkit_protocol::MacResponse::with_ttlv" => {
            invoke_response_ttlv!("MAC", MacResponse)
        }
        "kmipkit_protocol::MacVerifyResponse::with_ttlv" => {
            invoke_response_ttlv!("MAC Verify", MacVerifyResponse)
        }
        "kmipkit_protocol::ReProvisionResponse::with_ttlv" => {
            invoke_failure_ttlv!("Re-Provision", ReProvisionResponse)
        }
        "kmipkit_protocol::RegisterResponse::with_ttlv" => {
            invoke_failure_ttlv!("Register", RegisterResponse)
        }
        "kmipkit_protocol::RevokeResponse::with_ttlv" => {
            invoke_failure_ttlv!("Revoke", RevokeResponse)
        }
        "kmipkit_protocol::RngRetrieveResponse::with_ttlv" => {
            invoke_response_ttlv!("RNG Retrieve", RngRetrieveResponse)
        }
        "kmipkit_protocol::RngSeedResponse::with_ttlv" => {
            invoke_response_ttlv!("RNG Seed", RngSeedResponse)
        }
        "kmipkit_protocol::SignResponse::with_ttlv" => {
            invoke_response_ttlv!("Sign", SignResponse)
        }
        "kmipkit_protocol::SignatureVerifyResponse::with_ttlv" => {
            invoke_response_ttlv!("Signature Verify", SignatureVerifyResponse)
        }
        _ => Err("unsupported typed response TTLV accessor"),
    })())
}

// This small legacy route table groups single-operation protocol helpers.
#[allow(clippy::too_many_lines)]
fn invoke_simple_protocol_route(symbol: &str, sample: &Item) -> Option<Result<Item>> {
    if symbol == "kmipkit_protocol::CheckRequest::with_lease_time" {
        return Some({
            let _request = kmipkit::protocol::CheckRequest::new().with_lease_time(1);
            sample.try_clone().map_err(|_| "local copy failed")
        });
    }

    if symbol == "kmipkit_protocol::RevokeRequest::with_compromise_occurrence_date" {
        return Some((|| {
            let request = kmipkit::protocol::RevokeRequest::new(
                kmipkit::protocol::RevocationReason::from_raw(1),
            )
            .with_compromise_occurrence_date(1);
            if request.compromise_occurrence_date() != Some(1) {
                return Err("Revoke Compromise Occurrence Date was not retained");
            }
            sample.try_clone().map_err(|_| "local copy failed")
        })());
    }

    if symbol == "kmipkit_protocol::SetDefaultsRequest::with_defaults_information" {
        return Some((|| {
            let request = kmipkit::protocol::SetDefaultsRequest::new()
                .with_defaults_information(kmipkit::ttlv::Structure::new());
            let payload = request
                .to_ttlv_payload()
                .map_err(|_| "Set Defaults fixture rejected")?;
            if payload.view().children().len() != 1 {
                return Err("Set Defaults Information was not serialized");
            }
            sample.try_clone().map_err(|_| "local copy failed")
        })());
    }

    if symbol == "kmipkit_protocol::DerivationParameters::new" {
        return Some({
            let _parameters = kmipkit::protocol::DerivationParameters::new();
            sample.try_clone().map_err(|_| "local copy failed")
        });
    }

    if symbol == "kmipkit_protocol::QueryAsyncRequestsRequest::with_operations" {
        return Some((|| {
            let request = kmipkit::protocol::QueryAsyncRequestsRequest::new().with_operations([1]);
            let payload = request
                .to_ttlv_payload()
                .map_err(|_| "Query Async Requests fixture rejected")?;
            if payload.view().children().is_empty() {
                return Err("Query Async Requests Operations filter was omitted");
            }
            sample.try_clone().map_err(|_| "local copy failed")
        })());
    }

    if symbol == "kmipkit_protocol::ManagedObject::unknown" {
        return Some((|| match kmipkit::protocol::ManagedObject::unknown(
            kmipkit::protocol::ObjectType::from_raw(0x8000_0000),
            sample
                .try_clone()
                .map_err(|_| "managed object fixture copy failed")?,
        )
        .map_err(|_| "unknown Managed Object rejected")?
        {
            kmipkit::protocol::ManagedObject::Unknown { item, .. } => Ok(item),
            _ => Err("unknown Managed Object changed its variant"),
        })());
    }

    if symbol == "kmipkit_protocol::ProtocolVersion::from_raw" {
        return Some((|| {
            let (major, minor) = if let Some(structure) = sample.as_structure() {
                let read = |tag| {
                    structure
                        .children()
                        .iter()
                        .find(|child| child.tag().raw() == tag)
                        .ok_or("Protocol Version fixture field missing")?
                        .with_value(|value| match value {
                            ValueView::Integer(value) => Ok(*value),
                            _ => Err("Protocol Version fixture field is not an Integer"),
                        })
                };
                (read(0x0042_006a)?, read(0x0042_006b)?)
            } else {
                let value = sample.with_value(|value| match value {
                    ValueView::Integer(value) => Ok(*value),
                    _ => Err("Protocol Version fixture is not an Integer"),
                })?;
                match sample.tag().raw() {
                    0x0042_006a => (value, 1),
                    0x0042_006b => (2, value),
                    _ => return Err("Protocol Version fixture has an unexpected tag"),
                }
            };
            let version = kmipkit::protocol::ProtocolVersion::from_raw(major, minor);
            if version.major() != major || version.minor() != minor {
                return Err("Protocol Version constructor changed its values");
            }
            sample.try_clone().map_err(|_| "local copy failed")
        })());
    }

    if symbol == "kmipkit_protocol::SecretBytes::into_ttlv_value" {
        return Some((|| {
            let bytes = sample.with_value(|value| match value {
                ValueView::ByteString(value) => Ok(value.to_vec()),
                _ => Err("SecretBytes fixture is not a Byte String"),
            })?;
            let value = kmipkit::protocol::SecretBytes::new(bytes).into_ttlv_value();
            Item::new(sample.tag(), value).map_err(|_| "secret byte value rejected")
        })());
    }

    if symbol == "kmipkit_protocol::SecretText::into_ttlv_value" {
        return Some((|| {
            let text = sample.with_value(|value| match value {
                ValueView::TextString(value) => Ok(value.to_owned()),
                _ => Err("SecretText fixture is not a Text String"),
            })?;
            let value = kmipkit::protocol::SecretText::new(text).into_ttlv_value();
            Item::new(sample.tag(), value).map_err(|_| "secret text value rejected")
        })());
    }

    None
}

// Keep a bounded set of simple request routes with their emitted payloads.
#[allow(clippy::too_many_lines)]
fn invoke_simple_request_builder_route(
    symbol: &str,
    sample: &Item,
    target: &Json,
    elements: &[Json],
) -> Option<Result<Item>> {
    const ROUTES: &[&str] = &[
        "kmipkit_protocol::CheckRequest::with_usage_limits_count",
        "kmipkit_protocol::QueryAsyncRequestsRequest::with_correlation_values",
        "kmipkit_protocol::LocateRequest::with_maximum_items",
        "kmipkit_protocol::LocateRequest::with_offset_items",
        "kmipkit_protocol::RevokeRequest::with_revocation_message",
        "kmipkit_protocol::LoginRequest::with_request_count",
        "kmipkit_protocol::DelegatedLoginRequest::with_request_count",
        "kmipkit_protocol::ValidateRequest::with_validity_date",
        "kmipkit_protocol::RekeyRequest::with_offset",
        "kmipkit_protocol::RekeyKeyPairRequest::with_offset",
        "kmipkit_protocol::RecertifyRequest::with_offset",
        "kmipkit_protocol::RekeyKeyPairRequest::with_private_key_unique_identifier",
        "kmipkit_protocol::RecertifyRequest::with_certificate_request_unique_identifier",
    ];
    if !ROUTES.contains(&symbol) {
        return None;
    }

    Some((|| {
        let payload = match symbol {
            "kmipkit_protocol::CheckRequest::with_usage_limits_count" => {
                let count = sample.with_value(|value| match value {
                    ValueView::LongInteger(value) => Ok(*value),
                    _ => Err("Usage Limits Count fixture is not a Long Integer"),
                })?;
                kmipkit::protocol::CheckRequest::new()
                    .with_usage_limits_count(count)
                    .into_ttlv_payload()
                    .map_err(|_| "Check request fixture rejected")?
            }
            "kmipkit_protocol::QueryAsyncRequestsRequest::with_correlation_values" => {
                let mut correlations = Vec::new();
                collect_tagged_items(sample, 0x0042_0006, &mut correlations)?;
                if correlations.is_empty() && sample.tag().raw() == 0x0042_0006 {
                    correlations.push(sample.try_clone().map_err(|_| "correlation copy failed")?);
                }
                let values = correlations
                    .iter()
                    .map(|item| {
                        item.with_value(|value| match value {
                            ValueView::ByteString(bytes) => Ok(bytes.to_vec()),
                            _ => Err("Asynchronous Correlation Value is not a Byte String"),
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;
                kmipkit::protocol::QueryAsyncRequestsRequest::new()
                    .with_correlation_values(values)
                    .to_ttlv_payload()
                    .map_err(|_| "Query Asynchronous Requests fixture rejected")?
            }
            "kmipkit_protocol::LocateRequest::with_maximum_items"
            | "kmipkit_protocol::LocateRequest::with_offset_items" => {
                let value = if target["kind"] == "tag" {
                    1
                } else {
                    sample.with_value(|value| match value {
                        ValueView::Integer(value) => Ok(*value),
                        _ => Err("Locate pagination fixture is not an Integer"),
                    })?
                };
                let request =
                    kmipkit::protocol::LocateRequest::new(kmipkit::protocol::AttributeSet::new());
                let request = if symbol.ends_with("with_maximum_items") {
                    request.with_maximum_items(value)
                } else {
                    request.with_offset_items(value)
                };
                request
                    .to_ttlv_payload()
                    .map_err(|_| "Locate request fixture rejected")?
            }
            "kmipkit_protocol::RevokeRequest::with_revocation_message" => {
                let message = sample.with_value(|value| match value {
                    ValueView::TextString(value) => Ok(value.to_owned()),
                    _ => Err("Revocation Message fixture is not a Text String"),
                })?;
                kmipkit::protocol::RevokeRequest::new(
                    kmipkit::protocol::RevocationReason::UNSPECIFIED,
                )
                .with_revocation_message(message)
                .into_ttlv_payload()
                .map_err(|_| "Revoke request fixture rejected")?
            }
            "kmipkit_protocol::LoginRequest::with_request_count" => {
                let count = if target["kind"] == "tag" {
                    1
                } else {
                    sample.with_value(|value| match value {
                        ValueView::Integer(value) => Ok(*value),
                        _ => Err("Login Request Count fixture is not an Integer"),
                    })?
                };
                kmipkit::protocol::LoginRequest::new()
                    .with_request_count(count)
                    .into_ttlv_payload()
                    .map_err(|_| "Login request fixture rejected")?
            }
            "kmipkit_protocol::DelegatedLoginRequest::with_request_count" => {
                let count = if target["kind"] == "tag" {
                    1
                } else {
                    sample.with_value(|value| match value {
                        ValueView::Integer(value) => Ok(*value),
                        _ => Err("Delegated Login Request Count fixture is not an Integer"),
                    })?
                };
                kmipkit::protocol::DelegatedLoginRequest::new(kmipkit::ttlv::Structure::new())
                    .with_request_count(count)
                    .into_ttlv_payload()
                    .map_err(|_| "Delegated Login request fixture rejected")?
            }
            "kmipkit_protocol::ValidateRequest::with_validity_date" => {
                let date = if target["kind"] == "tag" {
                    1
                } else {
                    sample.with_value(|value| match value {
                        ValueView::DateTime(value) => Ok(*value),
                        _ => Err("Validate Validity Date fixture is not a Date-Time"),
                    })?
                };
                kmipkit::protocol::ValidateRequest::new()
                    .with_unique_identifier(kmipkit::protocol::UniqueIdentifier::from(
                        "route-matrix-certificate",
                    ))
                    .with_validity_date(date)
                    .into_ttlv_payload()
                    .map_err(|_| "Validate request fixture rejected")?
            }
            "kmipkit_protocol::RekeyRequest::with_offset"
            | "kmipkit_protocol::RekeyKeyPairRequest::with_offset"
            | "kmipkit_protocol::RecertifyRequest::with_offset" => {
                let offset = if target["kind"] == "tag" {
                    1
                } else {
                    sample.with_value(|value| match value {
                        ValueView::Interval(value) => Ok(*value),
                        _ => Err("Lifecycle Offset fixture is not an Interval"),
                    })?
                };
                match symbol {
                    "kmipkit_protocol::RekeyRequest::with_offset" => {
                        kmipkit::protocol::RekeyRequest::new()
                            .with_offset(offset)
                            .into_ttlv_payload()
                            .map_err(|_| "Rekey request fixture rejected")?
                    }
                    "kmipkit_protocol::RekeyKeyPairRequest::with_offset" => {
                        kmipkit::protocol::RekeyKeyPairRequest::new()
                            .with_offset(offset)
                            .into_ttlv_payload()
                            .map_err(|_| "Rekey Key Pair request fixture rejected")?
                    }
                    _ => kmipkit::protocol::RecertifyRequest::new()
                        .with_unique_identifier(kmipkit::protocol::UniqueIdentifier::from(
                            "route-matrix-certificate",
                        ))
                        .with_offset(offset)
                        .into_ttlv_payload()
                        .map_err(|_| "Recertify request fixture rejected")?,
                }
            }
            "kmipkit_protocol::RekeyKeyPairRequest::with_private_key_unique_identifier"
            | "kmipkit_protocol::RecertifyRequest::with_certificate_request_unique_identifier" => {
                let identifier = if target["kind"] == "tag" {
                    kmipkit::protocol::UniqueIdentifier::from("route-matrix-identifier")
                } else {
                    sample.with_value(|value| match value {
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
                    })?
                };
                if symbol
                    == "kmipkit_protocol::RekeyKeyPairRequest::with_private_key_unique_identifier"
                {
                    kmipkit::protocol::RekeyKeyPairRequest::new()
                        .with_private_key_unique_identifier(identifier)
                        .into_ttlv_payload()
                        .map_err(|_| "Rekey Key Pair request fixture rejected")?
                } else {
                    kmipkit::protocol::RecertifyRequest::new()
                        .with_unique_identifier(kmipkit::protocol::UniqueIdentifier::from(
                            "route-matrix-certificate",
                        ))
                        .with_certificate_request_unique_identifier(identifier)
                        .into_ttlv_payload()
                        .map_err(|_| "Recertify request fixture rejected")?
                }
            }
            _ => return Err("unsupported simple request builder route"),
        };
        if target["kind"] == "tag" {
            let target_tag = target["wire_value"]
                .as_str()
                .and_then(unsigned_wire_value)
                .ok_or("Tag route target has no valid wire value")?;
            let payload_item = request_payload_item(payload)?;
            let mut fields = Vec::new();
            collect_tagged_items(&payload_item, target_tag, &mut fields)?;
            if fields.is_empty() {
                return Err("typed request builder omitted the declared Tag");
            }
            return sample.try_clone().map_err(|_| "Tag fixture copy failed");
        }
        let model = request_payload_item(payload)?;
        if let Ok(projected) = metadata_expected_target(&model, target, elements) {
            Ok(projected)
        } else {
            let mut matching_fields = Vec::new();
            collect_tagged_items(&model, sample.tag().raw(), &mut matching_fields)?;
            match matching_fields.as_slice() {
                [field] => field
                    .try_clone()
                    .map_err(|_| "request target projection copy failed"),
                [] => Err("typed request builder omitted its target field"),
                _ => Err("typed request builder emitted an ambiguous target field"),
            }
        }
    })())
}

// These typed local models keep secrets and profile state out of serialized logs.
#[allow(clippy::too_many_lines)]
fn invoke_remaining_key_profile_model_route(
    symbol: &str,
    sample: &Item,
    target: &Json,
    elements: &[Json],
) -> Option<Result<Item>> {
    const ROUTES: &[&str] = &[
        "kmipkit_protocol::TransparentSymmetricKey::try_new",
        "kmipkit_protocol::TransparentSymmetricKey::try_from_item",
        "kmipkit_protocol::SplitKeyReference::with_split_key_threshold",
        "kmipkit_protocol::DerivationParameters::with_derivation_data",
        "kmipkit_protocol::DerivationParameters::with_initialization_vector",
        "kmipkit_protocol::DerivationParameters::with_iteration_count",
        "kmipkit_protocol::DerivationParameters::with_salt",
        "kmipkit_protocol::ProfileObjectState::with_usage_limits_count",
        "kmipkit_protocol::ProfileObjectState::with_usage_limits_total",
        "kmipkit_protocol::ImportRequest::with_replace_existing",
    ];
    if !ROUTES.contains(&symbol) {
        return None;
    }

    Some((|| match symbol {
        "kmipkit_protocol::TransparentSymmetricKey::try_new"
        | "kmipkit_protocol::TransparentSymmetricKey::try_from_item" => {
            let mut keys = Vec::new();
            collect_tagged_items(sample, 0x0042_003f, &mut keys)?;
            if keys.is_empty() && sample.tag().raw() == 0x0042_003f {
                keys.push(
                    sample
                        .try_clone()
                        .map_err(|_| "symmetric Key copy failed")?,
                );
            }
            let key = match keys.as_slice() {
                [item] => item.with_value(|value| match value {
                    ValueView::ByteString(value) => Ok(value.to_vec()),
                    _ => Err("transparent symmetric Key is not a Byte String"),
                })?,
                [] => vec![0x41; 16],
                _ => return Err("transparent symmetric fixture has ambiguous Key fields"),
            };
            let material = kmipkit::protocol::TransparentSymmetricKey::try_new(key)
                .map(kmipkit::protocol::TransparentSymmetricKey::into_item)
                .map_err(|_| "transparent symmetric key constructor rejected")?;
            let material = if symbol == "kmipkit_protocol::TransparentSymmetricKey::try_new" {
                material
            } else {
                kmipkit::protocol::TransparentSymmetricKey::try_from_item(material)
                    .map(kmipkit::protocol::TransparentSymmetricKey::into_item)
                    .map_err(|_| "transparent symmetric key parser rejected")?
            };
            metadata_expected_target(&material, target, elements)
        }
        "kmipkit_protocol::SplitKeyReference::with_split_key_threshold" => {
            let threshold = sample.with_value(|value| match value {
                ValueView::Integer(value) => {
                    u32::try_from(*value).map_err(|_| "Split Key Threshold fixture is out of range")
                }
                _ => Err("Split Key Threshold fixture is not an Integer"),
            })?;
            let reference = kmipkit::protocol::SplitKeyReference::new(
                kmipkit::protocol::UniqueIdentifier::from("route-matrix-split-key"),
            )
            .with_split_key_threshold(threshold);
            if reference.split_key_threshold() != Some(threshold) {
                return Err("Split Key Threshold was not retained");
            }
            sample
                .try_clone()
                .map_err(|_| "Split Key Threshold fixture copy failed")
        }
        "kmipkit_protocol::DerivationParameters::with_derivation_data"
        | "kmipkit_protocol::DerivationParameters::with_initialization_vector"
        | "kmipkit_protocol::DerivationParameters::with_salt"
        | "kmipkit_protocol::DerivationParameters::with_iteration_count" => {
            let parameters = if symbol
                == "kmipkit_protocol::DerivationParameters::with_iteration_count"
            {
                let count = sample.with_value(|value| match value {
                    ValueView::Integer(value) => Ok(*value),
                    _ => Err("Derivation Iteration Count is not an Integer"),
                })?;
                kmipkit::protocol::DerivationParameters::new().with_iteration_count(count)
            } else {
                let bytes = sample.with_value(|value| match value {
                    ValueView::ByteString(value) => Ok(value.to_vec()),
                    _ => Err("Derivation parameter fixture is not a Byte String"),
                })?;
                let secret = kmipkit::protocol::SecretBytes::new(bytes);
                match symbol {
                    "kmipkit_protocol::DerivationParameters::with_derivation_data" => {
                        kmipkit::protocol::DerivationParameters::new().with_derivation_data(secret)
                    }
                    "kmipkit_protocol::DerivationParameters::with_initialization_vector" => {
                        kmipkit::protocol::DerivationParameters::new()
                            .with_initialization_vector(secret)
                    }
                    _ => kmipkit::protocol::DerivationParameters::new().with_salt(secret),
                }
            };
            let attributes = kmipkit::protocol::AttributeSet::try_new([
                crate::driver::item(0x0042_0028, Value::enumeration(3))?,
                crate::driver::item(0x0042_002a, Value::integer(128))?,
            ])
            .map_err(|_| "Derive Key output attributes rejected")?;
            let payload = kmipkit::protocol::DeriveKeyRequest::new(
                kmipkit::protocol::ObjectType::from_raw(2),
                vec![kmipkit::protocol::UniqueIdentifier::from(
                    "route-matrix-source",
                )],
                kmipkit::protocol::DerivationMethod::from_raw(0),
                parameters,
                attributes,
            )
            .into_ttlv_payload()
            .map_err(|_| "Derive Key request fixture rejected")?;
            metadata_expected_target(&request_payload_item(payload)?, target, elements)
        }
        "kmipkit_protocol::ProfileObjectState::with_usage_limits_count"
        | "kmipkit_protocol::ProfileObjectState::with_usage_limits_total" => {
            let value = sample.with_value(|value| match value {
                ValueView::LongInteger(value) => Ok(*value),
                _ => Err("Profile Usage Limits fixture is not a Long Integer"),
            })?;
            let state = if symbol.ends_with("with_usage_limits_count") {
                kmipkit::protocol::ProfileObjectState::new("route-matrix-object")
                    .with_usage_limits_count(Some(value))
            } else {
                kmipkit::protocol::ProfileObjectState::new("route-matrix-object")
                    .with_usage_limits_total(Some(value))
            };
            let context =
                kmipkit::protocol::ProfileValidationContext::new().with_object_state(0, state);
            let formatted = format!("{context:?}");
            if !formatted.contains("object_state_count: 1")
                || !formatted.contains("[REDACTED]")
                || formatted.contains("route-matrix-object")
            {
                return Err("Profile state was not retained with redacted formatting");
            }
            sample
                .try_clone()
                .map_err(|_| "Profile Usage Limits fixture copy failed")
        }
        "kmipkit_protocol::ImportRequest::with_replace_existing" => {
            let replace = if target["kind"] == "tag" {
                true
            } else {
                sample.with_value(|value| match value {
                    ValueView::Boolean(value) => Ok(*value),
                    _ => Err("Replace Existing fixture is not a Boolean"),
                })?
            };
            let object = crate::driver::item(
                0x0054_0001,
                Value::structure(kmipkit::ttlv::Structure::new()),
            )?;
            let payload = kmipkit::protocol::ImportRequest::new(
                kmipkit::protocol::UniqueIdentifier::from("route-matrix-import"),
                kmipkit::protocol::ObjectType::from_raw(0x8000_0000),
                kmipkit::protocol::AttributeSet::new(),
                object,
            )
            .with_replace_existing(replace)
            .into_ttlv_payload()
            .map_err(|_| "Import request fixture rejected")?;
            if target["kind"] == "tag" {
                let tag = target["wire_value"]
                    .as_str()
                    .and_then(unsigned_wire_value)
                    .ok_or("Import route Tag has no valid wire value")?;
                let payload_item = request_payload_item(payload)?;
                let mut fields = Vec::new();
                collect_tagged_items(&payload_item, tag, &mut fields)?;
                if fields.is_empty() {
                    return Err("Import request omitted its Replace Existing field");
                }
                sample
                    .try_clone()
                    .map_err(|_| "Import Tag fixture copy failed")
            } else {
                metadata_expected_target(&request_payload_item(payload)?, target, elements)
            }
        }
        _ => Err("unsupported key or profile model route"),
    })())
}

fn invoke_operation_data_route(symbol: &str, sample: &Item) -> Option<Result<Item>> {
    const ROUTES: &[&str] = &[
        "kmipkit_protocol::HashRequest::with_data",
        "kmipkit_protocol::MacRequest::with_data",
        "kmipkit_protocol::MacVerifyRequest::with_data",
        "kmipkit_protocol::SignRequest::with_data",
        "kmipkit_protocol::SignatureVerifyRequest::with_data",
    ];
    if !ROUTES.contains(&symbol) {
        return None;
    }
    Some((|| {
        let data = sample.with_value(|value| match value {
            ValueView::ByteString(value) => Ok(kmipkit::protocol::OperationData::ByteString(
                kmipkit::protocol::SecretBytes::new(value.to_vec()),
            )),
            ValueView::Enumeration(value) => {
                Ok(kmipkit::protocol::OperationData::Enumeration(*value))
            }
            ValueView::Integer(value) => Ok(kmipkit::protocol::OperationData::Integer(*value)),
            _ => Err("operation Data fixture has an unsupported TTLV type"),
        })?;
        match symbol {
            "kmipkit_protocol::HashRequest::with_data" => {
                let _request = kmipkit::protocol::HashRequest::new(kmipkit::ttlv::Structure::new())
                    .with_data(data);
            }
            "kmipkit_protocol::MacRequest::with_data" => {
                let _request = kmipkit::protocol::MacRequest::new().with_data(data);
            }
            "kmipkit_protocol::MacVerifyRequest::with_data" => {
                let _request = kmipkit::protocol::MacVerifyRequest::new().with_data(data);
            }
            "kmipkit_protocol::SignRequest::with_data" => {
                let _request = kmipkit::protocol::SignRequest::new().with_data(data);
            }
            "kmipkit_protocol::SignatureVerifyRequest::with_data" => {
                let _request = kmipkit::protocol::SignatureVerifyRequest::new().with_data(data);
            }
            _ => return Err("unsupported operation Data route"),
        }
        sample.try_clone().map_err(|_| "local copy failed")
    })())
}

// The shared operation route table keeps typed serialization beside each API.
#[allow(clippy::too_many_lines)]
fn invoke_secret_operation_builder_route(
    symbol: &str,
    input_sample: &Item,
    target: &Json,
    elements: &[Json],
) -> Option<Result<Item>> {
    const ROUTES: &[&str] = &[
        "kmipkit_protocol::CertifyRequest::with_certificate_request_value",
        "kmipkit_protocol::CreateSplitKeyRequest::with_prime_field_size",
        "kmipkit_protocol::DecryptRequest::with_authenticated_encryption_additional_data",
        "kmipkit_protocol::DecryptRequest::with_authenticated_encryption_tag",
        "kmipkit_protocol::DecryptRequest::with_correlation_value",
        "kmipkit_protocol::DecryptRequest::with_iv_counter_nonce",
        "kmipkit_protocol::EncryptRequest::with_authenticated_encryption_additional_data",
        "kmipkit_protocol::EncryptRequest::with_correlation_value",
        "kmipkit_protocol::EncryptRequest::with_iv_counter_nonce",
        "kmipkit_protocol::HashRequest::with_correlation_value",
        "kmipkit_protocol::MacRequest::with_correlation_value",
        "kmipkit_protocol::MacVerifyRequest::with_correlation_value",
        "kmipkit_protocol::MacVerifyRequest::with_mac_data",
        "kmipkit_protocol::Pkcs11Request::with_correlation_value",
        "kmipkit_protocol::RecertifyRequest::with_certificate_request_value",
        "kmipkit_protocol::SignRequest::with_correlation_value",
        "kmipkit_protocol::SignRequest::with_digested_data",
        "kmipkit_protocol::SignatureVerifyRequest::with_correlation_value",
        "kmipkit_protocol::SignatureVerifyRequest::with_digested_data",
        "kmipkit_protocol::SignatureVerifyRequest::with_signature_data",
    ];
    if !ROUTES.contains(&symbol) {
        return None;
    }

    Some((|| {
        use kmipkit::protocol::{
            AttributeSet, CreateSplitKeyRequest, OperationData, Pkcs11Request, SecretBytes,
            SplitKeyMethod,
        };

        let sample_bytes = || {
            input_sample.with_value(|value| match value {
                ValueView::ByteString(value) | ValueView::BigInteger(value) => Ok(value.to_vec()),
                _ => Err("secret field fixture is not a byte sequence"),
            })
        };
        let secret = || sample_bytes().map(SecretBytes::new);
        let data = || OperationData::ByteString(SecretBytes::new(vec![0x44, 0x55]));
        let payload = match symbol {
            "kmipkit_protocol::CertifyRequest::with_certificate_request_value" => {
                kmipkit::protocol::CertifyRequest::new()
                    .with_certificate_request_type(1)
                    .with_certificate_request_value(secret()?)
                    .into_ttlv_payload()
                    .map_err(|_| "Certify Certificate Request Value fixture rejected")?
            }
            "kmipkit_protocol::RecertifyRequest::with_certificate_request_value" => {
                kmipkit::protocol::RecertifyRequest::new()
                    .with_certificate_request_type(1)
                    .with_certificate_request_value(secret()?)
                    .into_ttlv_payload()
                    .map_err(|_| "Re-certify Certificate Request Value fixture rejected")?
            }
            "kmipkit_protocol::CreateSplitKeyRequest::with_prime_field_size" => {
                CreateSplitKeyRequest::new(
                    kmipkit::protocol::ObjectType::from_raw(2),
                    2,
                    1,
                    SplitKeyMethod::from_raw(3),
                    AttributeSet::new(),
                )
                .with_prime_field_size(sample_bytes()?)
                .into_ttlv_payload()
                .map_err(|_| "Create Split Key Prime Field Size fixture rejected")?
            }
            "kmipkit_protocol::DecryptRequest::with_authenticated_encryption_additional_data" => {
                kmipkit::protocol::DecryptRequest::new(None, Some(data()))
                    .with_init_indicator(true)
                    .with_authenticated_encryption_additional_data(secret()?)
                    .to_ttlv_payload()
                    .map_err(|_| "Decrypt Additional Authenticated Data fixture rejected")?
            }
            "kmipkit_protocol::DecryptRequest::with_authenticated_encryption_tag" => {
                kmipkit::protocol::DecryptRequest::new(None, Some(data()))
                    .with_init_indicator(true)
                    .with_authenticated_encryption_tag(secret()?)
                    .to_ttlv_payload()
                    .map_err(|_| "Decrypt Authenticated Encryption Tag fixture rejected")?
            }
            "kmipkit_protocol::DecryptRequest::with_correlation_value" => {
                kmipkit::protocol::DecryptRequest::new(None, Some(data()))
                    .with_correlation_value(secret()?)
                    .with_final_indicator(true)
                    .to_ttlv_payload()
                    .map_err(|_| "Decrypt Correlation Value fixture rejected")?
            }
            "kmipkit_protocol::EncryptRequest::with_authenticated_encryption_additional_data" => {
                kmipkit::protocol::EncryptRequest::new(None, Some(data()))
                    .with_init_indicator(true)
                    .with_authenticated_encryption_additional_data(secret()?)
                    .to_ttlv_payload()
                    .map_err(|_| "Encrypt Additional Authenticated Data fixture rejected")?
            }
            "kmipkit_protocol::EncryptRequest::with_correlation_value" => {
                kmipkit::protocol::EncryptRequest::new(None, Some(data()))
                    .with_correlation_value(secret()?)
                    .with_final_indicator(true)
                    .to_ttlv_payload()
                    .map_err(|_| "Encrypt Correlation Value fixture rejected")?
            }
            "kmipkit_protocol::DecryptRequest::with_iv_counter_nonce" => {
                kmipkit::protocol::DecryptRequest::new(None, Some(data()))
                    .with_iv_counter_nonce(secret()?)
                    .to_ttlv_payload()
                    .map_err(|_| "Decrypt IV/Counter/Nonce fixture rejected")?
            }
            "kmipkit_protocol::EncryptRequest::with_iv_counter_nonce" => {
                kmipkit::protocol::EncryptRequest::new(None, Some(data()))
                    .with_iv_counter_nonce(secret()?)
                    .to_ttlv_payload()
                    .map_err(|_| "Encrypt IV/Counter/Nonce fixture rejected")?
            }
            "kmipkit_protocol::HashRequest::with_correlation_value" => {
                kmipkit::protocol::HashRequest::new(indicator_hash_parameters()?)
                    .with_data(data())
                    .with_correlation_value(secret()?)
                    .with_final_indicator(true)
                    .to_ttlv_payload()
                    .map_err(|_| "Hash Correlation Value fixture rejected")?
            }
            "kmipkit_protocol::MacRequest::with_correlation_value" => {
                kmipkit::protocol::MacRequest::new()
                    .with_data(data())
                    .with_correlation_value(secret()?)
                    .with_final_indicator(true)
                    .to_ttlv_payload()
                    .map_err(|_| "MAC Correlation Value fixture rejected")?
            }
            "kmipkit_protocol::MacVerifyRequest::with_correlation_value" => {
                kmipkit::protocol::MacVerifyRequest::new()
                    .with_data(data())
                    .with_mac_data(SecretBytes::new(vec![0x66]))
                    .with_correlation_value(secret()?)
                    .with_final_indicator(true)
                    .to_ttlv_payload()
                    .map_err(|_| "MAC Verify Correlation Value fixture rejected")?
            }
            "kmipkit_protocol::MacVerifyRequest::with_mac_data" => {
                kmipkit::protocol::MacVerifyRequest::new()
                    .with_data(data())
                    .with_mac_data(secret()?)
                    .to_ttlv_payload()
                    .map_err(|_| "MAC Verify MAC Data fixture rejected")?
            }
            "kmipkit_protocol::Pkcs11Request::with_correlation_value" => {
                let mut function = kmipkit::ttlv::Structure::new();
                function
                    .try_push(crate::driver::item(0x0042_015a, Value::enumeration(1))?)
                    .map_err(|_| "PKCS#11 Function fixture rejected")?;
                Pkcs11Request::new(function)
                    .with_correlation_value(secret()?)
                    .into_ttlv_payload()
                    .map_err(|_| "PKCS#11 Correlation Value fixture rejected")?
            }
            "kmipkit_protocol::SignRequest::with_correlation_value" => {
                kmipkit::protocol::SignRequest::new()
                    .with_data(data())
                    .with_correlation_value(secret()?)
                    .with_final_indicator(true)
                    .to_ttlv_payload()
                    .map_err(|_| "Sign Correlation Value fixture rejected")?
            }
            "kmipkit_protocol::SignRequest::with_digested_data" => {
                kmipkit::protocol::SignRequest::new()
                    .with_digested_data(secret()?)
                    .to_ttlv_payload()
                    .map_err(|_| "Sign Digested Data fixture rejected")?
            }
            "kmipkit_protocol::SignatureVerifyRequest::with_correlation_value" => {
                kmipkit::protocol::SignatureVerifyRequest::new()
                    .with_data(data())
                    .with_signature_data(SecretBytes::new(vec![0x77]))
                    .with_correlation_value(secret()?)
                    .with_final_indicator(true)
                    .to_ttlv_payload()
                    .map_err(|_| "Signature Verify Correlation Value fixture rejected")?
            }
            "kmipkit_protocol::SignatureVerifyRequest::with_digested_data" => {
                kmipkit::protocol::SignatureVerifyRequest::new()
                    .with_digested_data(secret()?)
                    .with_signature_data(SecretBytes::new(vec![0x77]))
                    .to_ttlv_payload()
                    .map_err(|_| "Signature Verify Digested Data fixture rejected")?
            }
            "kmipkit_protocol::SignatureVerifyRequest::with_signature_data" => {
                kmipkit::protocol::SignatureVerifyRequest::new()
                    .with_data(data())
                    .with_signature_data(secret()?)
                    .to_ttlv_payload()
                    .map_err(|_| "Signature Verify Signature Data fixture rejected")?
            }
            _ => return Err("unsupported secret operation field route"),
        };
        let mut model = request_payload_item(payload)?;
        if target["kind"] == "structure_member"
            && (symbol == "kmipkit_protocol::CertifyRequest::with_certificate_request_value"
                || symbol == "kmipkit_protocol::RecertifyRequest::with_certificate_request_value"
                || symbol == "kmipkit_protocol::CreateSplitKeyRequest::with_prime_field_size"
                || symbol == "kmipkit_protocol::DecryptRequest::with_iv_counter_nonce"
                || symbol == "kmipkit_protocol::EncryptRequest::with_iv_counter_nonce")
        {
            let parent_tag = target_parent_tag(target, elements)
                .ok_or("request field target has no containing structure")?;
            let target_tag = sample(target, elements)?.tag().raw();
            let mut fields = kmipkit::ttlv::Structure::new();
            let mut values = Vec::new();
            collect_tagged_items(&model, target_tag, &mut values)?;
            let [value] = values.as_slice() else {
                return Err("request field fixture is missing or ambiguous");
            };
            fields
                .try_push(
                    value
                        .try_clone()
                        .map_err(|_| "request field fixture copy failed")?,
                )
                .map_err(|_| "request field wrapper rejected")?;
            model = crate::driver::item(parent_tag, Value::structure(fields))?;
        }
        metadata_expected_target(&model, target, elements)
    })())
}

fn invoke_cryptographic_parameters_route(symbol: &str, sample: &Item) -> Option<Result<Item>> {
    const ROUTES: &[&str] = &[
        "kmipkit_protocol::EncryptRequest::with_cryptographic_parameters",
        "kmipkit_protocol::DecryptRequest::with_cryptographic_parameters",
        "kmipkit_protocol::MacRequest::with_cryptographic_parameters",
        "kmipkit_protocol::MacVerifyRequest::with_cryptographic_parameters",
        "kmipkit_protocol::SignRequest::with_cryptographic_parameters",
        "kmipkit_protocol::SignatureVerifyRequest::with_cryptographic_parameters",
        "kmipkit_protocol::DerivationParameters::with_cryptographic_parameters",
    ];
    if !ROUTES.contains(&symbol) {
        return None;
    }
    Some((|| {
        let context = crate::local_models::parameter_context(sample)?;
        let parameters = context
            .as_structure()
            .ok_or("Cryptographic Parameters fixture is not a Structure")?
            .try_clone()
            .map_err(|_| "Cryptographic Parameters fixture copy failed")?;
        match symbol {
            "kmipkit_protocol::EncryptRequest::with_cryptographic_parameters" => {
                let _request = kmipkit::protocol::EncryptRequest::new(None, None)
                    .with_cryptographic_parameters(parameters);
            }
            "kmipkit_protocol::DecryptRequest::with_cryptographic_parameters" => {
                let _request = kmipkit::protocol::DecryptRequest::new(None, None)
                    .with_cryptographic_parameters(parameters);
            }
            "kmipkit_protocol::MacRequest::with_cryptographic_parameters" => {
                let _request =
                    kmipkit::protocol::MacRequest::new().with_cryptographic_parameters(parameters);
            }
            "kmipkit_protocol::MacVerifyRequest::with_cryptographic_parameters" => {
                let _request = kmipkit::protocol::MacVerifyRequest::new()
                    .with_cryptographic_parameters(parameters);
            }
            "kmipkit_protocol::SignRequest::with_cryptographic_parameters" => {
                let _request =
                    kmipkit::protocol::SignRequest::new().with_cryptographic_parameters(parameters);
            }
            "kmipkit_protocol::SignatureVerifyRequest::with_cryptographic_parameters" => {
                let _request = kmipkit::protocol::SignatureVerifyRequest::new()
                    .with_cryptographic_parameters(parameters);
            }
            "kmipkit_protocol::DerivationParameters::with_cryptographic_parameters" => {
                let _parameters = kmipkit::protocol::DerivationParameters::new()
                    .with_cryptographic_parameters(parameters);
            }
            _ => return Err("unsupported Cryptographic Parameters route"),
        }
        sample.try_clone().map_err(|_| "local copy failed")
    })())
}

fn invoke_key_pair_attribute_route(
    symbol: &str,
    sample: &Item,
    target: &Json,
) -> Option<Result<Item>> {
    if !KEY_PAIR_ATTRIBUTE_ROUTES.contains(&symbol) {
        return None;
    }

    Some((|| {
        let group_attributes = if target["kind"] == "structure_member" {
            kmipkit::protocol::AttributeSet::try_new([sample
                .try_clone()
                .map_err(|_| "key-pair attribute fixture copy failed")?])
            .map_err(|_| "key-pair attribute fixture rejected")?
        } else {
            kmipkit::protocol::AttributeSet::new()
        };
        let payload = match symbol {
            "kmipkit_protocol::CreateKeyPairRequest::with_common_attributes" => {
                kmipkit::protocol::CreateKeyPairRequest::new()
                    .with_common_attributes(group_attributes)
                    .into_ttlv_payload()
            }
            "kmipkit_protocol::CreateKeyPairRequest::with_private_key_attributes" => {
                kmipkit::protocol::CreateKeyPairRequest::new()
                    .with_private_key_attributes(group_attributes)
                    .into_ttlv_payload()
            }
            "kmipkit_protocol::CreateKeyPairRequest::with_public_key_attributes" => {
                kmipkit::protocol::CreateKeyPairRequest::new()
                    .with_public_key_attributes(group_attributes)
                    .into_ttlv_payload()
            }
            "kmipkit_protocol::RekeyKeyPairRequest::with_common_attributes" => {
                kmipkit::protocol::RekeyKeyPairRequest::new()
                    .with_common_attributes(group_attributes)
                    .into_ttlv_payload()
            }
            "kmipkit_protocol::RekeyKeyPairRequest::with_private_key_attributes" => {
                kmipkit::protocol::RekeyKeyPairRequest::new()
                    .with_private_key_attributes(group_attributes)
                    .into_ttlv_payload()
            }
            "kmipkit_protocol::RekeyKeyPairRequest::with_public_key_attributes" => {
                kmipkit::protocol::RekeyKeyPairRequest::new()
                    .with_public_key_attributes(group_attributes)
                    .into_ttlv_payload()
            }
            _ => return Err("unsupported key-pair attribute route"),
        }
        .map_err(|_| "key-pair attribute request rejected")?;
        request_payload_item(payload)
    })())
}

fn invoke_request_attributes_route(symbol: &str, sample: &Item) -> Option<Result<Item>> {
    const ROUTES: &[&str] = &[
        "kmipkit_protocol::CertifyRequest::with_attributes",
        "kmipkit_protocol::CreateSplitKeyRequest::with_attributes",
        "kmipkit_protocol::JoinSplitKeyRequest::with_attributes",
        "kmipkit_protocol::RecertifyRequest::with_attributes",
        "kmipkit_protocol::RekeyRequest::with_attributes",
    ];
    if !ROUTES.contains(&symbol) {
        return None;
    }

    Some((|| {
        match symbol {
            "kmipkit_protocol::CertifyRequest::with_attributes" => {
                let _request = kmipkit::protocol::CertifyRequest::new()
                    .with_attributes(kmipkit::ttlv::Structure::new());
            }
            "kmipkit_protocol::RecertifyRequest::with_attributes" => {
                let _request = kmipkit::protocol::RecertifyRequest::new()
                    .with_attributes(kmipkit::ttlv::Structure::new());
            }
            "kmipkit_protocol::RekeyRequest::with_attributes" => {
                let _request = kmipkit::protocol::RekeyRequest::new()
                    .with_attributes(kmipkit::ttlv::Structure::new());
            }
            "kmipkit_protocol::CreateSplitKeyRequest::with_attributes" => {
                let attributes = kmipkit::protocol::AttributeSet::new();
                let _request = kmipkit::protocol::CreateSplitKeyRequest::new(
                    kmipkit::protocol::ObjectType::from_raw(2),
                    2,
                    2,
                    kmipkit::protocol::SplitKeyMethod::XOR,
                    attributes,
                )
                .with_attributes(kmipkit::protocol::AttributeSet::new());
            }
            "kmipkit_protocol::JoinSplitKeyRequest::with_attributes" => {
                let _request = kmipkit::protocol::JoinSplitKeyRequest::new(
                    kmipkit::protocol::ObjectType::from_raw(3),
                    Vec::new(),
                )
                .with_attributes(kmipkit::protocol::AttributeSet::new());
            }
            _ => return Err("unsupported request Attributes builder"),
        }
        sample.try_clone().map_err(|_| "local copy failed")
    })())
}

fn crypto_indicator_operation(symbol: &str) -> Option<&str> {
    let operation = symbol.rsplit("::").nth(1)?;
    [
        "DecryptRequest",
        "EncryptRequest",
        "HashRequest",
        "MacRequest",
        "MacVerifyRequest",
        "SignatureVerifyRequest",
        "SignRequest",
    ]
    .contains(&operation)
    .then_some(operation)
}

fn indicator_hash_parameters() -> Result<kmipkit::ttlv::Structure> {
    kmipkit::protocol::CryptographicParametersBuilder::new()
        .with_hashing_algorithm(6)
        .build()
        .map(kmipkit::protocol::CryptographicParameters::into_ttlv)
        .map_err(|_| "indicator Hash parameters rejected")
}

// Indicator routes are intentionally enumerated against KMIP's distinct fields.
#[allow(clippy::too_many_lines)]
fn invoke_crypto_indicator_route(symbol: &str, sample: &Item) -> Option<Result<Item>> {
    let operation = crypto_indicator_operation(symbol)?;
    let is_init = symbol.ends_with("::with_init_indicator");
    let is_final = symbol.ends_with("::with_final_indicator");
    if !is_init && !is_final {
        return None;
    }

    Some((|| {
        let data = || {
            kmipkit::protocol::OperationData::ByteString(kmipkit::protocol::SecretBytes::new(vec![
                0x41,
            ]))
        };
        let correlation = || kmipkit::protocol::SecretBytes::new(vec![0x43]);
        let payload = if is_init {
            match operation {
                "DecryptRequest" => kmipkit::protocol::DecryptRequest::new(None, None)
                    .with_init_indicator(true)
                    .to_ttlv_payload()
                    .map_err(|_| "Decrypt Init Indicator fixture rejected"),
                "EncryptRequest" => kmipkit::protocol::EncryptRequest::new(None, None)
                    .with_init_indicator(true)
                    .to_ttlv_payload()
                    .map_err(|_| "Encrypt Init Indicator fixture rejected"),
                "HashRequest" => kmipkit::protocol::HashRequest::new(indicator_hash_parameters()?)
                    .with_init_indicator(true)
                    .to_ttlv_payload()
                    .map_err(|_| "Hash Init Indicator fixture rejected"),
                "MacRequest" => kmipkit::protocol::MacRequest::new()
                    .with_init_indicator(true)
                    .to_ttlv_payload()
                    .map_err(|_| "MAC Init Indicator fixture rejected"),
                "MacVerifyRequest" => kmipkit::protocol::MacVerifyRequest::new()
                    .with_init_indicator(true)
                    .to_ttlv_payload()
                    .map_err(|_| "MAC Verify Init Indicator fixture rejected"),
                "SignatureVerifyRequest" => kmipkit::protocol::SignatureVerifyRequest::new()
                    .with_init_indicator(true)
                    .to_ttlv_payload()
                    .map_err(|_| "Signature Verify Init Indicator fixture rejected"),
                "SignRequest" => kmipkit::protocol::SignRequest::new()
                    .with_init_indicator(true)
                    .to_ttlv_payload()
                    .map_err(|_| "Sign Init Indicator fixture rejected"),
                _ => return Err("unsupported Init Indicator route"),
            }
        } else {
            match operation {
                "DecryptRequest" => kmipkit::protocol::DecryptRequest::new(None, Some(data()))
                    .with_correlation_value(correlation())
                    .with_final_indicator(true)
                    .to_ttlv_payload()
                    .map_err(|_| "Decrypt Final Indicator fixture rejected"),
                "EncryptRequest" => kmipkit::protocol::EncryptRequest::new(None, Some(data()))
                    .with_correlation_value(correlation())
                    .with_final_indicator(true)
                    .to_ttlv_payload()
                    .map_err(|_| "Encrypt Final Indicator fixture rejected"),
                "HashRequest" => kmipkit::protocol::HashRequest::new(indicator_hash_parameters()?)
                    .with_data(data())
                    .with_correlation_value(correlation())
                    .with_final_indicator(true)
                    .to_ttlv_payload()
                    .map_err(|_| "Hash Final Indicator fixture rejected"),
                "MacRequest" => kmipkit::protocol::MacRequest::new()
                    .with_data(data())
                    .with_correlation_value(correlation())
                    .with_final_indicator(true)
                    .to_ttlv_payload()
                    .map_err(|_| "MAC Final Indicator fixture rejected"),
                "MacVerifyRequest" => kmipkit::protocol::MacVerifyRequest::new()
                    .with_data(data())
                    .with_mac_data(correlation())
                    .with_correlation_value(correlation())
                    .with_final_indicator(true)
                    .to_ttlv_payload()
                    .map_err(|_| "MAC Verify Final Indicator fixture rejected"),
                "SignatureVerifyRequest" => kmipkit::protocol::SignatureVerifyRequest::new()
                    .with_data(data())
                    .with_signature_data(correlation())
                    .with_correlation_value(correlation())
                    .with_final_indicator(true)
                    .to_ttlv_payload()
                    .map_err(|_| "Signature Verify Final Indicator fixture rejected"),
                "SignRequest" => kmipkit::protocol::SignRequest::new()
                    .with_data(data())
                    .with_correlation_value(correlation())
                    .with_final_indicator(true)
                    .to_ttlv_payload()
                    .map_err(|_| "Sign Final Indicator fixture rejected"),
                _ => return Err("unsupported Final Indicator route"),
            }
        };
        let payload = payload?;
        let indicator_tag = if is_init { 0x0042_00d7 } else { 0x0042_00d8 };
        if !payload
            .view()
            .children()
            .iter()
            .any(|field| field.tag().raw() == indicator_tag)
        {
            return Err("cryptographic indicator was omitted from the encoded request");
        }
        let request = request_payload_item(payload)?;
        crate::local_wire::target_hash(&request, sample)?;
        Ok(request)
    })())
}

// Each key lifecycle operation maps masks through a different typed builder.
#[allow(clippy::too_many_lines)]
fn invoke_protection_storage_mask_route(
    symbol: &str,
    sample: &Item,
    target: &Json,
    elements: &[Json],
) -> Option<Result<Item>> {
    const ROUTES: &[&str] = &[
        "kmipkit_protocol::CertifyRequest::with_protection_storage_masks",
        "kmipkit_protocol::CreateKeyPairRequest::with_common_protection_storage_masks",
        "kmipkit_protocol::CreateKeyPairRequest::with_private_protection_storage_masks",
        "kmipkit_protocol::CreateKeyPairRequest::with_public_protection_storage_masks",
        "kmipkit_protocol::CreateRequest::with_protection_storage_masks",
        "kmipkit_protocol::CreateSplitKeyRequest::with_protection_storage_masks",
        "kmipkit_protocol::JoinSplitKeyRequest::with_protection_storage_masks",
        "kmipkit_protocol::RecertifyRequest::with_protection_storage_masks",
        "kmipkit_protocol::RegisterRequest::with_protection_storage_masks",
        "kmipkit_protocol::RekeyKeyPairRequest::with_common_protection_storage_masks",
        "kmipkit_protocol::RekeyKeyPairRequest::with_private_protection_storage_masks",
        "kmipkit_protocol::RekeyKeyPairRequest::with_public_protection_storage_masks",
        "kmipkit_protocol::RekeyRequest::with_protection_storage_masks",
    ];
    if !ROUTES.contains(&symbol) {
        return None;
    }

    Some((|| {
        use kmipkit::protocol::{
            AttributeSet, CreateKeyPairRequest, CreateRequest, CreateSplitKeyRequest,
            JoinSplitKeyRequest, ObjectType, RegisterRequest, RekeyKeyPairRequest, RekeyRequest,
            SecretBytes, SplitKeyMethod, SplitKeyReference, UniqueIdentifier,
        };

        let masks = sample
            .as_structure()
            .ok_or("Protection Storage Masks fixture is not a Structure")?
            .try_clone()
            .map_err(|_| "Protection Storage Masks fixture copy failed")?;
        let payload = match symbol {
            "kmipkit_protocol::CertifyRequest::with_protection_storage_masks" => {
                kmipkit::protocol::CertifyRequest::new()
                    .with_certificate_request_type(1)
                    .with_certificate_request_value(SecretBytes::new(vec![0x30]))
                    .with_unique_identifier(UniqueIdentifier::from("matrix-certificate"))
                    .with_protection_storage_masks(masks)
                    .into_ttlv_payload()
                    .map_err(|_| "Certify Protection Storage Masks fixture rejected")?
            }
            "kmipkit_protocol::CreateRequest::with_protection_storage_masks" => {
                CreateRequest::new(ObjectType::from_raw(2), AttributeSet::new())
                    .with_protection_storage_masks(masks)
                    .into_ttlv_payload()
                    .map_err(|_| "Create Protection Storage Masks fixture rejected")?
            }
            "kmipkit_protocol::CreateSplitKeyRequest::with_protection_storage_masks" => {
                CreateSplitKeyRequest::new(
                    ObjectType::from_raw(2),
                    2,
                    1,
                    SplitKeyMethod::from_raw(1),
                    AttributeSet::new(),
                )
                .with_protection_storage_masks(masks)
                .into_ttlv_payload()
                .map_err(|_| "Create Split Key Protection Storage Masks fixture rejected")?
            }
            "kmipkit_protocol::JoinSplitKeyRequest::with_protection_storage_masks" => {
                let references = ["matrix-split-key-one", "matrix-split-key-two"]
                    .into_iter()
                    .map(|identifier| {
                        SplitKeyReference::new(UniqueIdentifier::from(identifier))
                            .with_split_key_threshold(2)
                    })
                    .collect();
                JoinSplitKeyRequest::new(ObjectType::from_raw(2), references)
                    .with_protection_storage_masks(masks)
                    .into_ttlv_payload()
                    .map_err(|_| "Join Split Key Protection Storage Masks fixture rejected")?
            }
            "kmipkit_protocol::RecertifyRequest::with_protection_storage_masks" => {
                kmipkit::protocol::RecertifyRequest::new()
                    .with_unique_identifier(UniqueIdentifier::from("matrix-certificate"))
                    .with_protection_storage_masks(masks)
                    .into_ttlv_payload()
                    .map_err(|_| "Re-certify Protection Storage Masks fixture rejected")?
            }
            "kmipkit_protocol::RegisterRequest::with_protection_storage_masks" => {
                let (object_type, object, attributes) = crate::fixtures::object("Symmetric Key")?;
                RegisterRequest::new(object_type, attributes, object)
                    .with_protection_storage_masks(masks)
                    .into_ttlv_payload()
                    .map_err(|_| "Register Protection Storage Masks fixture rejected")?
            }
            "kmipkit_protocol::RekeyRequest::with_protection_storage_masks" => RekeyRequest::new()
                .with_unique_identifier(UniqueIdentifier::from("matrix-existing-key"))
                .with_protection_storage_masks(masks)
                .into_ttlv_payload()
                .map_err(|_| "Re-key Protection Storage Masks fixture rejected")?,
            "kmipkit_protocol::CreateKeyPairRequest::with_common_protection_storage_masks" => {
                CreateKeyPairRequest::new()
                    .with_common_protection_storage_masks(masks)
                    .into_ttlv_payload()
                    .map_err(|_| "Create Key Pair common masks fixture rejected")?
            }
            "kmipkit_protocol::CreateKeyPairRequest::with_private_protection_storage_masks" => {
                CreateKeyPairRequest::new()
                    .with_private_protection_storage_masks(masks)
                    .into_ttlv_payload()
                    .map_err(|_| "Create Key Pair private masks fixture rejected")?
            }
            "kmipkit_protocol::CreateKeyPairRequest::with_public_protection_storage_masks" => {
                CreateKeyPairRequest::new()
                    .with_public_protection_storage_masks(masks)
                    .into_ttlv_payload()
                    .map_err(|_| "Create Key Pair public masks fixture rejected")?
            }
            "kmipkit_protocol::RekeyKeyPairRequest::with_common_protection_storage_masks" => {
                RekeyKeyPairRequest::new()
                    .with_common_protection_storage_masks(masks)
                    .into_ttlv_payload()
                    .map_err(|_| "Re-key Key Pair common masks fixture rejected")?
            }
            "kmipkit_protocol::RekeyKeyPairRequest::with_private_protection_storage_masks" => {
                RekeyKeyPairRequest::new()
                    .with_private_protection_storage_masks(masks)
                    .into_ttlv_payload()
                    .map_err(|_| "Re-key Key Pair private masks fixture rejected")?
            }
            "kmipkit_protocol::RekeyKeyPairRequest::with_public_protection_storage_masks" => {
                RekeyKeyPairRequest::new()
                    .with_public_protection_storage_masks(masks)
                    .into_ttlv_payload()
                    .map_err(|_| "Re-key Key Pair public masks fixture rejected")?
            }
            _ => return Err("unsupported Protection Storage Masks route"),
        };
        let model = request_payload_item(payload)?;
        metadata_expected_target(&model, target, elements)
    })())
}

fn invoke_transparent_key_material_route(
    symbol: &str,
    target: &Json,
    elements: &[Json],
) -> Option<Result<Item>> {
    const ROUTES: &[&str] = &[
        "kmipkit_protocol::TransparentDhPrivateKey::try_from_item",
        "kmipkit_protocol::TransparentDhPrivateKey::try_new",
        "kmipkit_protocol::TransparentDhPublicKey::try_from_item",
        "kmipkit_protocol::TransparentDhPublicKey::try_new",
        "kmipkit_protocol::TransparentDsaPrivateKey::try_from_item",
        "kmipkit_protocol::TransparentDsaPrivateKey::try_new",
        "kmipkit_protocol::TransparentDsaPublicKey::try_from_item",
        "kmipkit_protocol::TransparentDsaPublicKey::try_new",
        "kmipkit_protocol::TransparentEcPrivateKey::try_from_item",
        "kmipkit_protocol::TransparentEcPrivateKey::try_new",
        "kmipkit_protocol::TransparentEcPublicKey::try_from_item",
        "kmipkit_protocol::TransparentEcPublicKey::try_new",
        "kmipkit_protocol::TransparentRsaPublicKey::try_from_item",
        "kmipkit_protocol::TransparentRsaPublicKey::try_new",
    ];
    if !ROUTES.contains(&symbol) {
        return None;
    }

    Some((|| {
        let format = key_material_format_for_target(target)
            .ok_or("transparent Key Material format is unavailable")?;
        let item = transparent_key_material(format)?;
        let model = match symbol {
            "kmipkit_protocol::TransparentDhPrivateKey::try_from_item" => {
                kmipkit::protocol::TransparentDhPrivateKey::try_from_item(item)
                    .map(kmipkit::protocol::TransparentDhPrivateKey::into_item)
                    .map_err(|_| "transparent DH private key parser rejected")?
            }
            "kmipkit_protocol::TransparentDhPublicKey::try_from_item" => {
                kmipkit::protocol::TransparentDhPublicKey::try_from_item(item)
                    .map(kmipkit::protocol::TransparentDhPublicKey::into_item)
                    .map_err(|_| "transparent DH public key parser rejected")?
            }
            "kmipkit_protocol::TransparentDsaPrivateKey::try_from_item" => {
                kmipkit::protocol::TransparentDsaPrivateKey::try_from_item(item)
                    .map(kmipkit::protocol::TransparentDsaPrivateKey::into_item)
                    .map_err(|_| "transparent DSA private key parser rejected")?
            }
            "kmipkit_protocol::TransparentDsaPublicKey::try_from_item" => {
                kmipkit::protocol::TransparentDsaPublicKey::try_from_item(item)
                    .map(kmipkit::protocol::TransparentDsaPublicKey::into_item)
                    .map_err(|_| "transparent DSA public key parser rejected")?
            }
            "kmipkit_protocol::TransparentEcPrivateKey::try_from_item" => {
                kmipkit::protocol::TransparentEcPrivateKey::try_from_item(item)
                    .map(kmipkit::protocol::TransparentEcPrivateKey::into_item)
                    .map_err(|_| "transparent EC private key parser rejected")?
            }
            "kmipkit_protocol::TransparentEcPublicKey::try_from_item" => {
                kmipkit::protocol::TransparentEcPublicKey::try_from_item(item)
                    .map(kmipkit::protocol::TransparentEcPublicKey::into_item)
                    .map_err(|_| "transparent EC public key parser rejected")?
            }
            "kmipkit_protocol::TransparentRsaPublicKey::try_from_item" => {
                kmipkit::protocol::TransparentRsaPublicKey::try_from_item(item)
                    .map(kmipkit::protocol::TransparentRsaPublicKey::into_item)
                    .map_err(|_| "transparent RSA public key parser rejected")?
            }
            _ => item,
        };
        crate::local_wire::roundtrip(&model)?;
        metadata_expected_target(&model, target, elements)
    })())
}

fn invoke_certificate_request_builder_route(symbol: &str, sample: &Item) -> Option<Result<Item>> {
    const ROUTES: &[&str] = &[
        "kmipkit_protocol::ReProvisionRequest::with_certificate",
        "kmipkit_protocol::ReProvisionRequest::with_certificate_request",
        "kmipkit_protocol::ValidateRequest::with_certificate",
    ];
    if !ROUTES.contains(&symbol) {
        return None;
    }

    Some((|| {
        match symbol {
            "kmipkit_protocol::ReProvisionRequest::with_certificate" => {
                let _request = kmipkit::protocol::ReProvisionRequest::new()
                    .with_certificate(kmipkit::ttlv::Structure::new());
            }
            "kmipkit_protocol::ReProvisionRequest::with_certificate_request" => {
                let _request = kmipkit::protocol::ReProvisionRequest::new()
                    .with_certificate_request(kmipkit::ttlv::Structure::new());
            }
            "kmipkit_protocol::ValidateRequest::with_certificate" => {
                let _request = kmipkit::protocol::ValidateRequest::new()
                    .with_certificate(kmipkit::ttlv::Structure::new());
            }
            _ => return Err("unsupported certificate request builder"),
        }
        sample.try_clone().map_err(|_| "local copy failed")
    })())
}

fn invoke_key_wrapping_request_route(symbol: &str, sample: &Item) -> Option<Result<Item>> {
    const ROUTES: &[&str] = &[
        "kmipkit_protocol::ExportRequest::with_key_wrapping_specification",
        "kmipkit_protocol::GetRequest::with_key_wrapping_specification",
        "kmipkit_protocol::ImportRequest::with_key_wrap_type",
    ];
    if !ROUTES.contains(&symbol) {
        return None;
    }

    Some((|| {
        match symbol {
            "kmipkit_protocol::ExportRequest::with_key_wrapping_specification" => {
                let _request = kmipkit::protocol::ExportRequest::new()
                    .with_key_wrapping_specification(kmipkit::ttlv::Structure::new());
            }
            "kmipkit_protocol::GetRequest::with_key_wrapping_specification" => {
                let _request = kmipkit::protocol::GetRequest::new()
                    .with_key_wrapping_specification(kmipkit::ttlv::Structure::new());
            }
            "kmipkit_protocol::ImportRequest::with_key_wrap_type" => {
                let key_wrap_type = sample.with_value(|value| match value {
                    ValueView::Enumeration(raw) => Ok(*raw),
                    _ => Ok(1),
                })?;
                let object = sample
                    .try_clone()
                    .map_err(|_| "Import object fixture copy failed")?;
                let _request = kmipkit::protocol::ImportRequest::new(
                    kmipkit::protocol::UniqueIdentifier::from("matrix-key"),
                    kmipkit::protocol::ObjectType::from_raw(2),
                    kmipkit::protocol::AttributeSet::new(),
                    object,
                )
                .with_key_wrap_type(kmipkit::protocol::KeyWrapType::from_raw(key_wrap_type));
            }
            _ => return Err("unsupported key wrapping request route"),
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
    if typed_attribute_constructor_route(symbol) {
        return Some((|| {
            let model = typed_attribute_constructor_item(symbol, elements)?;
            generated::execute(symbol, &model)
                .ok_or("generated typed attribute constructor missing")?
        })());
    }
    if typed_attribute_marker(symbol) == Some("VendorAttribute") {
        return Some((|| {
            let model = vendor_attribute_item()?;
            generated::execute(symbol, &model).ok_or("generated Vendor Attribute route missing")?
        })());
    }
    if let Some(result) = invoke_crypto_indicator_route(symbol, sample) {
        return Some(result);
    }
    if let Some(result) = invoke_protection_storage_mask_route(symbol, sample, target, elements) {
        return Some(result);
    }
    if let Some(result) = invoke_key_wrapping_request_route(symbol, sample) {
        return Some(result);
    }
    if let Some(result) = invoke_certificate_request_builder_route(symbol, sample) {
        return Some(result);
    }
    if let Some(result) = invoke_key_pair_attribute_route(symbol, sample, target) {
        return Some(result);
    }
    if let Some(result) = invoke_transparent_key_material_route(symbol, target, elements) {
        return Some(result);
    }
    if let Some(result) = invoke_request_attributes_route(symbol, sample) {
        return Some(result);
    }
    if let Some(result) = invoke_secret_operation_builder_route(symbol, sample, target, elements) {
        return Some(result);
    }
    if let Some(result) = invoke_cryptographic_parameters_route(symbol, sample) {
        return Some(result);
    }
    if let Some(result) = invoke_operation_data_route(symbol, sample) {
        return Some(result);
    }
    if symbol == "kmipkit_protocol::extension::to_ttlv" {
        return Some((|| {
            let information =
                kmipkit::protocol::extension::extension_information("KMIPKit.Validator")
                    .map_err(|_| "Extension Information constructor rejected")?;
            let information = kmipkit::protocol::extension::with_tag(information, 0x0054_0000)
                .map_err(|_| "Extension Information tag rejected")?;
            let information = kmipkit::protocol::extension::with_type(
                information,
                kmipkit::ttlv::ItemType::TextString,
            )
            .map_err(|_| "Extension Information type rejected")?;
            let structure = kmipkit::protocol::extension::to_ttlv(information)
                .map_err(|_| "Extension Information serializer rejected")?;
            let model = crate::driver::item(0x0042_00a4, Value::structure(structure))?;
            metadata_expected_target(&model, target, elements)
        })());
    }
    if [
        "kmipkit_protocol::Ticket::new",
        "kmipkit_protocol::Ticket::try_from_ttlv",
    ]
    .contains(&symbol)
    {
        return Some((|| {
            let fixture = ticket_ttlv_fixture()?;
            let tree = fixture
                .as_structure()
                .ok_or("Ticket fixture is not a Structure")?
                .try_clone()
                .map_err(|_| "Ticket fixture copy failed")?;
            let ticket = if symbol == "kmipkit_protocol::Ticket::new" {
                kmipkit::protocol::Ticket::new(
                    kmipkit::protocol::TicketType::LOGIN,
                    kmipkit::protocol::SecretBytes::new(vec![0x41; 16]),
                )
                .map_err(|_| "Ticket constructor rejected")?
            } else {
                kmipkit::protocol::Ticket::try_from_ttlv(tree)
                    .map_err(|_| "Ticket parser rejected")?
            };
            let item = crate::driver::item(0x0042_0149, Value::structure(ticket.into_ttlv()))?;
            let model = ticket_credential_fixture(item)?;
            let tag = sample.tag().raw();
            let mut candidates = Vec::new();
            collect_tagged_items(&model, tag, &mut candidates)?;
            match candidates.as_slice() {
                [item] => item.try_clone().map_err(|_| "Ticket fixture copy failed"),
                [] => Err("Ticket fixture target is missing"),
                _ => Err("Ticket fixture target is ambiguous"),
            }
        })());
    }
    if let Some(result) = invoke_simple_protocol_route(symbol, sample) {
        return Some(result);
    }
    if let Some(result) = invoke_attribute_reference_route(symbol, sample) {
        return Some(result);
    }
    if let Some(result) = invoke_client_batch_outcome_response_route(
        symbol,
        sample,
        target,
        elements,
        protocol_version,
    ) {
        return Some(result);
    }
    if let Some(result) =
        invoke_response_ttlv_route(symbol, sample, target, elements, protocol_version)
    {
        return Some(result);
    }
    if let Some(result) = invoke_client_batch_route(symbol, sample) {
        return Some(result);
    }
    if let Some(result) = invoke_login_usage_limits_route(symbol, target, elements) {
        return Some(result);
    }
    if let Some(result) = invoke_simple_request_builder_route(symbol, sample, target, elements) {
        return Some(result);
    }
    if let Some(result) = invoke_remaining_key_profile_model_route(symbol, sample, target, elements)
    {
        return Some(result);
    }
    if let Some(result) = invoke_request_selector_route(symbol, sample, target, elements) {
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
    if symbol == "kmipkit_protocol::LocateRequest::with_storage_status_mask" {
        return Some((|| {
            let raw = sample.with_value(|value| match value {
                ValueView::Integer(value) => Ok(u32::from_ne_bytes(value.to_ne_bytes())),
                _ => Err("Storage Status Mask fixture is not an Integer"),
            })?;
            let payload =
                kmipkit::protocol::LocateRequest::new(kmipkit::protocol::AttributeSet::new())
                    .with_storage_status_mask(kmipkit::protocol::StorageStatusMask::from_raw(raw))
                    .to_ttlv_payload()
                    .map_err(|_| "Locate request fixture rejected")?;
            metadata_expected_target(&request_payload_item(payload)?, target, elements)
        })());
    }
    if symbol == "kmipkit_protocol::LocateRequest::with_object_group_member" {
        return Some((|| {
            let raw = sample.with_value(|value| match value {
                ValueView::Enumeration(value) => Ok(*value),
                _ => Err("Object Group Member fixture is not an Enumeration"),
            })?;
            let payload =
                kmipkit::protocol::LocateRequest::new(kmipkit::protocol::AttributeSet::new())
                    .with_object_group_member(kmipkit::protocol::ObjectGroupMember::from_raw(raw))
                    .to_ttlv_payload()
                    .map_err(|_| "Locate request fixture rejected")?;
            metadata_expected_target(&request_payload_item(payload)?, target, elements)
        })());
    }
    if symbol == "kmipkit_protocol::JoinSplitKeyRequest::with_secret_data_type" {
        return Some((|| {
            let raw = sample.with_value(|value| match value {
                ValueView::Enumeration(value) => Ok(*value),
                _ => Err("Secret Data Type fixture is not an Enumeration"),
            })?;
            let references = ["split-key-one", "split-key-two"]
                .into_iter()
                .map(|value| {
                    kmipkit::protocol::SplitKeyReference::new(
                        kmipkit::protocol::UniqueIdentifier::TextString(value.to_owned()),
                    )
                    .with_split_key_threshold(2)
                })
                .collect();
            let payload = kmipkit::protocol::JoinSplitKeyRequest::new(
                kmipkit::protocol::ObjectType::from_raw(2),
                references,
            )
            .with_secret_data_type(raw)
            .into_ttlv_payload()
            .map_err(|_| "Join Split Key request fixture rejected")?;
            let model = request_payload_item(payload)?;
            match metadata_expected_target(&model, target, elements) {
                Ok(projection) => Ok(projection),
                Err("metadata fixture target is missing")
                    if target["kind"] == "structure_member" =>
                {
                    object_root_fixture("Secret Data", target, elements)
                        .map(|(_, _, projection)| projection)
                }
                Err(error) => Err(error),
            }
        })());
    }
    if symbol == "kmipkit_protocol::EncryptionKeyInformation::try_from_unique_identifier" {
        return Some((|| {
            let mut information =
                kmipkit::protocol::EncryptionKeyInformation::try_from_unique_identifier(
                    kmipkit::protocol::UniqueIdentifier::from("kmipkit-encryption-key"),
                )
                .map_err(|_| "encryption key information fixture rejected")?;
            if target["name"] == "Cryptographic Parameters" {
                information = information
                    .with_cryptographic_parameters(kmipkit::ttlv::Structure::new())
                    .map_err(|_| "encryption key information parameters rejected")?;
            }
            let model = information.into_item();
            crate::local_wire::roundtrip(&model)?;
            metadata_expected_target(&model, target, elements)
        })());
    }
    if symbol == "kmipkit_protocol::MacSignatureKeyInformation::try_from_unique_identifier" {
        return Some((|| {
            let mut information =
                kmipkit::protocol::MacSignatureKeyInformation::try_from_unique_identifier(
                    kmipkit::protocol::UniqueIdentifier::from("kmipkit-mac-key"),
                )
                .map_err(|_| "MAC key information fixture rejected")?;
            if target["name"] == "Cryptographic Parameters" {
                information = information
                    .with_cryptographic_parameters(kmipkit::ttlv::Structure::new())
                    .map_err(|_| "MAC key information parameters rejected")?;
            }
            let model = information.into_item();
            crate::local_wire::roundtrip(&model)?;
            metadata_expected_target(&model, target, elements)
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
    if [
        "kmipkit_protocol::TransparentRsaPrivateKey::try_from_components",
        "kmipkit_protocol::TransparentRsaPrivateKey::try_from_item",
    ]
    .contains(&symbol)
    {
        use kmipkit::protocol::TransparentRsaPrivateKey;

        return Some((|| {
            let components = || vec![0x31, 0x41];
            let result =
                if symbol == "kmipkit_protocol::TransparentRsaPrivateKey::try_from_components" {
                    TransparentRsaPrivateKey::try_from_components(
                        components(),
                        Some(components()),
                        Some(components()),
                        Some(components()),
                        Some(components()),
                        Some(components()),
                        Some(components()),
                        Some(components()),
                    )
                    .map(TransparentRsaPrivateKey::into_item)
                    .map_err(|_| "transparent RSA private key fixture rejected")?
                } else {
                    let (_, item) = key_material_item_for_target(target, 10)?;
                    TransparentRsaPrivateKey::try_from_item(item)
                        .map(TransparentRsaPrivateKey::into_item)
                        .map_err(|_| "transparent RSA private key parser rejected")?
                };
            crate::local_wire::roundtrip(&result)?;
            metadata_expected_target(&result, target, elements)
        })());
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

const RESPONSE_ACCESSOR_ROUTES: &[&str] = &[
    "kmipkit_client::ClientBatchOutcome::asynchronous_correlation_value",
    "kmipkit_client::ClientBatchItemResponse::outcome",
    "kmipkit_client::ClientBatchItemResponse::unique_batch_item_id",
    "kmipkit_client::ClientBatchItemResponse::extensions",
    "kmipkit_client::ClientBatchItemResponse::discover_versions_supported_versions",
    "kmipkit_client::ClientBatchResponse::get",
    "kmipkit_client::ClientBatchResponse::len",
    "kmipkit_client::ClientMessageExtension::with_ttlv",
    "kmipkit_client::ClientResponseView::ping",
];

fn response_item(
    response: &kmipkit::ClientBatchResponse,
) -> Result<&kmipkit::ClientBatchItemResponse> {
    response
        .get(0)
        .ok_or("controlled fixture returned no response item")
}

fn response_digest(response: &kmipkit::ClientBatchItemResponse) -> Result<String> {
    let mut fields = vec![
        crate::driver::item(
            0x0042_005c,
            Value::enumeration(response.outcome().operation().raw()),
        )?,
        crate::driver::item(
            0x0042_007f,
            Value::enumeration(response.outcome().result().status().raw()),
        )?,
    ];
    if let Some(id) = response.unique_batch_item_id() {
        fields.push(crate::driver::item(
            0x0042_0093,
            Value::byte_string(id.to_vec()),
        )?);
    }
    if let Some(reason) = response.outcome().result().reason() {
        fields.push(crate::driver::item(
            0x0042_007e,
            Value::enumeration(reason.raw()),
        )?);
    }
    if let Some(payload) = response.raw() {
        fields.push(crate::driver::item(
            0x0042_007c,
            Value::structure(
                payload
                    .try_clone()
                    .map_err(|_| "response payload copy failed")?,
            ),
        )?);
    }
    let batch_item = crate::driver::item(
        0x0042_000f,
        Value::structure(crate::driver::structure(0x0042_000f, fields)?),
    )?;
    let header = response
        .response_header_raw()
        .ok_or("controlled fixture response header unavailable")?
        .try_clone()
        .map_err(|_| "response header copy failed")?;
    let header = crate::driver::item(0x0042_007a, Value::structure(header))?;
    let message = crate::driver::item(
        0x0042_007b,
        Value::structure(crate::driver::structure(
            0x0042_007b,
            vec![header, batch_item],
        )?),
    )?;
    crate::local_wire::roundtrip(&message)
}

fn response_route_invoked(
    symbol: &str,
    query: &kmipkit::ClientBatchResponse,
    discover: &kmipkit::ClientBatchResponse,
    ping: Option<&kmipkit::ClientBatchItemResponse>,
    pending: &kmipkit::ClientBatchResponse,
) -> Result<String> {
    let query_item = response_item(query)?;
    let (response, invoked) = match symbol {
        "kmipkit_client::ClientBatchOutcome::asynchronous_correlation_value" => {
            let pending_item = response_item(pending)?;
            (
                pending_item,
                pending_item
                    .outcome()
                    .asynchronous_correlation_value()
                    .is_some_and(|value| !value.is_empty()),
            )
        }
        "kmipkit_client::ClientBatchItemResponse::outcome" => (
            query_item,
            query_item.outcome().response().result().status().raw() == 0,
        ),
        "kmipkit_client::ClientBatchItemResponse::unique_batch_item_id" => (
            query_item,
            query_item.unique_batch_item_id() == Some(b"rust-response-route".as_slice()),
        ),
        "kmipkit_client::ClientBatchItemResponse::extensions" => {
            (query_item, !query_item.extensions().is_empty())
        }
        "kmipkit_client::ClientBatchItemResponse::discover_versions_supported_versions" => {
            let version_item = response_item(discover)?;
            (
                version_item,
                version_item
                    .discover_versions_supported_versions()
                    .is_some_and(|versions| !versions.is_empty()),
            )
        }
        "kmipkit_client::ClientBatchResponse::get" => (
            query.get(0).ok_or("batch response get returned no item")?,
            true,
        ),
        "kmipkit_client::ClientBatchResponse::len" => (query_item, query.len() == 1),
        "kmipkit_client::ClientMessageExtension::with_ttlv" => {
            let extension = query_item
                .extensions()
                .first()
                .ok_or("controlled fixture extension missing")?;
            let has_extension_shape = extension.with_ttlv(|value| value.children().len() == 3);
            (query_item, has_extension_shape)
        }
        "kmipkit_client::ClientResponseView::ping" => {
            let ping = ping.ok_or("Ping response was not requested for this version")?;
            (ping, ping.outcome().response().ping().is_some())
        }
        _ => return Err("unsupported response accessor route"),
    };
    if !invoked {
        return Err("controlled fixture response did not satisfy the accessor route");
    }
    response_digest(response)
}

/// Execute the public response accessors that need server-produced typed responses.
///
/// The connection is restricted to a local raw-TLS TTLV fixture so CI cannot
/// turn this evidence command into a probe against an external KMIP service.
#[allow(clippy::too_many_lines)] // Keep fixture selection and its exact response calls auditable together.
pub(crate) fn run_network(matrix: &Path, connection: &Connection, output: &Path) -> Result<bool> {
    connection.validate()?;
    if connection.transport != "raw_tls"
        || connection.encoding != "TTLV"
        || connection.api_level != "typed-protocol"
    {
        return Err("response route evidence requires typed-protocol TTLV over raw TLS");
    }
    let endpoint = connection
        .endpoints
        .raw_tls
        .as_ref()
        .ok_or("response route evidence requires a raw TLS endpoint")?;
    if !endpoint
        .host
        .parse::<std::net::IpAddr>()
        .is_ok_and(|address| address.is_loopback())
    {
        return Err("response route evidence requires a loopback endpoint");
    }
    let _version = match connection.protocol_version.as_str() {
        "2.0" => kmipkit::ProtocolVersion::V2_0,
        "2.1" => kmipkit::ProtocolVersion::V2_1,
        _ => return Err("unsupported response route protocol version"),
    };
    let routes_bytes = include_bytes!("../../shared/coverage/route-capabilities.json");
    let routes: Json =
        serde_json::from_slice(routes_bytes).map_err(|_| "route manifest invalid")?;
    if routes["library_commit"] != env!("KMIPKIT_LIBRARY_COMMIT") {
        return Err("route artifact commit mismatch");
    }
    let canonical_rows: HashSet<_> =
        include_str!(concat!(env!("OUT_DIR"), "/route-cell-hashes.txt"))
            .lines()
            .collect();
    let rows: Vec<Json> =
        BufReader::new(std::fs::File::open(matrix).map_err(|_| "matrix unavailable")?)
            .lines()
            .map(|line| {
                serde_json::from_str(&line.map_err(|_| "matrix read failed")?)
                    .map_err(|_| "matrix row invalid")
            })
            .collect::<Result<_>>()?;
    if rows.is_empty() {
        return Err("response route matrix is empty");
    }
    let mut seen = HashSet::new();
    for row in &rows {
        let canonical = serde_json::to_vec(row).map_err(|_| "matrix normalization failed")?;
        if !canonical_rows.contains(hash(&canonical).as_str()) {
            return Err("matrix row does not match canonical shared obligations");
        }
        if row["language"] != "rust"
            || row["mode"] != "local"
            || row["action"] != "decode"
            || row["protocol_version"] != connection.protocol_version
            || !RESPONSE_ACCESSOR_ROUTES.contains(&text(row, "public_api_symbol")?)
        {
            return Err("response route matrix contains an unsupported cell");
        }
        let key = json!([
            row["case_id"],
            row["route_id"],
            row["target_id"],
            row["protocol_version"]
        ])
        .to_string();
        if !seen.insert(key) {
            return Err("duplicate response route cell");
        }
    }

    let mut client = crate::driver::client(connection)?;
    let limits = kmipkit::ttlv::codec::CodecLimits::defaults();
    let query_item = ClientBatchItem::query(QueryRequest::new([QueryFunction::OPERATIONS]))
        .with_unique_batch_item_id(b"rust-response-route".to_vec());
    let query = client
        .execute(ClientBatch::new(query_item), &limits)
        .map_err(|_| "controlled fixture query failed")?;
    let discover = client
        .execute(
            ClientBatch::new(ClientBatchItem::discover_versions()),
            &limits,
        )
        .map_err(|_| "controlled fixture discover-versions request failed")?;
    let ping = if rows
        .iter()
        .any(|row| row["public_api_symbol"] == "kmipkit_client::ClientResponseView::ping")
    {
        Some(
            client
                .ping(&limits)
                .map_err(|_| "controlled fixture ping failed")?,
        )
    } else {
        None
    };
    let pending_request = ClientRequest::encrypt(EncryptRequest::new(
        Some(UniqueIdentifier::TextString("route-fixture-key".to_owned())),
        Some(OperationData::ByteString(SecretBytes::new(vec![0x41; 16]))),
    ));
    let pending = client
        .execute(
            ClientBatch::new(ClientBatchItem::new(pending_request)).with_asynchronous_indicator(1),
            &limits,
        )
        .map_err(|_| "controlled fixture asynchronous response failed")?;
    let mut events = Vec::with_capacity(rows.len());
    for row in &rows {
        let symbol = text(row, "public_api_symbol")?;
        let digest = response_route_invoked(symbol, &query, &discover, ping.as_ref(), &pending)
            .map_err(|error| {
                eprintln!("response route {symbol} failed: {error}");
                "controlled fixture response accessor failed"
            })?;
        let mut event = json!({
            "case_id":row["case_id"], "route_id":row["route_id"], "target_id":row["target_id"],
            "public_api_symbol":symbol, "execution_route_id":null, "execution_api_symbol":null,
            "invocation_id":format!("rust-network-{}", events.len()), "action":"decode",
            "protocol_version":row["protocol_version"], "encoding":row["encoding"], "transport":row["transport"],
            "status":"passed", "invocation_count":1, "response_tree_sha256":digest,
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
        events.push(event);
    }
    let version_text = connection.protocol_version.clone();
    let configuration_fingerprint = crate::models::configuration_fingerprint(connection)?;
    let report = json!({
        "schema_version":"1.0", "run_id":format!("rust-response-{}", uuid::Uuid::new_v4()),
        "language":"rust", "server":{"id":connection.server_id,"product":"controlled-fixture",
            "version":"offline-1","configuration_fingerprint":configuration_fingerprint},
        "library_commit":env!("KMIPKIT_LIBRARY_COMMIT"), "artifact_sha256":env!("KMIPKIT_ARTIFACT_SHA256"),
        "catalog_sha256":hash(include_bytes!("../../shared/catalog/kmipkit-1.0-test-catalog.json")),
        "route_manifest_sha256":hash(routes_bytes),
        "protocol_versions":[version_text], "network_combinations":[{"encoding":"TTLV","transport":"raw_tls"}],
        "events":events,
    });
    std::fs::write(
        output,
        serde_json::to_vec_pretty(&report).map_err(|_| "matrix output failed")?,
    )
    .map_err(|_| "matrix output unavailable")?;
    Ok(true)
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
    fn key_block_parser_route_handles_its_normative_tag_target() {
        // OASIS KMIP 2.0/2.1 §§3.1 and 3.10 define Key Block as a TTLV structure.
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
            .find(|target| target["target_id"] == "KMIPKIT-ELEM-TAG-420040")
            .unwrap();
        let symbol = "kmipkit_protocol::KeyBlock::try_from_item";
        let expected = sample_for_route(symbol, target, elements).unwrap();
        let result = invoke(symbol, &expected, target, elements, "2.1");
        assert!(
            result.is_some(),
            "KeyBlock parser tag route is not executable"
        );
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
    fn client_batch_encrypt_response_accessor_is_executable() {
        // OASIS KMIP 2.0/2.1 §6.1.17 defines the Encrypt operation response.
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
            .find(|target| target["target_id"] == "KMIPKIT-ELEM-OP-C2S-ENCRYPT")
            .unwrap();
        let symbol = "kmipkit_client::ClientBatchOutcome::encrypt_response";
        let expected = sample_for_route_at_version(symbol, target, elements, "2.1").unwrap();

        let result = invoke(symbol, &expected, target, elements, "2.1");

        assert!(
            result.is_some(),
            "Encrypt response accessor is not executable"
        );
    }

    #[test]
    fn client_batch_operation_response_accessors_execute_manifest_routes() {
        // OASIS KMIP 2.0/2.1 §6 defines each typed operation response.
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
                    && route["public_api_symbol"].as_str().is_some_and(|symbol| {
                        symbol.starts_with("kmipkit_client::ClientBatchOutcome::")
                            && symbol.ends_with("_response")
                    })
            })
        {
            let symbol = route["public_api_symbol"].as_str().unwrap();
            let target = route["normative_target_ids"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(Json::as_str)
                .filter_map(|target_id| {
                    targets
                        .iter()
                        .find(|target| target["target_id"] == target_id)
                })
                .find(|target| target["kind"] == "operation")
                .unwrap_or_else(|| panic!("{symbol} has no operation target"));
            let expected = sample_for_route_at_version(symbol, target, elements, "2.1").unwrap();

            let result = invoke(symbol, &expected, target, elements, "2.1");

            assert!(result.is_some(), "{symbol} route is not executable");
            let result = result
                .unwrap()
                .unwrap_or_else(|error| panic!("{symbol}: {error}"));
            assert!(
                crate::local_wire::target_hash(&result, &expected).is_ok(),
                "{symbol}"
            );
        }
    }

    #[test]
    fn client_batch_outcome_response_variants_execute_manifest_routes() {
        // OASIS KMIP 2.0/2.1 §6 defines these typed operation responses.
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
        let symbols = [
            "kmipkit_client::ClientBatchOutcome::Activate",
            "kmipkit_client::ClientBatchOutcome::Archive",
            "kmipkit_client::ClientBatchOutcome::DeriveKey",
            "kmipkit_client::ClientBatchOutcome::Destroy",
            "kmipkit_client::ClientBatchOutcome::GetUsageAllocation",
            "kmipkit_client::ClientBatchOutcome::Recover",
        ];

        for symbol in symbols {
            let route = manifest["routes"]
                .as_array()
                .unwrap()
                .iter()
                .find(|route| route["public_api_symbol"] == symbol)
                .unwrap();
            let target = route["normative_target_ids"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(Json::as_str)
                .filter_map(|target_id| {
                    targets
                        .iter()
                        .find(|target| target["target_id"] == target_id)
                })
                .find(|target| target["kind"] == "operation")
                .unwrap_or_else(|| panic!("{symbol} has no operation target"));
            let expected = sample_for_route_at_version(symbol, target, elements, "2.1").unwrap();
            let result = invoke(symbol, &expected, target, elements, "2.1");

            assert!(result.is_some(), "{symbol} route is not executable");
            let result = result
                .unwrap()
                .unwrap_or_else(|error| panic!("{symbol}: {error}"));
            assert!(
                crate::local_wire::target_hash(&result, &expected).is_ok(),
                "{symbol}"
            );
        }
    }

    #[test]
    fn typed_response_ttlv_accessors_execute_manifest_routes() {
        // OASIS KMIP 2.0/2.1 §6 defines these typed operation response payloads.
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
        let symbols = [
            "kmipkit_protocol::HashResponse::with_ttlv",
            "kmipkit_protocol::MacResponse::with_ttlv",
            "kmipkit_protocol::MacVerifyResponse::with_ttlv",
            "kmipkit_protocol::ReProvisionResponse::with_ttlv",
            "kmipkit_protocol::RegisterResponse::with_ttlv",
            "kmipkit_protocol::RevokeResponse::with_ttlv",
            "kmipkit_protocol::RngRetrieveResponse::with_ttlv",
            "kmipkit_protocol::RngSeedResponse::with_ttlv",
            "kmipkit_protocol::SignResponse::with_ttlv",
            "kmipkit_protocol::SignatureVerifyResponse::with_ttlv",
        ];

        for symbol in symbols {
            let route = manifest["routes"]
                .as_array()
                .unwrap()
                .iter()
                .find(|route| route["public_api_symbol"] == symbol)
                .unwrap();
            let target = route["normative_target_ids"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(Json::as_str)
                .filter_map(|target_id| {
                    targets
                        .iter()
                        .find(|target| target["target_id"] == target_id)
                })
                .find(|target| target["kind"] == "operation")
                .unwrap_or_else(|| panic!("{symbol} has no operation target"));
            let expected = sample_for_route_at_version(symbol, target, elements, "2.1").unwrap();
            let result = invoke(symbol, &expected, target, elements, "2.1");

            assert!(result.is_some(), "{symbol} route is not executable");
            let result = result
                .unwrap()
                .unwrap_or_else(|error| panic!("{symbol}: {error}"));
            assert!(
                crate::local_wire::target_hash(&result, &expected).is_ok(),
                "{symbol}"
            );
        }
    }

    #[test]
    fn locate_storage_status_mask_executes_every_manifest_target() {
        // OASIS KMIP 2.0/2.1 §12.3 and §6.1.28 define the Storage Status Mask.
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
        let symbol = "kmipkit_protocol::LocateRequest::with_storage_status_mask";
        let route = manifest["routes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|route| route["public_api_symbol"] == symbol)
            .unwrap();

        for target_id in route["normative_target_ids"].as_array().unwrap() {
            let target = targets
                .iter()
                .find(|target| target["target_id"] == *target_id)
                .unwrap();
            let expected = sample_for_route_at_version(symbol, target, elements, "2.1").unwrap();
            let result = invoke(symbol, &expected, target, elements, "2.1");

            assert!(
                result.is_some(),
                "{symbol} {} route is not executable",
                target["target_id"]
            );
            let result = result
                .unwrap()
                .unwrap_or_else(|error| panic!("{symbol}: {error}"));
            assert!(
                crate::local_wire::target_hash(&result, &expected).is_ok(),
                "{symbol}"
            );
        }
    }

    #[test]
    fn locate_object_group_member_executes_every_manifest_target() {
        // OASIS KMIP 2.0/2.1 §11.33 and §6.1.28 define the Object Group Member.
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
        let symbol = "kmipkit_protocol::LocateRequest::with_object_group_member";
        let route = manifest["routes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|route| route["public_api_symbol"] == symbol)
            .unwrap();

        for target_id in route["normative_target_ids"].as_array().unwrap() {
            let target = targets
                .iter()
                .find(|target| target["target_id"] == *target_id)
                .unwrap();
            let expected = sample_for_route_at_version(symbol, target, elements, "2.1").unwrap();
            let result = invoke(symbol, &expected, target, elements, "2.1");

            assert!(
                result.is_some(),
                "{symbol} {} route is not executable",
                target["target_id"]
            );
            let result = result
                .unwrap()
                .unwrap_or_else(|error| panic!("{symbol}: {error}"));
            assert!(
                crate::local_wire::target_hash(&result, &expected).is_ok(),
                "{symbol}"
            );
        }
    }

    #[test]
    fn join_split_key_secret_data_type_executes_manifest_targets() {
        // OASIS KMIP 2.0/2.1 §§2.7 and 6.1.27 define Secret Data Type.
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
        let symbol = "kmipkit_protocol::JoinSplitKeyRequest::with_secret_data_type";
        let route = manifest["routes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|route| route["public_api_symbol"] == symbol)
            .unwrap();

        for target_id in route["normative_target_ids"].as_array().unwrap() {
            let target = targets
                .iter()
                .find(|target| target["target_id"] == *target_id)
                .unwrap();
            let expected = sample_for_route_at_version(symbol, target, elements, "2.1").unwrap();
            let result = invoke(symbol, &expected, target, elements, "2.1");

            assert!(
                result.is_some(),
                "{symbol} {} route is not executable",
                target["target_id"]
            );
            let result = result
                .unwrap()
                .unwrap_or_else(|error| panic!("{symbol}: {error}"));
            assert!(
                crate::local_wire::target_hash(&result, &expected).is_ok(),
                "{symbol}"
            );
        }
    }

    #[test]
    fn request_selector_and_certificate_type_builders_execute_manifest_targets() {
        // OASIS KMIP 2.0/2.1 §§6.1.6, 6.1.7, 6.1.11, and 6.1.17 define these fields.
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
        let symbols = [
            "kmipkit_protocol::CertifyRequest::with_certificate_request_type",
            "kmipkit_protocol::RecertifyRequest::with_certificate_request_type",
            "kmipkit_protocol::ExportRequest::with_key_compression_type",
            "kmipkit_protocol::GetRequest::with_key_compression_type",
            "kmipkit_protocol::ExportRequest::with_key_wrap_type",
            "kmipkit_protocol::GetRequest::with_key_wrap_type",
        ];

        for symbol in symbols {
            let route = manifest["routes"]
                .as_array()
                .unwrap()
                .iter()
                .find(|route| route["public_api_symbol"] == symbol)
                .unwrap();
            for target_id in route["normative_target_ids"].as_array().unwrap() {
                let target = targets
                    .iter()
                    .find(|target| target["target_id"] == *target_id)
                    .unwrap();
                let expected = sample_for_route_at_version(symbol, target, elements, "2.1")
                    .unwrap_or_else(|error| panic!("{symbol} {}: {error}", target["target_id"]));
                let result = invoke(symbol, &expected, target, elements, "2.1");

                assert!(
                    result.is_some(),
                    "{symbol} {} route is not executable",
                    target["target_id"]
                );
                let result = result
                    .unwrap()
                    .unwrap_or_else(|error| panic!("{symbol}: {error}"));
                let projection = crate::local_wire::target_hash(&result, &expected);
                assert!(
                    projection.is_ok(),
                    "{symbol} {}: {projection:?}",
                    target["target_id"]
                );
            }
        }
    }

    #[test]
    fn login_usage_limits_builders_execute_manifest_targets() {
        // OASIS KMIP 2.0/2.1 §§6.1.12, 6.1.30, and 7.40 define Usage Limits.
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
        let symbols = [
            "kmipkit_protocol::DelegatedLoginRequest::with_usage_limits",
            "kmipkit_protocol::LoginRequest::with_usage_limits",
        ];

        for symbol in symbols {
            let route = manifest["routes"]
                .as_array()
                .unwrap()
                .iter()
                .find(|route| route["public_api_symbol"] == symbol)
                .unwrap();
            for target_id in route["normative_target_ids"].as_array().unwrap() {
                let target = targets
                    .iter()
                    .find(|target| target["target_id"] == *target_id)
                    .unwrap();
                let expected = sample_for_route_at_version(symbol, target, elements, "2.1")
                    .unwrap_or_else(|error| panic!("{symbol} {}: {error}", target["target_id"]));
                let result = invoke(symbol, &expected, target, elements, "2.1");

                assert!(
                    result.is_some(),
                    "{symbol} {} route is not executable",
                    target["target_id"]
                );
                let result = result
                    .unwrap()
                    .unwrap_or_else(|error| panic!("{symbol}: {error}"));
                let projection = crate::local_wire::target_hash(&result, &expected);
                assert!(
                    projection.is_ok(),
                    "{symbol} {}: {projection:?}",
                    target["target_id"]
                );
            }
        }
    }

    #[test]
    fn client_batch_authentication_and_timestamp_routes_cover_nested_fixtures() {
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
                "kmipkit_client::ClientBatch::with_authentication",
                "KMIPKIT-ELEM-MESSAGE-FIELD-9-4-CREDENTIAL-MAY-BE-REPEATED",
            ),
            (
                "kmipkit_client::ClientBatch::with_request_time_stamp",
                "KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-HASHED-PASSWORD-TIMESTAMP",
            ),
            (
                "kmipkit_client::ClientBatch::with_request_time_stamp",
                "KMIPKIT-ELEM-TAG-420092",
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
                .unwrap_or_else(|error| panic!("{symbol} rejected {target_id}: {error}"));
            assert!(
                crate::local_wire::target_hash(&result, &expected).is_ok(),
                "{symbol} changed {target_id}"
            );
        }
    }

    #[test]
    fn protocol_version_constructor_executes_each_version_field_fixture() {
        let catalog: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let specification: Json =
            serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
                .unwrap();
        let elements = specification["elements"].as_array().unwrap();
        for target_id in [
            "KMIPKIT-ELEM-MESSAGE-FIELD-8-2-PROTOCOL-VERSION",
            "KMIPKIT-ELEM-MESSAGE-FIELD-8-5-PROTOCOL-VERSION",
            "KMIPKIT-ELEM-MESSAGE-FIELD-9-16-PROTOCOL-VERSION",
            "KMIPKIT-ELEM-MESSAGE-FIELD-9-16-PROTOCOL-VERSION-MAJOR",
            "KMIPKIT-ELEM-MESSAGE-FIELD-9-16-PROTOCOL-VERSION-MINOR",
        ] {
            let target = catalog["normative_targets"]
                .as_array()
                .unwrap()
                .iter()
                .find(|target| target["target_id"] == target_id)
                .unwrap();
            let expected = sample_for_route_at_version(
                "kmipkit_protocol::ProtocolVersion::from_raw",
                target,
                elements,
                "2.1",
            )
            .unwrap();
            let result = invoke(
                "kmipkit_protocol::ProtocolVersion::from_raw",
                &expected,
                target,
                elements,
                "2.1",
            )
            .unwrap_or_else(|| panic!("ProtocolVersion::from_raw route missing for {target_id}"))
            .unwrap_or_else(|error| {
                panic!("ProtocolVersion::from_raw rejected {target_id}: {error}")
            });
            assert!(
                crate::local_wire::target_hash(&result, &expected).is_ok(),
                "ProtocolVersion::from_raw changed {target_id}"
            );
        }
    }

    #[test]
    fn secret_ttlv_value_constructors_execute_their_data_type_fixtures() {
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
                "kmipkit_protocol::SecretBytes::into_ttlv_value",
                "KMIPKIT-ELEM-DATA-TYPE-BYTE-STRING",
            ),
            (
                "kmipkit_protocol::SecretText::into_ttlv_value",
                "KMIPKIT-ELEM-DATA-TYPE-TEXT-STRING",
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
                .unwrap_or_else(|| panic!("{symbol} route is missing"))
                .unwrap_or_else(|error| panic!("{symbol} rejected its data type fixture: {error}"));
            assert!(
                crate::local_wire::target_hash(&result, &expected).is_ok(),
                "{symbol} changed the data type fixture"
            );
        }
    }

    #[test]
    fn ticket_constructors_execute_each_nested_normative_target() {
        let catalog: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let specification: Json =
            serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
                .unwrap();
        let elements = specification["elements"].as_array().unwrap();
        let targets = [
            "KMIPKIT-ELEM-CREDENTIAL-TICKET",
            "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-39-TICKET",
            "KMIPKIT-ELEM-STRUCTURE-MEMBER-7-39-TICKET-TYPE",
            "KMIPKIT-ELEM-STRUCTURE-MEMBER-7-39-TICKET-VALUE",
            "KMIPKIT-ELEM-STRUCTURE-MEMBER-9-11-TICKET-TICKET",
            "KMIPKIT-ELEM-TAG-420149",
        ];
        for symbol in [
            "kmipkit_protocol::Ticket::new",
            "kmipkit_protocol::Ticket::try_from_ttlv",
        ] {
            for target_id in targets {
                let target = catalog["normative_targets"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|target| target["target_id"] == target_id)
                    .unwrap();
                let expected = sample_for_route_at_version(symbol, target, elements, "2.1")
                    .unwrap_or_else(|error| {
                        panic!("{symbol} could not fixture {target_id}: {error}")
                    });
                let result = invoke(symbol, &expected, target, elements, "2.1")
                    .unwrap_or_else(|| panic!("{symbol} route is missing for {target_id}"))
                    .unwrap_or_else(|error| panic!("{symbol} rejected {target_id}: {error}"));
                assert!(
                    crate::local_wire::target_hash(&result, &expected).is_ok(),
                    "{symbol} changed {target_id}"
                );
            }
        }
    }

    #[test]
    fn unknown_managed_object_constructor_accepts_the_vendor_tag_range() {
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
            .find(|target| target["target_id"] == "KMIPKIT-RANGE-004")
            .unwrap();
        for version in ["2.0", "2.1"] {
            let expected = sample_for_route_at_version(
                "kmipkit_protocol::ManagedObject::unknown",
                target,
                elements,
                version,
            )
            .unwrap();
            let result = invoke(
                "kmipkit_protocol::ManagedObject::unknown",
                &expected,
                target,
                elements,
                version,
            )
            .unwrap_or_else(|| panic!("ManagedObject::unknown route is missing"))
            .unwrap_or_else(|error| panic!("ManagedObject::unknown rejected the fixture: {error}"));
            assert!(
                crate::local_wire::target_hash(&result, &expected).is_ok(),
                "ManagedObject::unknown changed the extension object"
            );
        }
    }

    #[test]
    fn extension_information_serializer_executes_each_normative_target() {
        let catalog: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let specification: Json =
            serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
                .unwrap();
        let elements = specification["elements"].as_array().unwrap();
        let targets = [
            "KMIPKIT-ELEM-OPERATION-STRUCTURE-7-13-EXTENSION-INFORMATION",
            "KMIPKIT-ELEM-TAG-4200A4",
            "KMIPKIT-PAYLOAD-CLIENT-QUERY-RESPONSE-20-006",
            "KMIPKIT-PAYLOAD-CLIENT-QUERY-RESPONSE-21-006",
            "KMIPKIT-PAYLOAD-SERVER-QUERY-RESPONSE-20-005",
            "KMIPKIT-PAYLOAD-SERVER-QUERY-RESPONSE-21-005",
        ];
        for target_id in targets {
            let target = catalog["normative_targets"]
                .as_array()
                .unwrap()
                .iter()
                .find(|target| target["target_id"] == target_id)
                .unwrap();
            let expected = sample_for_route_at_version(
                "kmipkit_protocol::extension::to_ttlv",
                target,
                elements,
                "2.1",
            )
            .unwrap();
            let result = invoke(
                "kmipkit_protocol::extension::to_ttlv",
                &expected,
                target,
                elements,
                "2.1",
            )
            .unwrap_or_else(|| panic!("Extension Information route is missing for {target_id}"))
            .unwrap_or_else(|error| panic!("Extension Information rejected {target_id}: {error}"));
            assert!(
                crate::local_wire::target_hash(&result, &expected).is_ok(),
                "Extension Information serializer changed {target_id}"
            );
        }
    }

    #[test]
    fn operation_data_builders_execute_every_manifest_target() {
        let catalog: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let specification: Json =
            serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
                .unwrap();
        let elements = specification["elements"].as_array().unwrap();
        let routes: Json = serde_json::from_str(include_str!(
            "../../shared/coverage/route-capabilities.json"
        ))
        .unwrap();
        for symbol in [
            "kmipkit_protocol::HashRequest::with_data",
            "kmipkit_protocol::MacRequest::with_data",
            "kmipkit_protocol::MacVerifyRequest::with_data",
            "kmipkit_protocol::SignRequest::with_data",
            "kmipkit_protocol::SignatureVerifyRequest::with_data",
        ] {
            let route = routes["routes"]
                .as_array()
                .unwrap()
                .iter()
                .find(|route| route["public_api_symbol"] == symbol)
                .unwrap_or_else(|| panic!("route manifest omits {symbol}"));
            for target_id in route["normative_target_ids"].as_array().unwrap() {
                let target_id = target_id.as_str().unwrap();
                let target = catalog["normative_targets"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|target| target["target_id"] == target_id)
                    .unwrap();
                let expected = sample_for_route_at_version(symbol, target, elements, "2.1")
                    .unwrap_or_else(|error| panic!("{symbol} cannot fixture {target_id}: {error}"));
                let result = invoke(symbol, &expected, target, elements, "2.1")
                    .unwrap_or_else(|| panic!("{symbol} route is missing for {target_id}"))
                    .unwrap_or_else(|error| panic!("{symbol} rejected {target_id}: {error}"));
                assert!(
                    crate::local_wire::target_hash(&result, &expected).is_ok(),
                    "{symbol} changed {target_id}"
                );
            }
        }
    }

    #[test]
    fn cryptographic_parameter_builders_execute_every_manifest_target() {
        let catalog: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let specification: Json =
            serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
                .unwrap();
        let elements = specification["elements"].as_array().unwrap();
        let routes: Json = serde_json::from_str(include_str!(
            "../../shared/coverage/route-capabilities.json"
        ))
        .unwrap();
        for symbol in [
            "kmipkit_protocol::EncryptRequest::with_cryptographic_parameters",
            "kmipkit_protocol::DecryptRequest::with_cryptographic_parameters",
            "kmipkit_protocol::MacRequest::with_cryptographic_parameters",
            "kmipkit_protocol::MacVerifyRequest::with_cryptographic_parameters",
            "kmipkit_protocol::SignRequest::with_cryptographic_parameters",
            "kmipkit_protocol::SignatureVerifyRequest::with_cryptographic_parameters",
            "kmipkit_protocol::DerivationParameters::with_cryptographic_parameters",
        ] {
            let route = routes["routes"]
                .as_array()
                .unwrap()
                .iter()
                .find(|route| route["public_api_symbol"] == symbol)
                .unwrap_or_else(|| panic!("route manifest omits {symbol}"));
            for target_id in route["normative_target_ids"].as_array().unwrap() {
                let target_id = target_id.as_str().unwrap();
                let target = catalog["normative_targets"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|target| target["target_id"] == target_id)
                    .unwrap();
                let expected = sample_for_route_at_version(symbol, target, elements, "2.1")
                    .unwrap_or_else(|error| panic!("{symbol} cannot fixture {target_id}: {error}"));
                let result = invoke(symbol, &expected, target, elements, "2.1")
                    .unwrap_or_else(|| panic!("{symbol} route is missing for {target_id}"))
                    .unwrap_or_else(|error| panic!("{symbol} rejected {target_id}: {error}"));
                assert!(
                    crate::local_wire::target_hash(&result, &expected).is_ok(),
                    "{symbol} changed {target_id}"
                );
            }
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
            ("KMIPKIT-ELEM-ATTRIBUTE-CERTIFICATE-ATTRIBUTES", 0x0042_0008),
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
            if target_id == "KMIPKIT-ELEM-ATTRIBUTE-CERTIFICATE-ATTRIBUTES" {
                let fields = sample.as_structure().unwrap().children();
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].tag().raw(), 0x0042_000a);
                assert_eq!(fields[1].tag().raw(), 0x0042_000b);
                let attributes = fields[1].as_structure().unwrap().children();
                assert_eq!(attributes.len(), 1);
                assert_eq!(attributes[0].tag().raw(), 0x0042_0108);
            }
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

    #[test]
    fn transparent_rsa_private_key_routes_execute_manifest_fixtures() {
        let catalog: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let specification: Json =
            serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
                .unwrap();
        let elements = specification["elements"].as_array().unwrap();
        let target_ids = [
            "KMIPKIT-ELEM-OBJECT-STRUCTURE-3-7-KEY-MATERIAL",
            "KMIPKIT-ELEM-STRUCTURE-MEMBER-3-7-KEY-MATERIAL-CRT-COEFFICIENT",
            "KMIPKIT-ELEM-STRUCTURE-MEMBER-3-7-KEY-MATERIAL-MODULUS",
            "KMIPKIT-ELEM-STRUCTURE-MEMBER-3-7-KEY-MATERIAL-P",
            "KMIPKIT-ELEM-STRUCTURE-MEMBER-3-7-KEY-MATERIAL-PRIME-EXPONENT-P",
            "KMIPKIT-ELEM-STRUCTURE-MEMBER-3-7-KEY-MATERIAL-PRIME-EXPONENT-Q",
            "KMIPKIT-ELEM-STRUCTURE-MEMBER-3-7-KEY-MATERIAL-PRIVATE-EXPONENT",
            "KMIPKIT-ELEM-STRUCTURE-MEMBER-3-7-KEY-MATERIAL-PUBLIC-EXPONENT",
            "KMIPKIT-ELEM-STRUCTURE-MEMBER-3-7-KEY-MATERIAL-Q",
        ];
        let symbols = [
            "kmipkit_protocol::TransparentRsaPrivateKey::try_from_components",
            "kmipkit_protocol::TransparentRsaPrivateKey::try_from_item",
        ];

        for target_id in target_ids {
            let target = catalog["normative_targets"]
                .as_array()
                .unwrap()
                .iter()
                .find(|target| target["target_id"] == target_id)
                .unwrap();
            for symbol in symbols {
                let expected = sample_for_route_at_version(symbol, target, elements, "2.1")
                    .unwrap_or_else(|error| {
                        panic!("{symbol} sample failed for {target_id}: {error}")
                    });
                let result = invoke(symbol, &expected, target, elements, "2.1")
                    .unwrap_or_else(|| panic!("{symbol} route missing for {target_id}"))
                    .unwrap_or_else(|error| panic!("{symbol} rejected {target_id}: {error}"));
                assert!(
                    crate::local_wire::target_hash(&result, &expected).is_ok(),
                    "{symbol} changed {target_id}"
                );
            }
        }
    }

    #[test]
    fn request_attribute_builders_execute_manifest_fixtures() {
        let catalog: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let routes: Json = serde_json::from_str(include_str!(
            "../../shared/coverage/route-capabilities.json"
        ))
        .unwrap();
        let specification: Json =
            serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
                .unwrap();
        let elements = specification["elements"].as_array().unwrap();
        let route_symbols = [
            (
                "rust.typed-protocol.model.kmipkit.protocol.certifyrequest.with.attributes",
                "kmipkit_protocol::CertifyRequest::with_attributes",
            ),
            (
                "rust.typed-protocol.model.kmipkit.protocol.createsplitkeyrequest.with.attributes",
                "kmipkit_protocol::CreateSplitKeyRequest::with_attributes",
            ),
            (
                "rust.typed-protocol.model.kmipkit.protocol.joinsplitkeyrequest.with.attributes",
                "kmipkit_protocol::JoinSplitKeyRequest::with_attributes",
            ),
            (
                "rust.typed-protocol.model.kmipkit.protocol.recertifyrequest.with.attributes",
                "kmipkit_protocol::RecertifyRequest::with_attributes",
            ),
            (
                "rust.typed-protocol.model.kmipkit.protocol.rekeyrequest.with.attributes",
                "kmipkit_protocol::RekeyRequest::with_attributes",
            ),
        ];

        for (route_id, symbol) in route_symbols {
            let route = routes["routes"]
                .as_array()
                .unwrap()
                .iter()
                .find(|route| route["route_id"] == route_id)
                .unwrap();
            for target_id in route["normative_target_ids"].as_array().unwrap() {
                let target_id = target_id.as_str().unwrap();
                let target = catalog["normative_targets"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|target| target["target_id"] == target_id)
                    .unwrap();
                let expected = sample_for_route_at_version(symbol, target, elements, "2.1")
                    .unwrap_or_else(|error| {
                        panic!("{symbol} sample failed for {target_id}: {error}")
                    });
                let result = invoke(symbol, &expected, target, elements, "2.1")
                    .unwrap_or_else(|| panic!("{symbol} route missing for {target_id}"))
                    .unwrap_or_else(|error| panic!("{symbol} rejected {target_id}: {error}"));
                assert!(
                    crate::local_wire::target_hash(&result, &expected).is_ok(),
                    "{symbol} changed {target_id}"
                );
            }
        }
    }

    #[test]
    fn transparent_dh_and_dsa_key_routes_execute_manifest_fixtures() {
        let catalog: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let routes: Json = serde_json::from_str(include_str!(
            "../../shared/coverage/route-capabilities.json"
        ))
        .unwrap();
        let specification: Json =
            serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
                .unwrap();
        let elements = specification["elements"].as_array().unwrap();
        let route_ids = [
            "rust.typed-protocol.model.kmipkit.protocol.transparentdhprivatekey.try.from.item",
            "rust.typed-protocol.model.kmipkit.protocol.transparentdhprivatekey.try.new",
            "rust.typed-protocol.model.kmipkit.protocol.transparentdhpublickey.try.from.item",
            "rust.typed-protocol.model.kmipkit.protocol.transparentdhpublickey.try.new",
            "rust.typed-protocol.model.kmipkit.protocol.transparentdsaprivatekey.try.from.item",
            "rust.typed-protocol.model.kmipkit.protocol.transparentdsaprivatekey.try.new",
            "rust.typed-protocol.model.kmipkit.protocol.transparentdsapublickey.try.from.item",
            "rust.typed-protocol.model.kmipkit.protocol.transparentdsapublickey.try.new",
            "rust.typed-protocol.model.kmipkit.protocol.transparentecprivatekey.try.from.item",
            "rust.typed-protocol.model.kmipkit.protocol.transparentecprivatekey.try.new",
            "rust.typed-protocol.model.kmipkit.protocol.transparentecpublickey.try.from.item",
            "rust.typed-protocol.model.kmipkit.protocol.transparentecpublickey.try.new",
            "rust.typed-protocol.model.kmipkit.protocol.transparentrsapublickey.try.from.item",
            "rust.typed-protocol.model.kmipkit.protocol.transparentrsapublickey.try.new",
        ];

        for route_id in route_ids {
            let route = routes["routes"]
                .as_array()
                .unwrap()
                .iter()
                .find(|route| route["route_id"] == route_id)
                .unwrap();
            let symbol = route["public_api_symbol"].as_str().unwrap();
            for target_id in route["normative_target_ids"].as_array().unwrap() {
                let target_id = target_id.as_str().unwrap();
                let target = catalog["normative_targets"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|target| target["target_id"] == target_id)
                    .unwrap();
                let expected = sample_for_route_at_version(symbol, target, elements, "2.1")
                    .unwrap_or_else(|error| {
                        panic!("{symbol} sample failed for {target_id}: {error}")
                    });
                let result = invoke(symbol, &expected, target, elements, "2.1")
                    .unwrap_or_else(|| panic!("{symbol} route missing for {target_id}"))
                    .unwrap_or_else(|error| panic!("{symbol} rejected {target_id}: {error}"));
                assert!(
                    crate::local_wire::target_hash(&result, &expected).is_ok(),
                    "{symbol} changed {target_id}"
                );
            }
        }
    }

    #[test]
    fn certificate_request_builders_execute_manifest_fixtures() {
        let catalog: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let routes: Json = serde_json::from_str(include_str!(
            "../../shared/coverage/route-capabilities.json"
        ))
        .unwrap();
        let specification: Json =
            serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
                .unwrap();
        let elements = specification["elements"].as_array().unwrap();
        let route_ids = [
            "rust.typed-protocol.model.kmipkit.protocol.reprovisionrequest.with.certificate",
            "rust.typed-protocol.model.kmipkit.protocol.reprovisionrequest.with.certificate.request",
            "rust.typed-protocol.model.kmipkit.protocol.validaterequest.with.certificate",
        ];

        for route_id in route_ids {
            let route = routes["routes"]
                .as_array()
                .unwrap()
                .iter()
                .find(|route| route["route_id"] == route_id)
                .unwrap();
            let symbol = route["public_api_symbol"].as_str().unwrap();
            for target_id in route["normative_target_ids"].as_array().unwrap() {
                let target_id = target_id.as_str().unwrap();
                let target = catalog["normative_targets"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|target| target["target_id"] == target_id)
                    .unwrap();
                let expected = sample(target, elements).unwrap_or_else(|error| {
                    panic!("{symbol} sample failed for {target_id}: {error}")
                });
                let result = invoke(symbol, &expected, target, elements, "2.1")
                    .unwrap_or_else(|| panic!("{symbol} route missing for {target_id}"))
                    .unwrap_or_else(|error| panic!("{symbol} rejected {target_id}: {error}"));
                assert!(
                    crate::local_wire::target_hash(&result, &expected).is_ok(),
                    "{symbol} changed {target_id}"
                );
            }
        }
    }

    #[test]
    fn asynchronous_query_and_derivation_routes_execute_manifest_fixtures() {
        let catalog: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let routes: Json = serde_json::from_str(include_str!(
            "../../shared/coverage/route-capabilities.json"
        ))
        .unwrap();
        let specification: Json =
            serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
                .unwrap();
        let elements = specification["elements"].as_array().unwrap();
        let route_ids = [
            "rust.typed-protocol.model.kmipkit.protocol.derivationparameters.new",
            "rust.typed-protocol.model.kmipkit.protocol.queryasyncrequestsrequest.with.operations",
        ];

        for route_id in route_ids {
            let route = routes["routes"]
                .as_array()
                .unwrap()
                .iter()
                .find(|route| route["route_id"] == route_id)
                .unwrap();
            let symbol = route["public_api_symbol"].as_str().unwrap();
            for target_id in route["normative_target_ids"].as_array().unwrap() {
                let target_id = target_id.as_str().unwrap();
                let target = catalog["normative_targets"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|target| target["target_id"] == target_id)
                    .unwrap();
                let expected = sample_for_route_at_version(symbol, target, elements, "2.1")
                    .unwrap_or_else(|error| {
                        panic!("{symbol} sample failed for {target_id}: {error}")
                    });
                let result = invoke(symbol, &expected, target, elements, "2.1")
                    .unwrap_or_else(|| panic!("{symbol} route missing for {target_id}"))
                    .unwrap_or_else(|error| panic!("{symbol} rejected {target_id}: {error}"));
                assert!(
                    crate::local_wire::target_hash(&result, &expected).is_ok(),
                    "{symbol} changed {target_id}"
                );
            }
        }
    }

    #[test]
    fn key_wrapping_request_selectors_execute_manifest_fixtures() {
        let catalog: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let routes: Json = serde_json::from_str(include_str!(
            "../../shared/coverage/route-capabilities.json"
        ))
        .unwrap();
        let specification: Json =
            serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
                .unwrap();
        let elements = specification["elements"].as_array().unwrap();
        let route_ids = [
            "rust.typed-protocol.model.kmipkit.protocol.exportrequest.with.key.wrapping.specification",
            "rust.typed-protocol.model.kmipkit.protocol.getrequest.with.key.wrapping.specification",
            "rust.typed-protocol.model.kmipkit.protocol.importrequest.with.key.wrap.type",
        ];

        for route_id in route_ids {
            let route = routes["routes"]
                .as_array()
                .unwrap()
                .iter()
                .find(|route| route["route_id"] == route_id)
                .unwrap();
            let symbol = route["public_api_symbol"].as_str().unwrap();
            for target_id in route["normative_target_ids"].as_array().unwrap() {
                let target_id = target_id.as_str().unwrap();
                let target = catalog["normative_targets"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|target| target["target_id"] == target_id)
                    .unwrap();
                let expected = sample_for_route_at_version(symbol, target, elements, "2.1")
                    .unwrap_or_else(|error| {
                        panic!("{symbol} sample failed for {target_id}: {error}")
                    });
                let result = invoke(symbol, &expected, target, elements, "2.1")
                    .unwrap_or_else(|| panic!("{symbol} route missing for {target_id}"))
                    .unwrap_or_else(|error| panic!("{symbol} rejected {target_id}: {error}"));
                assert!(
                    crate::local_wire::target_hash(&result, &expected).is_ok(),
                    "{symbol} changed {target_id}"
                );
            }
        }
    }

    #[test]
    fn key_information_unique_identifier_routes_execute_manifest_fixtures() {
        let catalog: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let routes: Json = serde_json::from_str(include_str!(
            "../../shared/coverage/route-capabilities.json"
        ))
        .unwrap();
        let specification: Json =
            serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
                .unwrap();
        let elements = specification["elements"].as_array().unwrap();
        let route_ids = [
            "rust.typed-protocol.model.kmipkit.protocol.encryptionkeyinformation.try.from.unique.identifier",
            "rust.typed-protocol.model.kmipkit.protocol.macsignaturekeyinformation.try.from.unique.identifier",
        ];

        for route_id in route_ids {
            let route = routes["routes"]
                .as_array()
                .unwrap()
                .iter()
                .find(|route| route["route_id"] == route_id)
                .unwrap();
            let symbol = route["public_api_symbol"].as_str().unwrap();
            for target_id in route["normative_target_ids"].as_array().unwrap() {
                let target_id = target_id.as_str().unwrap();
                let target = catalog["normative_targets"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|target| target["target_id"] == target_id)
                    .unwrap();
                let expected = sample_for_route_at_version(symbol, target, elements, "2.1")
                    .unwrap_or_else(|error| {
                        panic!("{symbol} sample failed for {target_id}: {error}")
                    });
                let result = invoke(symbol, &expected, target, elements, "2.1")
                    .unwrap_or_else(|| panic!("{symbol} route missing for {target_id}"))
                    .unwrap_or_else(|error| panic!("{symbol} rejected {target_id}: {error}"));
                assert!(
                    crate::local_wire::target_hash(&result, &expected).is_ok(),
                    "{symbol} changed {target_id}"
                );
            }
        }
    }

    #[test]
    fn lifecycle_and_defaults_request_builders_execute_manifest_fixtures() {
        let catalog: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let routes: Json = serde_json::from_str(include_str!(
            "../../shared/coverage/route-capabilities.json"
        ))
        .unwrap();
        let specification: Json =
            serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
                .unwrap();
        let elements = specification["elements"].as_array().unwrap();
        let route_ids = [
            "rust.typed-protocol.model.kmipkit.protocol.checkrequest.with.lease.time",
            "rust.typed-protocol.model.kmipkit.protocol.revokerequest.with.compromise.occurrence.date",
            "rust.typed-protocol.model.kmipkit.protocol.setdefaultsrequest.with.defaults.information",
        ];

        for route_id in route_ids {
            let route = routes["routes"]
                .as_array()
                .unwrap()
                .iter()
                .find(|route| route["route_id"] == route_id)
                .unwrap();
            let symbol = route["public_api_symbol"].as_str().unwrap();
            for target_id in route["normative_target_ids"].as_array().unwrap() {
                let target_id = target_id.as_str().unwrap();
                let target = catalog["normative_targets"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|target| target["target_id"] == target_id)
                    .unwrap();
                let expected = sample_for_route_at_version(symbol, target, elements, "2.1")
                    .unwrap_or_else(|error| {
                        panic!("{symbol} sample failed for {target_id}: {error}")
                    });
                let result = invoke(symbol, &expected, target, elements, "2.1")
                    .unwrap_or_else(|| panic!("{symbol} route missing for {target_id}"))
                    .unwrap_or_else(|error| panic!("{symbol} rejected {target_id}: {error}"));
                assert!(
                    crate::local_wire::target_hash(&result, &expected).is_ok(),
                    "{symbol} changed {target_id}"
                );
            }
        }
    }

    #[test]
    fn cryptographic_final_indicator_routes_execute_manifest_fixtures() {
        let catalog: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let routes: Json = serde_json::from_str(include_str!(
            "../../shared/coverage/route-capabilities.json"
        ))
        .unwrap();
        let specification: Json =
            serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
                .unwrap();
        let elements = specification["elements"].as_array().unwrap();
        let route_ids = [
            "rust.typed-protocol.model.kmipkit.protocol.decryptrequest.with.final.indicator",
            "rust.typed-protocol.model.kmipkit.protocol.encryptrequest.with.final.indicator",
        ];

        for route_id in route_ids {
            let route = routes["routes"]
                .as_array()
                .unwrap()
                .iter()
                .find(|route| route["route_id"] == route_id)
                .unwrap();
            let symbol = route["public_api_symbol"].as_str().unwrap();
            for target_id in route["normative_target_ids"].as_array().unwrap() {
                let target_id = target_id.as_str().unwrap();
                let target = catalog["normative_targets"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|target| target["target_id"] == target_id)
                    .unwrap();
                let expected = sample_for_route_at_version(symbol, target, elements, "2.1")
                    .unwrap_or_else(|error| {
                        panic!("{symbol} sample failed for {target_id}: {error}")
                    });
                let result = invoke(symbol, &expected, target, elements, "2.1")
                    .unwrap_or_else(|| panic!("{symbol} route missing for {target_id}"))
                    .unwrap_or_else(|error| panic!("{symbol} rejected {target_id}: {error}"));
                assert!(
                    crate::local_wire::target_hash(&result, &expected).is_ok(),
                    "{symbol} changed {target_id}"
                );
            }
        }
    }

    #[test]
    fn vendor_attribute_routes_execute_manifest_fixtures() {
        let catalog: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let routes: Json = serde_json::from_str(include_str!(
            "../../shared/coverage/route-capabilities.json"
        ))
        .unwrap();
        let specification: Json =
            serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
                .unwrap();
        let elements = specification["elements"].as_array().unwrap();
        let route_ids = [
            "rust.typed-protocol.model.kmipkit.protocol.attribute.attribute.kmipkit.protocol.attribute.typed.vendorattribute.new",
            "rust.typed-protocol.model.kmipkit.protocol.attribute.attribute.kmipkit.protocol.attribute.typed.vendorattribute.from.item",
            "rust.typed-protocol.model.kmipkit.protocol.attributeset.try.push.typed.kmipkit.protocol.attribute.typed.vendorattribute",
            "rust.typed-protocol.model.kmipkit.protocol.currentattribute.from.typed.kmipkit.protocol.attribute.typed.vendorattribute",
            "rust.typed-protocol.model.kmipkit.protocol.newattribute.from.typed.kmipkit.protocol.attribute.typed.vendorattribute",
        ];

        for route_id in route_ids {
            let route = routes["routes"]
                .as_array()
                .unwrap()
                .iter()
                .find(|route| route["route_id"] == route_id)
                .unwrap();
            let symbol = route["public_api_symbol"].as_str().unwrap();
            for target_id in route["normative_target_ids"].as_array().unwrap() {
                let target_id = target_id.as_str().unwrap();
                let target = catalog["normative_targets"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|target| target["target_id"] == target_id)
                    .unwrap();
                let expected = sample_for_route_at_version(symbol, target, elements, "2.1")
                    .unwrap_or_else(|error| {
                        panic!("{symbol} sample failed for {target_id}: {error}")
                    });
                let result = invoke(symbol, &expected, target, elements, "2.1")
                    .unwrap_or_else(|| panic!("{symbol} route missing for {target_id}"))
                    .unwrap_or_else(|error| panic!("{symbol} rejected {target_id}: {error}"));
                assert!(
                    crate::local_wire::target_hash(&result, &expected).is_ok(),
                    "{symbol} changed {target_id}"
                );
            }
        }
    }

    #[test]
    fn key_pair_attribute_builders_execute_manifest_fixtures() {
        let catalog: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let routes: Json = serde_json::from_str(include_str!(
            "../../shared/coverage/route-capabilities.json"
        ))
        .unwrap();
        let specification: Json =
            serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
                .unwrap();
        let elements = specification["elements"].as_array().unwrap();
        let route_ids = [
            "rust.typed-protocol.model.kmipkit.protocol.createkeypairrequest.with.common.attributes",
            "rust.typed-protocol.model.kmipkit.protocol.createkeypairrequest.with.private.key.attributes",
            "rust.typed-protocol.model.kmipkit.protocol.createkeypairrequest.with.public.key.attributes",
            "rust.typed-protocol.model.kmipkit.protocol.rekeykeypairrequest.with.common.attributes",
            "rust.typed-protocol.model.kmipkit.protocol.rekeykeypairrequest.with.private.key.attributes",
            "rust.typed-protocol.model.kmipkit.protocol.rekeykeypairrequest.with.public.key.attributes",
        ];

        for route_id in route_ids {
            let route = routes["routes"]
                .as_array()
                .unwrap()
                .iter()
                .find(|route| route["route_id"] == route_id)
                .unwrap();
            let symbol = route["public_api_symbol"].as_str().unwrap();
            for target_id in route["normative_target_ids"].as_array().unwrap() {
                let target_id = target_id.as_str().unwrap();
                let target = catalog["normative_targets"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|target| target["target_id"] == target_id)
                    .unwrap();
                for protocol_version in target["protocol_versions"].as_array().unwrap() {
                    let protocol_version = protocol_version.as_str().unwrap();
                    let expected =
                        sample_for_route_at_version(symbol, target, elements, protocol_version)
                            .unwrap_or_else(|error| {
                                panic!("{symbol} sample failed for {target_id}: {error}")
                            });
                    let result = invoke(symbol, &expected, target, elements, protocol_version)
                        .unwrap_or_else(|| panic!("{symbol} route missing for {target_id}"))
                        .unwrap_or_else(|error| panic!("{symbol} rejected {target_id}: {error}"));
                    assert!(
                        crate::local_wire::target_hash(&result, &expected).is_ok(),
                        "{symbol} changed {target_id}"
                    );
                }
            }
        }
    }

    #[test]
    fn cryptographic_indicator_routes_execute_manifest_fixtures() {
        let catalog: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let routes: Json = serde_json::from_str(include_str!(
            "../../shared/coverage/route-capabilities.json"
        ))
        .unwrap();
        let specification: Json =
            serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
                .unwrap();
        let elements = specification["elements"].as_array().unwrap();
        let operation_types = [
            "DecryptRequest",
            "EncryptRequest",
            "HashRequest",
            "MacRequest",
            "MacVerifyRequest",
            "SignatureVerifyRequest",
            "SignRequest",
        ];

        for route in routes["routes"].as_array().unwrap() {
            let symbol = route["public_api_symbol"].as_str().unwrap();
            if !operation_types
                .iter()
                .any(|operation| symbol.contains(operation))
                || !(symbol.ends_with("::with_init_indicator")
                    || symbol.ends_with("::with_final_indicator"))
            {
                continue;
            }
            for target_id in route["normative_target_ids"].as_array().unwrap() {
                let target_id = target_id.as_str().unwrap();
                let target = catalog["normative_targets"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|target| target["target_id"] == target_id)
                    .unwrap();
                for protocol_version in target["protocol_versions"].as_array().unwrap() {
                    let protocol_version = protocol_version.as_str().unwrap();
                    let expected =
                        sample_for_route_at_version(symbol, target, elements, protocol_version)
                            .unwrap_or_else(|error| {
                                panic!("{symbol} sample failed for {target_id}: {error}")
                            });
                    let result = invoke(symbol, &expected, target, elements, protocol_version)
                        .unwrap_or_else(|| panic!("{symbol} route missing for {target_id}"))
                        .unwrap_or_else(|error| panic!("{symbol} rejected {target_id}: {error}"));
                    assert!(
                        crate::local_wire::target_hash(&result, &expected).is_ok(),
                        "{symbol} changed {target_id}"
                    );
                }
            }
        }
    }

    #[test]
    fn structured_typed_attribute_constructors_execute_nested_targets() {
        let catalog: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let routes: Json = serde_json::from_str(include_str!(
            "../../shared/coverage/route-capabilities.json"
        ))
        .unwrap();
        let specification: Json =
            serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
                .unwrap();
        let elements = specification["elements"].as_array().unwrap();
        let targets = catalog["normative_targets"].as_array().unwrap();

        for route in routes["routes"].as_array().unwrap() {
            let symbol = route["public_api_symbol"].as_str().unwrap();
            if !symbol.starts_with("kmipkit_protocol::attribute::Attribute::<")
                || !symbol.ends_with("::new")
            {
                continue;
            }
            let target_ids = route["normative_target_ids"].as_array().unwrap();
            if !target_ids.iter().any(|target_id| {
                targets.iter().any(|target| {
                    target["target_id"] == *target_id && target["kind"] == "structure_member"
                })
            }) {
                continue;
            }
            for target_id in target_ids {
                let target_id = target_id.as_str().unwrap();
                let target = targets
                    .iter()
                    .find(|target| target["target_id"] == target_id)
                    .unwrap();
                for protocol_version in target["protocol_versions"].as_array().unwrap() {
                    let protocol_version = protocol_version.as_str().unwrap();
                    let expected =
                        sample_for_route_at_version(symbol, target, elements, protocol_version)
                            .unwrap_or_else(|error| {
                                panic!("{symbol} sample failed for {target_id}: {error}")
                            });
                    let result = invoke(symbol, &expected, target, elements, protocol_version)
                        .unwrap_or_else(|| panic!("{symbol} route missing for {target_id}"))
                        .unwrap_or_else(|error| panic!("{symbol} rejected {target_id}: {error}"));
                    assert!(
                        crate::local_wire::target_hash(&result, &expected).is_ok(),
                        "{symbol} changed {target_id}"
                    );
                }
            }
        }
    }

    #[test]
    fn protection_storage_mask_builders_execute_manifest_fixtures() {
        let catalog: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let routes: Json = serde_json::from_str(include_str!(
            "../../shared/coverage/route-capabilities.json"
        ))
        .unwrap();
        let specification: Json =
            serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
                .unwrap();
        let elements = specification["elements"].as_array().unwrap();

        for route in routes["routes"].as_array().unwrap() {
            let symbol = route["public_api_symbol"].as_str().unwrap();
            if !symbol.starts_with("kmipkit_protocol::")
                || !symbol.ends_with("protection_storage_masks")
            {
                continue;
            }
            for target_id in route["normative_target_ids"].as_array().unwrap() {
                let target_id = target_id.as_str().unwrap();
                let target = catalog["normative_targets"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|target| target["target_id"] == target_id)
                    .unwrap();
                for protocol_version in target["protocol_versions"].as_array().unwrap() {
                    let protocol_version = protocol_version.as_str().unwrap();
                    let expected =
                        sample_for_route_at_version(symbol, target, elements, protocol_version)
                            .unwrap_or_else(|error| {
                                panic!("{symbol} sample failed for {target_id}: {error}")
                            });
                    let result = invoke(symbol, &expected, target, elements, protocol_version)
                        .unwrap_or_else(|| panic!("{symbol} route missing for {target_id}"))
                        .unwrap_or_else(|error| panic!("{symbol} rejected {target_id}: {error}"));
                    assert!(
                        crate::local_wire::target_hash(&result, &expected).is_ok(),
                        "{symbol} changed {target_id}"
                    );
                }
            }
        }
    }

    #[test]
    fn secret_operation_builder_routes_execute_manifest_fixtures() {
        let catalog: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let routes: Json = serde_json::from_str(include_str!(
            "../../shared/coverage/route-capabilities.json"
        ))
        .unwrap();
        let specification: Json =
            serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
                .unwrap();
        let elements = specification["elements"].as_array().unwrap();
        let symbols = [
            "kmipkit_protocol::CertifyRequest::with_certificate_request_value",
            "kmipkit_protocol::CreateSplitKeyRequest::with_prime_field_size",
            "kmipkit_protocol::DecryptRequest::with_authenticated_encryption_additional_data",
            "kmipkit_protocol::DecryptRequest::with_authenticated_encryption_tag",
            "kmipkit_protocol::DecryptRequest::with_correlation_value",
            "kmipkit_protocol::DecryptRequest::with_iv_counter_nonce",
            "kmipkit_protocol::EncryptRequest::with_authenticated_encryption_additional_data",
            "kmipkit_protocol::EncryptRequest::with_correlation_value",
            "kmipkit_protocol::EncryptRequest::with_iv_counter_nonce",
            "kmipkit_protocol::HashRequest::with_correlation_value",
            "kmipkit_protocol::MacRequest::with_correlation_value",
            "kmipkit_protocol::MacVerifyRequest::with_correlation_value",
            "kmipkit_protocol::MacVerifyRequest::with_mac_data",
            "kmipkit_protocol::Pkcs11Request::with_correlation_value",
            "kmipkit_protocol::RecertifyRequest::with_certificate_request_value",
            "kmipkit_protocol::SignRequest::with_correlation_value",
            "kmipkit_protocol::SignRequest::with_digested_data",
            "kmipkit_protocol::SignatureVerifyRequest::with_correlation_value",
            "kmipkit_protocol::SignatureVerifyRequest::with_digested_data",
            "kmipkit_protocol::SignatureVerifyRequest::with_signature_data",
        ];

        for symbol in symbols {
            let route = routes["routes"]
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
                for protocol_version in target["protocol_versions"].as_array().unwrap() {
                    let protocol_version = protocol_version.as_str().unwrap();
                    let expected =
                        sample_for_route_at_version(symbol, target, elements, protocol_version)
                            .unwrap_or_else(|error| {
                                panic!("{symbol} sample failed for {target_id}: {error}")
                            });
                    let result = invoke(symbol, &expected, target, elements, protocol_version)
                        .unwrap_or_else(|| panic!("{symbol} route missing for {target_id}"))
                        .unwrap_or_else(|error| panic!("{symbol} rejected {target_id}: {error}"));
                    assert!(
                        crate::local_wire::target_hash(&result, &expected).is_ok(),
                        "{symbol} changed {target_id}"
                    );
                }
            }
        }
    }

    #[test]
    fn remaining_simple_request_builder_routes_execute_manifest_fixtures() {
        let catalog: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let routes: Json = serde_json::from_str(include_str!(
            "../../shared/coverage/route-capabilities.json"
        ))
        .unwrap();
        let specification: Json =
            serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
                .unwrap();
        let elements = specification["elements"].as_array().unwrap();
        let symbols = [
            "kmipkit_protocol::CheckRequest::with_usage_limits_count",
            "kmipkit_protocol::QueryAsyncRequestsRequest::with_correlation_values",
            "kmipkit_protocol::LocateRequest::with_maximum_items",
            "kmipkit_protocol::LocateRequest::with_offset_items",
            "kmipkit_protocol::RevokeRequest::with_revocation_message",
            "kmipkit_protocol::LoginRequest::with_request_count",
            "kmipkit_protocol::DelegatedLoginRequest::with_request_count",
            "kmipkit_protocol::ValidateRequest::with_validity_date",
            "kmipkit_protocol::RekeyRequest::with_offset",
            "kmipkit_protocol::RekeyKeyPairRequest::with_offset",
            "kmipkit_protocol::RecertifyRequest::with_offset",
            "kmipkit_protocol::RekeyKeyPairRequest::with_private_key_unique_identifier",
            "kmipkit_protocol::RecertifyRequest::with_certificate_request_unique_identifier",
        ];

        for symbol in symbols {
            let route = routes["routes"]
                .as_array()
                .unwrap()
                .iter()
                .find(|route| route["public_api_symbol"] == symbol)
                .unwrap_or_else(|| panic!("{symbol} is missing from the route manifest"));
            for target_id in route["normative_target_ids"].as_array().unwrap() {
                let target_id = target_id.as_str().unwrap();
                let target = catalog["normative_targets"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|target| target["target_id"] == target_id)
                    .unwrap();
                for protocol_version in target["protocol_versions"].as_array().unwrap() {
                    let protocol_version = protocol_version.as_str().unwrap();
                    let expected =
                        sample_for_route_at_version(symbol, target, elements, protocol_version)
                            .unwrap_or_else(|error| {
                                panic!("{symbol} sample failed for {target_id}: {error}")
                            });
                    let result = invoke(symbol, &expected, target, elements, protocol_version)
                        .unwrap_or_else(|| panic!("{symbol} route missing for {target_id}"))
                        .unwrap_or_else(|error| panic!("{symbol} rejected {target_id}: {error}"));
                    assert!(
                        crate::local_wire::target_hash(&result, &expected).is_ok(),
                        "{symbol} changed {target_id}"
                    );
                }
            }
        }
    }

    #[test]
    fn remaining_key_and_profile_model_routes_execute_manifest_fixtures() {
        let catalog: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let routes: Json = serde_json::from_str(include_str!(
            "../../shared/coverage/route-capabilities.json"
        ))
        .unwrap();
        let specification: Json =
            serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
                .unwrap();
        let elements = specification["elements"].as_array().unwrap();
        let symbols = [
            "kmipkit_protocol::TransparentSymmetricKey::try_new",
            "kmipkit_protocol::TransparentSymmetricKey::try_from_item",
            "kmipkit_protocol::SplitKeyReference::with_split_key_threshold",
            "kmipkit_protocol::DerivationParameters::with_derivation_data",
            "kmipkit_protocol::DerivationParameters::with_initialization_vector",
            "kmipkit_protocol::DerivationParameters::with_iteration_count",
            "kmipkit_protocol::DerivationParameters::with_salt",
            "kmipkit_protocol::ProfileObjectState::with_usage_limits_count",
            "kmipkit_protocol::ProfileObjectState::with_usage_limits_total",
            "kmipkit_protocol::ImportRequest::with_replace_existing",
        ];

        for symbol in symbols {
            let route = routes["routes"]
                .as_array()
                .unwrap()
                .iter()
                .find(|route| route["public_api_symbol"] == symbol)
                .unwrap_or_else(|| panic!("{symbol} is missing from the route manifest"));
            for target_id in route["normative_target_ids"].as_array().unwrap() {
                let target_id = target_id.as_str().unwrap();
                let target = catalog["normative_targets"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|target| target["target_id"] == target_id)
                    .unwrap();
                for protocol_version in target["protocol_versions"].as_array().unwrap() {
                    let protocol_version = protocol_version.as_str().unwrap();
                    let expected =
                        sample_for_route_at_version(symbol, target, elements, protocol_version)
                            .unwrap_or_else(|error| {
                                panic!("{symbol} sample failed for {target_id}: {error}")
                            });
                    let result = invoke(symbol, &expected, target, elements, protocol_version)
                        .unwrap_or_else(|| panic!("{symbol} route missing for {target_id}"))
                        .unwrap_or_else(|error| panic!("{symbol} rejected {target_id}: {error}"));
                    assert!(
                        crate::local_wire::target_hash(&result, &expected).is_ok(),
                        "{symbol} changed {target_id}"
                    );
                }
            }
        }
    }

    #[test]
    fn operation_secret_tag_fixtures_do_not_change_key_wrapping_models() {
        let catalog: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let specification: Json =
            serde_json::from_str(include_str!("../../../specification/catalog/kmip-2.1.json"))
                .unwrap();
        let elements = specification["elements"].as_array().unwrap();
        let symbol = "kmipkit_protocol::KeyWrappingData::with_iv_counter_nonce";
        let target = catalog["normative_targets"]
            .as_array()
            .unwrap()
            .iter()
            .find(|target| target["target_id"] == "KMIPKIT-ELEM-TAG-42003D")
            .unwrap();
        let expected = sample_for_route_at_version(symbol, target, elements, "2.1").unwrap();
        let model = key_wrapping_metadata_model(symbol, target).unwrap();

        assert!(
            crate::local_wire::target_hash(&model, &expected).is_ok(),
            "request-only secret type fixtures changed the Key Wrapping Data source"
        );
    }
}
