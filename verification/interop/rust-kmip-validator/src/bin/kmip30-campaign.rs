//! Collect local evidence from the installed KMIP 3.0 draft public API.
#![forbid(unsafe_code)]

use clap::{Parser, ValueEnum};
use kmipkit::ttlv::{Item, RawTag, Structure, Value as TtlvValue, ValueView};
use kmipkit::{ClientBatchItem, ClientRequest};
use serde_json::{Value, json};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::process::ExitCode;

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

fn run(args: Args) -> Result<bool, Box<dyn std::error::Error>> {
    let mut events = Vec::new();
    let mut all_passed = true;
    for line in BufReader::new(File::open(args.plan)?).lines() {
        let row: Value = serde_json::from_str(&line?)?;
        if row["language"] != "rust"
            || row["protocol_version"] != "3.0"
            || row["kind"] != "public_route"
            || row["action"] != "construct"
            || row["execution_status"] != "required"
        {
            continue;
        }
        let symbol = row["public_api_symbol"]
            .as_str()
            .ok_or("missing API symbol")?;
        let Some(passed) = check(symbol) else {
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
            "assertion": "draft_operation_and_ordered_payload",
        }));
    }

    if events.is_empty() {
        return Err("no supported Rust draft constructor obligations in the plan".into());
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
    match run(Args::parse()) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(_) => {
            eprintln!("Rust draft campaign failed. Check the plan and evidence paths.");
            ExitCode::from(2)
        }
    }
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
