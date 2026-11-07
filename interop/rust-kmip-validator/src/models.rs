use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConnection {
    pub server_id: String,
    pub description: Option<String>,
    pub endpoints: Endpoints,
    pub tls: Option<TlsConfig>,
    pub execution_policy: Option<ExecutionPolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endpoints {
    pub raw_tls: Option<RawTlsEndpoint>,
    pub https: Option<HttpsEndpoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawTlsEndpoint {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpsEndpoint {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    pub ca_cert_path: Option<String>,
    pub client_cert_path: Option<String>,
    pub client_key_path: Option<String>,
    pub server_name_override: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExecutionPolicy {
    #[serde(default)]
    pub probe_unsupported: bool,
    pub encodings: Option<Vec<String>>,
    pub api_levels: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    pub server: ServerMetadata,
    pub transports: Option<TransportsSupport>,
    pub encodings: HashMap<String, EncodingSupport>,
    #[serde(default)]
    pub operations: HashMap<String, OperationSupport>,
    #[serde(default)]
    pub managed_objects: HashMap<String, ObjectSupport>,
    #[serde(default)]
    pub attributes: HashMap<String, AttributeSupport>,
}

fn default_schema_version() -> String {
    "1.0".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerMetadata {
    pub id: String,
    pub vendor: String,
    pub product: String,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportsSupport {
    pub raw_tls: Option<SupportStatus>,
    pub https: Option<SupportStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportStatus {
    pub supported: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodingSupport {
    pub supported: bool,
    #[serde(default)]
    pub transports: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OperationSupport {
    pub supported: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ObjectSupport {
    pub supported: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AttributeSupport {
    pub supported: bool,
    pub read: Option<bool>,
    pub create: Option<bool>,
    pub modify: Option<bool>,
    pub delete: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCatalog {
    pub version: String,
    pub statistics: CatalogStatistics,
    pub test_cases: Vec<TestCase>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogStatistics {
    pub operations_covered: usize,
    pub managed_objects_covered: usize,
    pub attributes_covered: usize,
    pub total_cases: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub case_id: String,
    pub category: String,
    pub operation: String,
    pub oasis_section: Option<String>,
    pub description: Option<String>,
    pub requires: CaseRequirements,
    pub isolation_recipe: IsolationRecipe,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CaseRequirements {
    pub operation: Option<String>,
    pub object_type: Option<String>,
    pub algorithm: Option<String>,
    pub key_length: Option<usize>,
    pub attribute: Option<String>,
    pub attribute_op: Option<String>,
    pub requires_async: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsolationRecipe {
    pub unique_name_prefix: String,
    #[serde(default)]
    pub setup_steps: Vec<String>,
    pub target_step: String,
    #[serde(default)]
    pub teardown_steps: Vec<String>,
    #[serde(default)]
    pub normalized_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResults {
    pub run_id: String,
    pub timestamp: String,
    pub language: String,
    pub server_id: String,
    pub summary: ResultsSummary,
    pub results: Vec<CaseResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResultsSummary {
    pub total_cases: usize,
    pub passed: usize,
    pub failed: usize,
    pub unsupported_by_server: usize,
    pub unexpected_success: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseResult {
    pub case_id: String,
    pub operation: String,
    pub status: CaseStatus,
    #[serde(default)]
    pub encodings_verified: Vec<String>,
    #[serde(default)]
    pub api_levels_verified: Vec<String>,
    pub duration_ms: f64,
    pub error_detail: Option<ErrorDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CaseStatus {
    Passed,
    Failed,
    UnsupportedSkipped,
    UnsupportedVerifiedRejected,
    UnexpectedSuccess,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub message: String,
    pub result_reason: Option<String>,
    pub encoding: Option<String>,
    pub api_level: Option<String>,
}
