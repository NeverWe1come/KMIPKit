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

#[allow(clippy::too_many_lines)] // Exact source tag/type selection remains auditable in one recipe.
fn sample(target: &Json, elements: &[Json]) -> Result<Item> {
    if let Some(fixture) = crate::local_credentials::fixture(target) {
        return fixture;
    }
    let kind = text(target, "kind")?;
    let mut name = text(target, "name")?;
    if ["enumeration_value", "bitmask_value", "object_type"].contains(&kind)
        && let Some(parent) = target["parent_element_ids"]
            .as_array()
            .and_then(|v| v.first())
            .and_then(Json::as_str)
        && let Some(element) = elements.iter().find(|e| e["element_id"] == parent)
    {
        name = element["name"].as_str().ok_or("target name missing")?;
    }
    let operation = if kind == "operation" {
        operation_value(name).ok_or("operation target needs a concrete public enum value")?
    } else {
        0
    };
    let tag = if kind == "operation" {
        0x0042_005c
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
            .find(|e| e["kind"] == "tag" && e["name"] == name && e["allocation"] == "assigned")
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
            Some(value)
                if [
                    "enumeration_value",
                    "referenced_enum_value",
                    "bitmask_value",
                    "object_type",
                ]
                .contains(&kind) =>
            {
                unsigned_wire_value(value).ok_or("target range needs a concrete fixture recipe")?
            }
            _ => 1,
        }
    };
    let encoding = target["source_encoding"]
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
                .find(|element| element["name"] == name && element["source_encoding"].is_string())
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
        .unwrap_or(if kind == "data_type" { name } else { "" });
    let value = if [
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
        match encoding {
            "Boolean" => Value::boolean(true),
            "Integer" => Value::integer(256),
            "Long Integer" => Value::long_integer(-17),
            "Big Integer" => Value::big_integer(vec![0; 8]),
            "Enumeration" => Value::enumeration(1),
            "Byte String" => Value::byte_string(vec![0x4b]),
            "Date Time" | "Date-Time" => Value::date_time(1),
            "Date Time Extended" | "Date-Time Extended" => Value::date_time_extended(1),
            "Interval" => Value::interval(1),
            "Text String" | "Text String, Enumeration or Integer" => {
                Value::text_string("KMIPKit".to_owned())
            }
            // §4.16 Table 59 permits every Cryptographic Parameters member to be absent.
            "Structure" if tag == 0x0042_002b => Value::structure(kmipkit::ttlv::Structure::new()),
            "Structure" if kind == "data_type" => {
                let mut structure = kmipkit::ttlv::Structure::new();
                structure
                    .try_push(crate::driver::item(0x0042_000d, Value::integer(1))?)
                    .map_err(|_| "local structure rejected")?;
                Value::structure(structure)
            }
            _ if kind == "tag" => Value::boolean(true),
            _ => return Err("target value needs a concrete fixture recipe"),
        }
    };
    crate::driver::item(tag, value)
}

#[allow(clippy::too_many_lines)] // Explicit public route dispatch stays together for review.
fn invoke(symbol: &str, sample: &Item, target: &Json, elements: &[Json]) -> Option<Result<Item>> {
    if let Some(result) = crate::local_credentials::invoke(symbol, sample, target, elements) {
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
        let children = sample.as_structure()?;
        if symbol.ends_with("::try_push") && children.children().is_empty() {
            return None;
        }
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
    if let Some(result) = crate::local_models::invoke(symbol, sample) {
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

fn expected_tag_decode_rejection(row: &Json, target: &Json, catalog: &Json) -> Option<u32> {
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
    let case = catalog["test_cases"]
        .as_array()?
        .iter()
        .find(|case| case["case_id"] == row["case_id"])?;
    let requires = &case["requires"];
    if !["tag", "tag_range"].contains(&requires["target_kind"].as_str()?)
        || !["reserved", "unused"].contains(&requires["allocation"].as_str()?)
    {
        return None;
    }
    let wire_value = text(target, "wire_value").ok()?;
    let start = wire_value
        .split(['-', '–', '—'])
        .next()?
        .trim()
        .trim_start_matches("0x")
        .replace(['X', 'x'], "0");
    u32::from_str_radix(&start, 16).ok()
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
    for row in &rows {
        let target = catalog["normative_targets"]
            .as_array()
            .ok_or("catalog invalid")?
            .iter()
            .find(|t| t["target_id"] == row["target_id"])
            .ok_or("unknown normative target")?;
        let symbol = text(row, "public_api_symbol")?;
        versions.insert(text(row, "protocol_version")?.to_owned());
        if let Some(raw_tag) = expected_tag_decode_rejection(row, target, &catalog) {
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
                event["decode_rejection_code"] = "tag_not_allocated".into();
            } else {
                event["failure_code"] = "expected_tag_rejection_not_observed".into();
            }
            events.push(event);
            continue;
        }
        let Ok(sample) = sample(target, elements) else {
            pending += 1;
            continue;
        };
        let Some(result) = invoke(symbol, &sample, target, elements) else {
            pending += 1;
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
    }
    Ok(success)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vendor_attribute_target_uses_a_table_150_structure_fixture() {
        let cases: Json = serde_json::from_str(include_str!(
            "../../shared/catalog/kmipkit-1.0-test-catalog.json"
        ))
        .unwrap();
        let specification: Json = serde_json::from_str(include_str!(
            "../../../specification/catalog/kmip-2.1.json"
        ))
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
        let result = invoke("kmipkit_protocol::AttributeSet::try_new", &fixture, target, elements)
            .unwrap()
            .unwrap();
        assert!(crate::local_wire::target_hash(&result, &fixture).is_ok());
    }
}
