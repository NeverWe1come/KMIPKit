//! Collect local draft and generic TTLV evidence from the installed public API.
#![forbid(unsafe_code)]

#[path = "../generic_wire.rs"]
mod generic_wire;

use clap::{Parser, ValueEnum};
use kmipkit::protocol::named_values_generated::values::enumeration::{
    key_format_type, object_type,
};
use kmipkit::ttlv::{Item, ItemType, ModelError, RawTag, Structure, Value as TtlvValue, ValueView};
use kmipkit::{ClientBatchItem, ClientRequest, ProtocolVersion, VersionGate};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::process::ExitCode;

const TTLV_TYPES: [ItemType; 14] = [
    ItemType::Structure,
    ItemType::Integer,
    ItemType::LongInteger,
    ItemType::BigInteger,
    ItemType::Enumeration,
    ItemType::Boolean,
    ItemType::TextString,
    ItemType::ByteString,
    ItemType::DateTime,
    ItemType::Interval,
    ItemType::DateTimeExtended,
    ItemType::Identifier,
    ItemType::Reference,
    ItemType::NameReference,
];

#[derive(Clone, Copy, ValueEnum)]
enum Scope {
    Local,
}

#[derive(Parser)]
struct Args {
    #[arg(long)]
    plan: PathBuf,
    #[arg(long)]
    matrix_sha256: String,
    #[arg(long)]
    scope: Scope,
    #[arg(long)]
    evidence: PathBuf,
    #[arg(long)]
    profile: Option<PathBuf>,
}

fn check(symbol: &str) -> Option<bool> {
    let (make, expected): (fn(Structure) -> ClientBatchItem, u32) =
        match symbol.strip_prefix("kmipkit_client::ClientBatchItem::")? {
            "create_group_draft" => (ClientBatchItem::create_group_draft, 0x3c),
            "obliterate_draft" => (ClientBatchItem::obliterate_draft, 0x3d),
            "create_user_draft" => (ClientBatchItem::create_user_draft, 0x3e),
            "create_credential_draft" => (ClientBatchItem::create_credential_draft, 0x3f),
            "deactivate_draft" => (ClientBatchItem::deactivate_draft, 0x40),
            "encapsulate_draft" => (ClientBatchItem::encapsulate_draft, 0x41),
            "decapsulate_draft" => (ClientBatchItem::decapsulate_draft, 0x42),
            _ => return None,
        };
    let fields = [(0x0054_0010, 17), (0x0042_0001, 29), (0x0054_0010, 31)];
    let mut payload = Structure::new();
    for (tag, value) in fields {
        let tag = RawTag::new(tag).ok()?.try_checked().ok()?;
        payload
            .try_push(Item::new(tag, TtlvValue::integer(value)).ok()?)
            .ok()?;
    }
    let item = make(payload);
    let ClientRequest::Kmip30Draft { payload, .. } = item.request() else {
        return Some(false);
    };
    let children = payload.view().children();
    let preserved = children.len() == fields.len()
        && children.iter().zip(fields).all(|(item, (tag, value))| {
            item.tag().raw() == tag
                && item.with_value(|actual| matches!(actual, ValueView::Integer(n) if *n == value))
        });
    Some(item.operation() == expected && preserved)
}

fn check_tag(
    row: &Value,
    symbol: &str,
    tags: &HashMap<&str, bool>,
) -> Option<(&'static str, bool)> {
    if row["mode"] != "local" || row["case_id"] != "test-enum-tag" {
        return None;
    }
    let target = row["target_id"].as_str()?;
    let assigned = *tags.get(target)?;
    let code = u32::from_str_radix(target.strip_prefix("KMIPKIT-ELEM-TAG-")?, 16).ok()?;
    match symbol {
        "kmipkit_ttlv::RawTag::new" => Some((
            "generic_ttlv_raw_tag_value",
            RawTag::new(code).is_ok_and(|tag| tag.raw() == code),
        )),
        "kmipkit_ttlv::RawTag::try_checked" => Some((
            "generic_ttlv_checked_tag_value",
            RawTag::new(code).is_ok_and(|raw| match raw.try_checked() {
                Ok(tag) => assigned && tag.raw() == code,
                Err(error) => !assigned && error == ModelError::TagNotAllocated,
            }),
        )),
        _ => None,
    }
}

fn run(args: Args) -> Result<bool, Box<dyn std::error::Error>> {
    let catalog: Value = serde_json::from_str(include_str!(
        "../../../shared/catalog/kmipkit-2x-matrix-catalog.json"
    ))?;
    let tags = catalog["normative_targets"]
        .as_array()
        .ok_or("normative targets are missing")?
        .iter()
        .filter(|target| target["kind"] == "tag")
        .filter_map(|target| {
            Some((
                target["target_id"].as_str()?,
                target["allocation"] == "assigned",
            ))
        })
        .collect();
    let mut events = Vec::new();
    let mut all_passed = true;
    for line in BufReader::new(File::open(args.plan)?).lines() {
        let row: Value = serde_json::from_str(&line?)?;
        if row["language"] != "rust"
            || row["action"] != "construct"
            || row["execution_status"] != "required"
        {
            continue;
        }
        let symbol = row["public_api_symbol"]
            .as_str()
            .ok_or("missing API symbol")?;
        let outcome = match row["protocol_version"].as_str() {
            Some("3.0") if row["kind"] == "public_route" => {
                check(symbol).map(|passed| ("draft_operation_and_ordered_payload", passed))
            }
            Some("2.0" | "2.1") => check_tag(&row, symbol, &tags),
            _ => None,
        };
        let Some((assertion, passed)) = outcome else {
            continue;
        };
        let obligation = row["obligation_id"]
            .as_str()
            .ok_or("missing obligation ID")?;
        all_passed &= passed;
        events.push(json!({
            "obligation_id": obligation,
            "public_api_symbol": symbol,
            "status": if passed { "passed" } else { "failed" },
            "assertion": assertion,
        }));
    }

    if events.is_empty() {
        return Err("no supported Rust local obligations in the plan".into());
    }
    let mut output = BufWriter::new(File::create(args.evidence)?);
    serde_json::to_writer(
        &mut output,
        &json!({
            "schema_version": "1.0",
            "run_id": uuid::Uuid::new_v4().to_string(),
            "scope": "local",
            "language": "rust",
            "library_commit": env!("KMIPKIT_LIBRARY_COMMIT"),
            "artifact_sha256": env!("KMIPKIT_ARTIFACT_SHA256"),
            "matrix_sha256": args.matrix_sha256,
            "events": events,
        }),
    )?;
    output.flush()?;
    Ok(all_passed)
}

fn main() -> ExitCode {
    let mode = std::env::args().nth(1);
    if matches!(
        mode.as_deref(),
        Some("--inspect-oasis-ttlv" | "--construct-oasis-ttlv")
    ) {
        let construct = mode.as_deref() == Some("--construct-oasis-ttlv");
        return if inspect_ttlv(construct).is_ok() {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        };
    }
    if mode.as_deref() == Some("--inspect-table-version") {
        return if inspect_version().is_ok() {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        };
    }
    match run(Args::parse()) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(_) => {
            eprintln!("Rust draft campaign failed. Check the plan and evidence paths.");
            ExitCode::from(2)
        }
    }
}

fn inspect_version() -> Result<(), Box<dyn std::error::Error>> {
    let mut output = BufWriter::new(std::io::stdout().lock());
    writeln!(
        output,
        "{}",
        json!({
            "library_commit": env!("KMIPKIT_LIBRARY_COMMIT"),
            "artifact_sha256": env!("KMIPKIT_ARTIFACT_SHA256"),
        })
    )?;
    for line in std::io::stdin().lock().lines() {
        let input = line?;
        let (kind, raw) = input.split_once(' ').ok_or("missing table version input")?;
        let code = raw.parse()?;
        for version in [ProtocolVersion::V2_0, ProtocolVersion::V2_1] {
            let assigned = match kind {
                "tag" => VersionGate::validate_tag(version, code).is_ok(),
                "key-format" => key_format_type::name(code).is_some(),
                "object-type" => object_type::name(code).is_some(),
                _ => return Err("unsupported table version input".into()),
            };
            writeln!(output, "{assigned}")?;
        }
    }
    output.flush()?;
    Ok(())
}

fn inspect_ttlv(construct: bool) -> Result<(), Box<dyn std::error::Error>> {
    let mut output = BufWriter::new(std::io::stdout().lock());
    writeln!(
        output,
        "{}",
        json!({
            "library_commit": env!("KMIPKIT_LIBRARY_COMMIT"),
            "artifact_sha256": env!("KMIPKIT_ARTIFACT_SHA256"),
        })
    )?;
    for line in std::io::stdin().lock().lines() {
        let line = line?;
        if line.len() % 2 != 0 {
            return Err("fixture hex length is not even".into());
        }
        let wire = (0..line.len())
            .step_by(2)
            .map(|index| u8::from_str_radix(line.get(index..index + 2).unwrap_or(""), 16))
            .collect::<Result<Vec<_>, _>>()?;
        let item = if construct {
            construct_item(&wire, 0)
                .filter(|(_, remaining)| remaining.is_empty())
                .map(|(item, _)| item)
                .filter(|item| {
                    let mut encoded = Vec::new();
                    generic_wire::encode(item, &mut encoded).is_ok() && encoded == wire
                })
        } else {
            kmipkit::ttlv::codec::decode(&wire).ok()
        };
        let result = if let Some(item) = item {
            let mut nodes = Vec::new();
            append_node(&item, &mut nodes)?;
            json!(nodes)
        } else {
            Value::Null
        };
        writeln!(output, "{result}")?;
    }
    output.flush()?;
    Ok(())
}

fn construct_item(wire: &[u8], depth: u8) -> Option<(Item, &[u8])> {
    if depth >= 32 {
        return None;
    }
    let (header, rest) = wire.split_at_checked(8)?;
    let tag = RawTag::new(u32::from_be_bytes([0, header[0], header[1], header[2]]))
        .ok()?
        .try_checked()
        .ok()?;
    let len = usize::try_from(u32::from_be_bytes(header[4..].try_into().ok()?)).ok()?;
    let (padded, remaining) = rest.split_at_checked(len.checked_next_multiple_of(8)?)?;
    let (body, padding) = padded.split_at_checked(len)?;
    if padding.iter().any(|byte| *byte != 0) {
        return None;
    }
    let value = match header[3] {
        1 => {
            let mut fields = Structure::new();
            let mut children = body;
            while !children.is_empty() {
                let (field, remaining) = construct_item(children, depth + 1)?;
                fields.try_push(field).ok()?;
                children = remaining;
            }
            TtlvValue::structure(fields)
        }
        2 => TtlvValue::integer(i32::from_be_bytes(body.try_into().ok()?)),
        3 => TtlvValue::long_integer(i64::from_be_bytes(body.try_into().ok()?)),
        4 if !body.is_empty() && len % 8 == 0 => TtlvValue::big_integer(body.to_vec()),
        5 => TtlvValue::enumeration(u32::from_be_bytes(body.try_into().ok()?)),
        6 => TtlvValue::boolean(match u64::from_be_bytes(body.try_into().ok()?) {
            0 => false,
            1 => true,
            _ => return None,
        }),
        7 => TtlvValue::text_string(std::str::from_utf8(body).ok()?.to_owned()),
        8 => TtlvValue::byte_string(body.to_vec()),
        9 => TtlvValue::date_time(i64::from_be_bytes(body.try_into().ok()?)),
        10 => TtlvValue::interval(u32::from_be_bytes(body.try_into().ok()?)),
        11 => TtlvValue::date_time_extended(i64::from_be_bytes(body.try_into().ok()?)),
        _ => return None,
    };
    Some((Item::new(tag, value).ok()?, remaining))
}

fn append_node(item: &Item, nodes: &mut Vec<Value>) -> Result<(), &'static str> {
    let kind = TTLV_TYPES
        .iter()
        .position(|kind| *kind == item.item_type())
        .ok_or("unknown decoded item type")?
        + 1;
    item.with_value(|value| {
        if let ValueView::Structure(structure) = value {
            nodes.push(json!([item.tag().raw(), kind, structure.children().len()]));
            for child in structure.children() {
                append_node(child, nodes)?;
            }
        } else {
            let scalar = match value {
                ValueView::Integer(value) => json!(value),
                ValueView::LongInteger(value)
                | ValueView::DateTime(value)
                | ValueView::DateTimeExtended(value) => json!(value),
                ValueView::Enumeration(value) | ValueView::Interval(value) => json!(value),
                ValueView::Boolean(value) => json!(value),
                ValueView::BigInteger(value) | ValueView::ByteString(value) => json!(hex(value)),
                ValueView::TextString(value)
                | ValueView::Identifier(value)
                | ValueView::Reference(value)
                | ValueView::NameReference(value) => json!(hex(value.as_bytes())),
                _ => return Err("unknown scalar type"),
            };
            nodes.push(json!([item.tag().raw(), kind, 0, scalar]));
        }
        Ok(())
    })
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut result = String::with_capacity(2 * bytes.len());
    for byte in bytes {
        result.push(char::from(DIGITS[usize::from(byte >> 4)]));
        result.push(char::from(DIGITS[usize::from(byte & 15)]));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::check;

    #[test]
    fn checks_each_draft_constructor_through_the_installed_crate() {
        for operation in [
            "create_group",
            "obliterate",
            "create_user",
            "create_credential",
            "deactivate",
            "encapsulate",
            "decapsulate",
        ] {
            let symbol = format!("kmipkit_client::ClientBatchItem::{operation}_draft");
            assert_eq!(check(&symbol), Some(true), "{symbol}");
        }
        assert_eq!(check("kmipkit_client::ClientBatchItem::unknown"), None);
    }

    #[test]
    fn installed_fluent_builders_produce_all_seven_typed_operations() {
        use kmipkit::Client;
        use kmipkit::client_config::{ClientConfiguration, client_extension_registry};
        use kmipkit::protocol::{
            AttributeSet, CreateCredentialObject, PasswordCredentialObject,
            PasswordCredentialObjectFields, SecretText, UniqueIdentifier,
        };
        use kmipkit_transport::{
            CertificateInput, ClientIdentity, Endpoint, PrivateKeyInput, TransportConfig,
            TrustSource,
        };

        let registry =
            client_extension_registry(Vec::new(), kmipkit::protocol::extension::defaults())
                .expect("empty extension registry");
        let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../bindings/shared/fixtures/login-mtls");
        let transport = TransportConfig::builder(Endpoint::raw_tls("127.0.0.1", 1))
            .client_identity(ClientIdentity::new(
                CertificateInput::from_pem_file(fixture.join("client-cert.pem"))
                    .expect("test certificate"),
                PrivateKeyInput::from_pem_file(fixture.join("client-key.pem"))
                    .expect("test private key"),
            ))
            .trust_source(TrustSource::certificate_authorities(vec![
                CertificateInput::from_pem_file(fixture.join("server-cert.pem"))
                    .expect("test trust anchor"),
            ]))
            .build()
            .expect("valid endpoint configuration");
        let mut client = Client::new(ClientConfiguration::new(registry), transport)
            .expect("client construction without connecting");
        let identifier = || UniqueIdentifier::TextString("kmip30-fixture".into());
        let password = PasswordCredentialObject::try_new(PasswordCredentialObjectFields {
            password: Some(SecretText::new("disposable-fixture".into())),
            ..PasswordCredentialObjectFields::default()
        })
        .expect("valid password fixture");
        let cases = [
            (
                client
                    .request()
                    .create_group(AttributeSet::new())
                    .to_batch_item(),
                0x3c,
            ),
            (
                client.request().obliterate(identifier()).to_batch_item(),
                0x3d,
            ),
            (
                client
                    .request()
                    .create_user(AttributeSet::new())
                    .to_batch_item(),
                0x3e,
            ),
            (
                client
                    .request()
                    .create_credential(
                        AttributeSet::new(),
                        CreateCredentialObject::PasswordCredential(password),
                    )
                    .to_batch_item(),
                0x3f,
            ),
            (
                client.request().deactivate(identifier()).to_batch_item(),
                0x40,
            ),
            (
                client.request().encapsulate(identifier()).to_batch_item(),
                0x41,
            ),
            (
                client
                    .request()
                    .decapsulate(identifier(), vec![1, 2, 3])
                    .to_batch_item(),
                0x42,
            ),
        ];
        for (item, code) in cases {
            assert_eq!(item.operation(), code);
        }
    }
}
