//! Live mTLS KMIP execution using `kmipkit-client` with DEV/STAGING/PROD guardrails.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default, Eq, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum Environment {
    #[default]
    Dev,
    Staging,
    Prod,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileTlsInput {
    pub ca_cert_path: Option<String>,
    pub client_cert_path: Option<String>,
    pub client_key_path: Option<String>,
    pub server_name: Option<String>,
    #[serde(default)]
    pub allow_tls12: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveProfile {
    pub name: String,
    #[serde(default)]
    pub environment: Environment,
    pub host: String,
    pub port: u16,
    #[serde(default)]
    pub read_only: bool,
    pub tls: Option<ProfileTlsInput>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteRequestParams {
    pub profile: LiveProfile,
    pub operation: String,
    pub unique_identifier: Option<String>,
    #[serde(default)]
    pub unlock_prod: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteResponse {
    pub operation: String,
    pub environment: Environment,
    pub latency_ms: u64,
    pub result_status: String,
    pub summary: String,
    pub supported_versions: Vec<String>,
    pub raw_response_hex: Option<String>,
}

/// Returns `true` if the operation mutates or destroys key material/state.
#[must_use]
pub fn is_mutating_operation(op: &str) -> bool {
    matches!(
        op.to_ascii_lowercase().as_str(),
        "create"
            | "createkeypair"
            | "register"
            | "rekey"
            | "rekeykeypair"
            | "activate"
            | "revoke"
            | "destroy"
            | "archive"
            | "recover"
            | "deleteattribute"
            | "addattribute"
            | "modifyattribute"
            | "setattribute"
            | "obliterate"
    )
}

/// Enforces mandatory Phase 2/Security guardrails before touching network transport.
///
/// # Errors
///
/// Returns a descriptive guardrail error if `readOnly` or `PROD` protection blocks the operation.
pub fn check_guardrails(profile: &LiveProfile, operation: &str, unlock_prod: bool) -> Result<(), String> {
    if profile.read_only && is_mutating_operation(operation) {
        return Err(format!(
            "GUARDRAIL_READ_ONLY: Profile '{}' is in read-only mode; operation '{}' is blocked.",
            profile.name, operation
        ));
    }
    if profile.environment == Environment::Prod && is_mutating_operation(operation) && !unlock_prod {
        return Err(format!(
            "GUARDRAIL_PROD_LOCKED: Operation '{}' on PROD profile '{}' requires explicit unlockProd confirmation.",
            operation, profile.name
        ));
    }
    Ok(())
}

#[cfg(feature = "live-client")]
pub fn execute_live(params: &ExecuteRequestParams) -> Result<ExecuteResponse, String> {
    use std::path::Path;
    use std::time::Instant;

    use kmipkit_client::extension_registry::{ClientConfiguration, client_extension_registry};
    use kmipkit_client::{Client, ClientBatch};
    use kmipkit_transport::{
        CertificateInput, ClientIdentity, Endpoint, PrivateKeyInput, TlsVersionPolicy,
        TransportConfig, TrustSource,
    };
    use kmipkit_ttlv::codec::CodecLimits;

    check_guardrails(&params.profile, &params.operation, params.unlock_prod)?;

    let tls = params
        .profile
        .tls
        .as_ref()
        .ok_or_else(|| "TLS configuration (clientCertPath, clientKeyPath) is required for live mTLS".to_string())?;

    let client_cert = tls
        .client_cert_path
        .as_deref()
        .ok_or_else(|| "tls.clientCertPath is required".to_string())?;
    let client_key = tls
        .client_key_path
        .as_deref()
        .ok_or_else(|| "tls.clientKeyPath is required".to_string())?;

    let cert_input = CertificateInput::from_pem_file(Path::new(client_cert))
        .map_err(|e| format!("Invalid client certificate: {e}"))?;
    let key_input = PrivateKeyInput::from_pem_file(Path::new(client_key))
        .map_err(|e| format!("Invalid client private key: {e}"))?;
    let identity = ClientIdentity::new(cert_input, key_input);

    let trust = if let Some(ca_path) = tls.ca_cert_path.as_deref() {
        let ca_cert = CertificateInput::from_pem_file(Path::new(ca_path))
            .map_err(|e| format!("Invalid CA certificate: {e}"))?;
        TrustSource::certificate_authorities(vec![ca_cert])
    } else {
        TrustSource::platform()
    };

    let endpoint = Endpoint::raw_tls(&params.profile.host, params.profile.port);
    let mut builder = TransportConfig::builder(endpoint)
        .client_identity(identity)
        .trust_source(trust);
    if tls.allow_tls12 {
        builder = builder.tls_version_policy(TlsVersionPolicy::Tls12And13Compat);
    }
    if let Some(ref sni) = tls.server_name {
        builder = builder.tls_server_name(sni.clone());
    }

    let transport_cfg = builder
        .build()
        .map_err(|e| format!("Invalid transport configuration: {e}"))?;

    let registry = client_extension_registry(Vec::new(), kmipkit_protocol::extension::defaults())
        .map_err(|e| format!("Extension registry error: {e}"))?;
    let client_cfg = ClientConfiguration::new(registry);

    let mut client = Client::new(client_cfg, transport_cfg)
        .map_err(|e| format!("Client initialization error: {e}"))?;

    let batch_item = build_batch_item(&params.operation, params.unique_identifier.as_deref())?;
    let batch = ClientBatch::new(batch_item);

    let started = Instant::now();
    let outcome = client
        .execute(batch, &CodecLimits::defaults())
        .map_err(|e| format!("KMIP execution failed: {e}"))?;
    let latency_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);

    let mut supported_versions = Vec::new();
    let result_status = "Success".to_string();
    let mut summary = format!("Executed {} in {} ms", params.operation, latency_ms);
    let mut raw_response_hex = None;

    if let Some(item_resp) = outcome.get(0) {
        raw_response_hex = Some(item_resp.with_raw_response_bytes(|b| hex::encode(b)));
        if let Some(versions) = item_resp.discover_versions_supported_versions() {
            supported_versions = versions
                .iter()
                .map(|v| format!("{}.{}", v.major(), v.minor()))
                .collect();
            summary = format!("Server supports KMIP versions: {}", supported_versions.join(", "));
        }
    }

    Ok(ExecuteResponse {
        operation: params.operation.clone(),
        environment: params.profile.environment,
        latency_ms,
        result_status,
        summary,
        supported_versions,
        raw_response_hex,
    })
}

#[cfg(feature = "live-client")]
fn build_batch_item(
    operation: &str,
    unique_id: Option<&str>,
) -> Result<kmipkit_client::ClientBatchItem, String> {
    use kmipkit_client::{ClientBatchItem, ClientRequest};
    use kmipkit_protocol::{
        AttributeReference, AttributeSet, DestroyRequest, DiscoverVersionsRequest,
        GetAttributesRequest, GetRequest, LocateRequest, QueryFunction, QueryRequest,
        UniqueIdentifier,
    };

    match operation.to_ascii_lowercase().as_str() {
        "discoverversions" => Ok(ClientBatchItem::new(ClientRequest::DiscoverVersions(
            DiscoverVersionsRequest::new(),
        ))),
        "ping" => Ok(ClientBatchItem::ping()),
        "query" => Ok(ClientBatchItem::query(QueryRequest::new(vec![
            QueryFunction::OPERATIONS,
            QueryFunction::OBJECTS,
        ]))),
        "locate" => Ok(ClientBatchItem::new(ClientRequest::Locate(
            LocateRequest::new(AttributeSet::new()),
        ))),
        "get" => {
            let uid = unique_id.ok_or_else(|| "uniqueIdentifier is required for Get".to_string())?;
            Ok(ClientBatchItem::new(ClientRequest::Get(
                GetRequest::new().with_unique_identifier(UniqueIdentifier::TextString(uid.to_string())),
            )))
        }
        "getattributes" => {
            let uid = unique_id
                .ok_or_else(|| "uniqueIdentifier is required for GetAttributes".to_string())?;
            let req = GetAttributesRequest::try_new(
                Some(uid.to_string()),
                Vec::<AttributeReference>::new(),
            )
            .map_err(|e| format!("{e}"))?;
            Ok(ClientBatchItem::new(ClientRequest::GetAttributes(req)))
        }
        "destroy" => {
            let uid =
                unique_id.ok_or_else(|| "uniqueIdentifier is required for Destroy".to_string())?;
            Ok(ClientBatchItem::new(ClientRequest::Destroy(
                DestroyRequest::new(Some(UniqueIdentifier::TextString(uid.to_string()))),
            )))
        }
        other => Err(format!("Unsupported Playground operation '{other}'")),
    }
}

#[cfg(not(feature = "live-client"))]
pub fn execute_live(params: &ExecuteRequestParams) -> Result<ExecuteResponse, String> {
    check_guardrails(&params.profile, &params.operation, params.unlock_prod)?;
    Err("FEATURE_NOT_ENABLED: Compile kmip-lsp with --features live-client for live mTLS execution.".to_string())
}
