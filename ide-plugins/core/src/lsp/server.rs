//! LSP `Content-Length` stdio server supporting standard LSP + custom `kmip/*` methods.

use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Read, Write};

use serde_json::{Value, json};

use crate::rpc::handlers::handle_kmip_method;
use crate::sdk::analyzer::analyze_source;
use crate::spec::catalog;

#[derive(Default)]
struct ServerState {
    documents: HashMap<String, String>,
    shutdown_requested: bool,
}

/// Runs the LSP + JSON-RPC server loop over `reader` and `writer` until `exit` or EOF.
///
/// # Errors
///
/// Returns an `io::Error` if writing to the output stream fails.
pub fn run_stdio_server<R: Read, W: Write>(reader: R, mut writer: W) -> io::Result<()> {
    let mut buf_reader = BufReader::new(reader);
    let mut state = ServerState::default();

    while let Some(raw_msg) = read_lsp_message(&mut buf_reader)? {
        let msg: Value = match serde_json::from_str(&raw_msg) {
            Ok(v) => v,
            Err(err) => {
                let err_resp = json!({
                    "jsonrpc": "2.0",
                    "id": Value::Null,
                    "error": { "code": -32700, "message": format!("Parse error: {err}") }
                });
                write_lsp_message(&mut writer, &err_resp)?;
                continue;
            }
        };

        let id = msg.get("id").cloned();
        let method = msg
            .get("method")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let params = msg.get("params").cloned();

        if method == "exit" {
            break;
        }

        if let Some(req_id) = id {
            let response = handle_request(&mut state, method, params, req_id);
            write_lsp_message(&mut writer, &response)?;
        } else {
            let notifications = handle_notification(&mut state, method, params);
            for notif in notifications {
                write_lsp_message(&mut writer, &notif)?;
            }
        }
    }

    Ok(())
}

fn handle_request(
    state: &mut ServerState,
    method: &str,
    params: Option<Value>,
    id: Value,
) -> Value {
    if method.starts_with("kmip/") {
        return match handle_kmip_method(method, params) {
            Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
            Err((code, message)) => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": code, "message": message }
            }),
        };
    }

    match method {
        "initialize" => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "serverInfo": {
                    "name": "kmip-lsp",
                    "version": env!("CARGO_PKG_VERSION")
                },
                "capabilities": {
                    "textDocumentSync": 1,
                    "hoverProvider": true,
                    "completionProvider": {
                        "triggerCharacters": ["\"", ":", " ", "x"]
                    },
                    "codeActionProvider": true
                }
            }
        }),
        "shutdown" => {
            state.shutdown_requested = true;
            json!({ "jsonrpc": "2.0", "id": id, "result": Value::Null })
        }
        "textDocument/hover" => {
            let result = compute_hover(state, params);
            json!({ "jsonrpc": "2.0", "id": id, "result": result })
        }
        "textDocument/completion" => {
            let result = compute_completion();
            json!({ "jsonrpc": "2.0", "id": id, "result": result })
        }
        "textDocument/codeAction" => {
            let result = compute_code_actions(state, params);
            json!({ "jsonrpc": "2.0", "id": id, "result": result })
        }
        _ => json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": -32601, "message": format!("Method not found: {method}") }
        }),
    }
}

fn handle_notification(
    state: &mut ServerState,
    method: &str,
    params: Option<Value>,
) -> Vec<Value> {
    let mut out = Vec::new();
    match method {
        "textDocument/didOpen" => {
            if let Some(p) = params
                && let (Some(uri), Some(text)) = (
                    p.pointer("/textDocument/uri").and_then(Value::as_str),
                    p.pointer("/textDocument/text").and_then(Value::as_str),
                )
            {
                state.documents.insert(uri.to_string(), text.to_string());
                out.push(build_diagnostics_notification(uri, text));
            }
        }
        "textDocument/didChange" => {
            if let Some(p) = params
                && let Some(uri) = p.pointer("/textDocument/uri").and_then(Value::as_str)
                && let Some(text) = p
                    .pointer("/contentChanges/0/text")
                    .and_then(Value::as_str)
            {
                state.documents.insert(uri.to_string(), text.to_string());
                out.push(build_diagnostics_notification(uri, text));
            }
        }
        "textDocument/didClose" => {
            if let Some(p) = params
                && let Some(uri) = p.pointer("/textDocument/uri").and_then(Value::as_str)
            {
                state.documents.remove(uri);
                out.push(json!({
                    "jsonrpc": "2.0",
                    "method": "textDocument/publishDiagnostics",
                    "params": { "uri": uri, "diagnostics": [] }
                }));
            }
        }
        _ => {}
    }
    out
}

fn compute_hover(state: &ServerState, params: Option<Value>) -> Value {
    let Some(p) = params else {
        return Value::Null;
    };
    let uri = p
        .pointer("/textDocument/uri")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let line_idx = p
        .pointer("/position/line")
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize;
    let char_idx = p
        .pointer("/position/character")
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize;

    let Some(text) = state.documents.get(uri) else {
        return Value::Null;
    };
    let Some(line) = text.lines().nth(line_idx) else {
        return Value::Null;
    };
    let token = extract_token_at(line, char_idx);
    if token.is_empty() {
        return Value::Null;
    }

    let entries = catalog::lookup(&token);
    let Some(first) = entries.first() else {
        return Value::Null;
    };

    let mut md = format!(
        "**{}** (`{}`)\n\n- **TTLV Type:** `{}`\n- **KMIP Versions:** `{}`\n\n{}",
        first.oasis_name, first.tag_hex, first.expected_type, first.versions, first.summary
    );
    if let Some(ref warn) = first.deprecation_warning {
        md.push_str(&format!("\n\n> **Warning:** {warn}"));
    }

    json!({
        "contents": {
            "kind": "markdown",
            "value": md
        }
    })
}

fn compute_completion() -> Value {
    let entries = catalog::lookup("");
    let mut items: Vec<Value> = entries
        .into_iter()
        .map(|entry| {
            json!({
                "label": entry.name,
                "kind": 14,
                "detail": format!("{} ({}) [KMIP {}]", entry.tag_hex, entry.expected_type, entry.versions),
                "documentation": entry.summary
            })
        })
        .collect();

    for op in [
        "DiscoverVersions",
        "Create",
        "CreateKeyPair",
        "Locate",
        "Get",
        "Rekey",
        "Activate",
        "Revoke",
        "Destroy",
    ] {
        items.push(json!({
            "label": op,
            "kind": 3,
            "detail": "KMIP Operation (OASIS)"
        }));
    }

    json!({ "isIncomplete": false, "items": items })
}

fn build_diagnostics_notification(uri: &str, text: &str) -> Value {
    let diagnostics: Vec<Value> = analyze_source(uri, text)
        .into_iter()
        .map(|d| {
            json!({
                "range": {
                    "start": { "line": d.line, "character": d.start_col },
                    "end": { "line": d.line, "character": d.end_col }
                },
                "severity": 2,
                "code": d.code,
                "source": "kmip-lsp",
                "message": d.message
            })
        })
        .collect();

    json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": {
            "uri": uri,
            "diagnostics": diagnostics
        }
    })
}

fn compute_code_actions(state: &ServerState, params: Option<Value>) -> Value {
    let Some(p) = params else {
        return json!([]);
    };
    let uri = p
        .pointer("/textDocument/uri")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let line_idx = p
        .pointer("/range/start/line")
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize;
    let Some(text) = state.documents.get(uri) else {
        return json!([]);
    };

    let actions: Vec<Value> = analyze_source(uri, text)
        .into_iter()
        .filter(|d| d.line == line_idx)
        .filter_map(|d| {
            let (old, new) = d.replacement?;
            Some(json!({
                "title": format!("Replace {old} with {new}"),
                "kind": "quickfix",
                "edit": {
                    "changes": {
                        uri: [{
                            "range": {
                                "start": { "line": d.line, "character": d.start_col },
                                "end": { "line": d.line, "character": d.end_col }
                            },
                            "newText": new
                        }]
                    }
                }
            }))
        })
        .collect();

    json!(actions)
}

fn extract_token_at(line: &str, char_idx: usize) -> String {
    let bytes = line.as_bytes();
    if bytes.is_empty() {
        return String::new();
    }
    let idx = char_idx.min(bytes.len().saturating_sub(1));
    let is_tok = |b: u8| b.is_ascii_alphanumeric() || b == b'_' || b == b'-';
    if !is_tok(bytes[idx]) {
        return String::new();
    }
    let mut start = idx;
    while start > 0 && is_tok(bytes[start - 1]) {
        start -= 1;
    }
    let mut end = idx;
    while end < bytes.len() && is_tok(bytes[end]) {
        end += 1;
    }
    String::from_utf8_lossy(&bytes[start..end]).to_string()
}

fn read_lsp_message<R: BufRead>(reader: &mut R) -> io::Result<Option<String>> {
    let mut content_length: Option<usize> = None;
    loop {
        let mut header_line = String::new();
        let bytes_read = reader.read_line(&mut header_line)?;
        if bytes_read == 0 {
            return Ok(None);
        }
        let trimmed = header_line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break;
        }
        if let Some(rest) = trimmed.strip_prefix("Content-Length:") {
            content_length = rest.trim().parse::<usize>().ok();
        }
    }

    let Some(len) = content_length else {
        return Ok(None);
    };

    let mut body = vec![0u8; len];
    reader.read_exact(&mut body)?;
    String::from_utf8(body)
        .map(Some)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

fn write_lsp_message<W: Write>(writer: &mut W, value: &Value) -> io::Result<()> {
    let payload = serde_json::to_string(value)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    write!(writer, "Content-Length: {}\r\n\r\n{}", payload.len(), payload)?;
    writer.flush()
}
