//! Handlers for custom `kmip/*` JSON-RPC methods over the LSP connection.

use serde::Deserialize;
use serde_json::{Value, json};

use crate::client::live::{ExecuteRequestParams, execute_live};
use crate::sdk::analyzer::{TargetLanguage, generate_test_from_ttlv};
use crate::spec::catalog;
use crate::ttlv::inspector::{DecodeFormat, DecodeOptions, EncodeNodeInput, decode_ttlv, encode_ttlv};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DecodeTtlvParams {
    input: String,
    #[serde(default)]
    format: DecodeFormat,
    #[serde(default)]
    reveal_offsets: Vec<usize>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EncodeTtlvParams {
    node: EncodeNodeInput,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LookupSpecParams {
    query: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GenerateTestParams {
    input: String,
    language: TargetLanguage,
}

/// Dispatches a `kmip/*` method. Returns `Ok(result)` or `Err((code, message))`.
pub fn handle_kmip_method(method: &str, params: Option<Value>) -> Result<Value, (i32, String)> {
    match method {
        "kmip/decodeTtlv" => {
            let parsed: DecodeTtlvParams = parse_params(params)?;
            let options = DecodeOptions {
                format: parsed.format,
                reveal_all: false,
                reveal_offsets: parsed.reveal_offsets,
            };
            let decoded = decode_ttlv(&parsed.input, &options).map_err(|e| (-32001, e))?;
            serde_json::to_value(decoded).map_err(|e| (-32603, e.to_string()))
        }
        "kmip/encodeTtlv" => {
            let parsed: EncodeTtlvParams = parse_params(params)?;
            let encoded = encode_ttlv(&parsed.node).map_err(|e| (-32002, e))?;
            serde_json::to_value(encoded).map_err(|e| (-32603, e.to_string()))
        }
        "kmip/lookupSpec" => {
            let parsed: LookupSpecParams = parse_params(params)?;
            let entries = catalog::lookup(&parsed.query);
            serde_json::to_value(entries).map_err(|e| (-32603, e.to_string()))
        }
        "kmip/discoverVersions" => {
            let mut parsed: ExecuteRequestParams = parse_params(params)?;
            parsed.operation = "DiscoverVersions".to_string();
            let resp = execute_live(&parsed).map_err(|e| (-32010, e))?;
            serde_json::to_value(resp).map_err(|e| (-32603, e.to_string()))
        }
        "kmip/executeRequest" => {
            let parsed: ExecuteRequestParams = parse_params(params)?;
            let resp = execute_live(&parsed).map_err(|e| (-32010, e))?;
            serde_json::to_value(resp).map_err(|e| (-32603, e.to_string()))
        }
        "kmip/generateTest" => {
            let parsed: GenerateTestParams = parse_params(params)?;
            let generated =
                generate_test_from_ttlv(&parsed.input, parsed.language).map_err(|e| (-32011, e))?;
            serde_json::to_value(generated).map_err(|e| (-32603, e.to_string()))
        }
        _ => Err((-32601, format!("Method not found: {method}"))),
    }
}

fn parse_params<T: for<'de> Deserialize<'de>>(params: Option<Value>) -> Result<T, (i32, String)> {
    let val = params.unwrap_or_else(|| json!({}));
    serde_json::from_value(val).map_err(|e| (-32602, format!("Invalid params: {e}")))
}
