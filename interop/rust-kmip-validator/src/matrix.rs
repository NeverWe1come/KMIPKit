//! Execute only named public routes; pending recipes never produce call evidence.
use crate::models::{Result, hash};
use kmipkit::ttlv::{Item, Value, ValueView};
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

fn sample(target: &Json, elements: &[Json]) -> Result<Item> {
    let kind = text(target, "kind")?;
    let mut name = text(target, "name")?;
    if ["enumeration_value", "bitmask_value"].contains(&kind)
        && let Some(parent) = target["parent_element_ids"]
            .as_array()
            .and_then(|v| v.first())
            .and_then(Json::as_str)
        && let Some(element) = elements.iter().find(|e| e["element_id"] == parent)
    {
        name = element["name"].as_str().ok_or("target name missing")?;
    }
    let tag = if kind == "tag" {
        u32::from_str_radix(text(target, "wire_value")?, 16).map_err(|_| "invalid target tag")?
    } else {
        elements
            .iter()
            .find(|e| e["kind"] == "tag" && e["name"] == name && e["allocation"] == "assigned")
            .and_then(|e| e["wire_value"].as_str())
            .and_then(|v| u32::from_str_radix(v, 16).ok())
            .unwrap_or(0x0042_0001)
    };
    let wire_value = target["wire_value"]
        .as_str()
        .and_then(|v| u32::from_str_radix(v, 16).ok())
        .unwrap_or(1);
    let encoding = target["source_encoding"]
        .as_str()
        .or_else(|| target["value_type"].as_str())
        .unwrap_or(name);
    let value = if ["enumeration_value", "referenced_enum_value"].contains(&kind) {
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
            "Date Time Extended" => Value::date_time_extended(1),
            "Interval" => Value::interval(1),
            "Text String" | "Text String, Enumeration or Integer" => {
                Value::text_string("KMIPKit".to_owned())
            }
            _ => Value::structure(kmipkit::ttlv::Structure::new()),
        }
    };
    crate::driver::item(tag, value)
}

fn invoke(symbol: &str, sample: &Item) -> Option<Result<Item>> {
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
    let mut seen = HashSet::new();
    for row in &rows {
        if row["language"] != "rust" || row["mode"] != "local" {
            return Err("local matrix requires Rust local cells");
        }
        let route = manifest["routes"]
            .as_array()
            .ok_or("route manifest invalid")?
            .iter()
            .find(|route| route["route_id"] == row["route_id"])
            .ok_or("unknown matrix route")?;
        for field in [
            "public_api_symbol",
            "artifact_commit",
            "mode",
            "language",
            "execution_route_id",
        ] {
            if route[field] != row[field] {
                return Err("matrix route metadata mismatch");
            }
        }
        if row["transport"] != Json::Null
            || row["encoding"] != "TTLV"
            || row["execution_api_symbol"] != Json::Null
            || !route["actions"]
                .as_array()
                .is_some_and(|a| a.contains(&row["action"]))
        {
            return Err("invalid local route axes");
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
        let Ok(sample) = sample(target, elements) else {
            pending += 1;
            continue;
        };
        let Some(result) = invoke(symbol, &sample) else {
            pending += 1;
            continue;
        };
        let result = result.and_then(|item| crate::local_wire::roundtrip(&item));
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
