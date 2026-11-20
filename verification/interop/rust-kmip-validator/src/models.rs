//! Shared contract data. Sensitive connection data deliberately has no Debug implementation.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::Path;

pub(crate) const PIN: &str = "9692057c9ade474fb0c4e0b7130539da368eb794";
pub(crate) type Result<T> = std::result::Result<T, &'static str>;

pub(crate) fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub(crate) fn load<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let bytes = std::fs::read(path).map_err(|_| "input file unavailable")?;
    serde_json::from_slice(&bytes).map_err(|_| "input does not match the contract")
}

#[derive(Deserialize)]
pub(crate) struct Connection {
    pub server_id: String,
    pub protocol_version: String,
    pub transport: String,
    pub encoding: String,
    pub api_level: String,
    pub endpoints: Endpoints,
    pub tls: Option<Tls>,
    pub authentication: Option<Authentication>,
    pub environment: Environment,
    pub execution_policy: Policy,
}
#[derive(Deserialize)]
pub(crate) struct Endpoints {
    pub raw_tls: Option<RawTls>,
    pub https: Option<Https>,
}
#[derive(Deserialize)]
pub(crate) struct RawTls {
    pub host: String,
    pub port: u16,
}
#[derive(Deserialize)]
pub(crate) struct Https {
    pub url: String,
}
#[derive(Deserialize)]
pub(crate) struct Tls {
    pub ca_cert_path: String,
    pub client_cert_path: String,
    pub client_key_path: String,
    pub server_name_override: Option<String>,
    pub connect_timeout_seconds: Option<u64>,
    pub read_timeout_seconds: Option<u64>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Authentication {
    pub username_password: Option<EnvCredentials>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EnvCredentials {
    pub username_env: String,
    pub password_env: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Environment {
    pub kind: String,
    pub dedicated_test_environment_confirmed: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Policy {
    pub probe_unsupported: bool,
    pub allow_mutations: bool,
    pub encodings: Option<Vec<String>>,
    pub api_levels: Option<Vec<String>>,
    pub cleanup_on_failure: Option<bool>,
}
impl Connection {
    pub fn validate(&self) -> Result<()> {
        if self.server_id.is_empty()
            || !self
                .server_id
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
        {
            return Err("invalid server identity");
        }
        if !["2.0", "2.1"].contains(&self.protocol_version.as_str())
            || !["TTLV", "JSON", "XML"].contains(&self.encoding.as_str())
            || !["raw_tls", "https"].contains(&self.transport.as_str())
            || !["high-level", "typed-protocol", "generic-ttlv"].contains(&self.api_level.as_str())
        {
            return Err("unsupported execution selection");
        }
        if self.environment.kind != "dedicated_test"
            || !self.environment.dedicated_test_environment_confirmed
        {
            return Err("dedicated test environment confirmation required");
        }
        // Profiles are hints: this flag never suppresses an applicable probe.
        let _ = self.execution_policy.probe_unsupported;
        let _ = self.execution_policy.cleanup_on_failure;
        if self
            .execution_policy
            .encodings
            .as_ref()
            .is_some_and(|values| {
                !values
                    .iter()
                    .all(|v| ["TTLV", "JSON", "XML"].contains(&v.as_str()))
            })
            || self
                .execution_policy
                .api_levels
                .as_ref()
                .is_some_and(|values| {
                    !values.iter().all(|v| {
                        ["high-level", "typed-protocol", "generic-ttlv"].contains(&v.as_str())
                    })
                })
        {
            return Err("invalid policy execution matrix");
        }
        if let Some(credentials) = self
            .authentication
            .as_ref()
            .and_then(|a| a.username_password.as_ref())
        {
            for name in [&credentials.username_env, &credentials.password_env] {
                if name.is_empty()
                    || !name.bytes().enumerate().all(|(i, b)| {
                        b == b'_' || b.is_ascii_uppercase() || i > 0 && b.is_ascii_digit()
                    })
                {
                    return Err("invalid credential environment reference");
                }
            }
        }
        Ok(())
    }
}

#[derive(Deserialize)]
pub(crate) struct Catalog {
    pub test_cases: Vec<Case>,
}
#[derive(Deserialize)]
#[allow(clippy::struct_field_names)] // The shared contract fixes the case_id field name.
pub(crate) struct Case {
    pub case_id: String,
    pub operation: String,
    pub execution: Execution,
    pub normative_refs: Vec<Reference>,
    #[serde(default)]
    pub requires: Value,
}
#[derive(Deserialize)]
pub(crate) struct Reference {
    pub document: String,
    pub section: String,
}
#[derive(Deserialize)]
pub(crate) struct Execution {
    pub mode: String,
    pub protocol_versions: Vec<String>,
    pub mutation: String,
}

#[derive(Serialize)]
pub(crate) struct CaseResult {
    pub case_id: String,
    pub status: &'static str,
    pub library_correctness: &'static str,
    pub server_compatibility: &'static str,
    pub parity: &'static str,
    pub duration_ms: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_reason: Option<String>,
    pub diagnostic: String,
}
impl CaseResult {
    pub fn non_run(case: &Case, status: &'static str, diagnostic: String) -> Self {
        Self {
            case_id: case.case_id.clone(),
            status,
            library_correctness: "not_checked",
            server_compatibility: if status == "policy_restricted" {
                "policy_restricted"
            } else {
                "unknown"
            },
            parity: "not_compared",
            duration_ms: 0.0,
            result_status: None,
            result_reason: None,
            diagnostic,
        }
    }
}

impl Catalog {
    pub(crate) fn validate(&self) -> Result<()> {
        let mut ids = std::collections::HashSet::new();
        for case in &self.test_cases {
            if case.case_id.is_empty()
                || case.case_id.len() > 128
                || !case
                    .case_id
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"-_.#".contains(&b))
                || !ids.insert(&case.case_id)
            {
                return Err("invalid or duplicate catalog case identity");
            }
            if !["local_model", "server_exchange"].contains(&case.execution.mode.as_str())
                || !["read_only", "mutating"].contains(&case.execution.mutation.as_str())
                || case.execution.protocol_versions.is_empty()
                || !case
                    .execution
                    .protocol_versions
                    .iter()
                    .all(|v| ["2.0", "2.1"].contains(&v.as_str()))
            {
                return Err("invalid catalog execution contract");
            }
            if case.normative_refs.is_empty()
                || !case.normative_refs.iter().all(|r| {
                    [
                        "OASIS KMIP 2.0 Specification",
                        "OASIS KMIP 2.1 Specification",
                    ]
                    .contains(&r.document.as_str())
                        && !r.section.is_empty()
                        && r.section.bytes().all(|b| b.is_ascii_digit() || b == b'.')
                })
            {
                return Err("invalid normative case reference");
            }
        }
        Ok(())
    }
}
