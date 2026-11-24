//! Read-only KMIP 2.x exchanges constructed using public raw-tag models.
use crate::driver::{item, structure, transport_config};
use crate::models::{Case, CaseResult, Connection, Result};
use kmipkit::RequestOptions;
use kmipkit::protocol::{
    DiscoverVersionsResponse, HashResponse, LocateResponse, PingResponse, ProtocolVersion,
    QueryResponse, ResponseMessage, RngRetrieveResponse,
};
use kmipkit::ttlv::{Item, Value, ValueView, codec};
use kmipkit_transport::RawTlsTransport;
use sha2::{Digest, Sha256};

const HASH_INPUT: &[u8] = b"abc";

pub(crate) fn supports(case: &Case, connection: &Connection) -> bool {
    operation(case, connection).is_some()
        && connection.transport == "raw_tls"
        && connection.encoding == "TTLV"
        && connection.authentication.is_none()
        && (case.execution.mutation == "read_only" || operation(case, connection) == Some(0x25))
}

fn operation(case: &Case, connection: &Connection) -> Option<u32> {
    match (case.case_id.as_str(), connection.protocol_version.as_str()) {
        ("test-op-locate", "2.0" | "2.1") => Some(0x08),
        ("test-op-query", "2.0" | "2.1") => Some(0x18),
        ("test-op-discover-versions", "2.0" | "2.1") => Some(0x1e),
        ("test-crypto-rng-retrieve-rng-32", "2.0" | "2.1") => Some(0x25),
        ("test-op-hash" | "test-crypto-hash-sha_256-0", "2.0" | "2.1") => Some(0x27),
        ("test-op-ping", "2.1") => Some(0x3b),
        _ => None,
    }
}

pub(crate) fn run(case: &Case, connection: &Connection) -> CaseResult {
    let start = std::time::Instant::now();
    let mut result = match exchange(case, connection) {
        Ok((status, reason)) => {
            let mut result = crate::runner::observed_result(case, status, reason);
            "generic TTLV request and response validated".clone_into(&mut result.diagnostic);
            result
        }
        Err(problem) => CaseResult::non_run(case, "failed", problem.to_owned()),
    };
    result.duration_ms = start.elapsed().as_secs_f64() * 1000.0;
    result
}

fn exchange(case: &Case, connection: &Connection) -> Result<(u32, Option<u32>)> {
    let minor = i32::from(connection.protocol_version == "2.1");
    let version = ProtocolVersion::from_raw(2, minor);
    let operation = operation(case, connection).ok_or("no generic exchange recipe")?;
    let version_item = || {
        node(
            0x0042_0069,
            vec![
                item(0x0042_006a, Value::integer(2))?,
                item(0x0042_006b, Value::integer(minor))?,
            ],
        )
    };
    let payload_fields = match operation {
        0x08 => vec![
            item(0x0042_004f, Value::integer(1))?,
            item(0x0042_00d4, Value::integer(0))?,
        ],
        0x18 => vec![item(0x0042_0074, Value::enumeration(1))?],
        0x1e => vec![version_item()?],
        0x25 => vec![item(0x0042_00c4, Value::integer(32))?],
        0x27 => vec![
            node(0x0042_002b, vec![item(0x0042_0038, Value::enumeration(6))?])?,
            item(0x0042_00c2, Value::byte_string(HASH_INPUT.to_vec()))?,
        ],
        _ => Vec::new(),
    };
    let request = node(
        0x0042_0078,
        vec![
            node(
                0x0042_0077,
                vec![version_item()?, item(0x0042_000d, Value::integer(1))?],
            )?,
            node(
                0x0042_000f,
                vec![
                    item(0x0042_005c, Value::enumeration(operation))?,
                    node(0x0042_0079, payload_fields)?,
                ],
            )?,
        ],
    )?;
    let mut transport = RawTlsTransport::new(transport_config(connection)?)
        .map_err(|_| "generic TLS configuration rejected")?;
    let limits = codec::CodecLimits::defaults();
    let mut wire = Vec::new();
    encode(&request, &mut wire)?;
    let response = transport
        .exchange_with_options(
            &wire,
            limits.max_message_bytes(),
            &RequestOptions::default(),
        )
        .map_err(|_| "generic TTLV exchange failed")?;
    check_response(response.as_bytes(), operation, version)
}

fn check_response(
    wire: &[u8],
    operation: u32,
    version: ProtocolVersion,
) -> Result<(u32, Option<u32>)> {
    let root = codec::decode(wire).map_err(|_| "generic response failed TTLV decoding")?;
    if root.tag().raw() != 0x0042_007b {
        return Err("generic response has the wrong root tag");
    }
    let tree = root
        .as_structure()
        .ok_or("generic response needs a structure")?;
    let response =
        ResponseMessage::try_from_ttlv(tree.try_clone().map_err(|_| "response copy failed")?)
            .map_err(|_| "generic response failed message validation")?;
    let mut items = response.batch_items();
    if response.header().protocol_version() != version || items.len() != 1 {
        return Err("response version or batch count differs from the request");
    }
    let batch = items
        .next()
        .filter(|item| item.operation() == Some(operation))
        .ok_or("response operation differs from the request")?;
    let status = batch
        .result_status()
        .ok_or("response status missing")?
        .raw();
    if status == 0 {
        let matches = if operation == 0x08 {
            let locate = LocateResponse::try_from_response_item(batch)
                .map_err(|_| "invalid Locate response")?;
            let returned = locate.unique_identifiers().len();
            returned <= 1
                && locate
                    .located_items()
                    .is_none_or(|total| usize::try_from(total).is_ok_and(|total| total >= returned))
        } else if operation == 0x18 {
            let query = QueryResponse::try_from_response_item(batch)
                .map_err(|_| "invalid Query response")?;
            query.operations().next().is_some()
        } else if operation == 0x1e {
            let discover =
                DiscoverVersionsResponse::try_from_response_item_for_offer(batch, &[version])
                    .map_err(|_| "invalid Discover Versions response")?;
            discover.supported_versions() == Some(&[version])
        } else if operation == 0x25 {
            RngRetrieveResponse::try_from_response_item(batch)
                .map_err(|_| "invalid RNG Retrieve response")?
                .data()
                .is_some_and(|data| data.with_bytes(|bytes| bytes.len() == 32))
        } else if operation == 0x27 {
            let hash =
                HashResponse::try_from_response_item(batch).map_err(|_| "invalid Hash response")?;
            let expected = Sha256::digest(HASH_INPUT);
            hash.correlation_value().is_none()
                && hash
                    .data()
                    .is_some_and(|data| data.with_bytes(|bytes| bytes == &expected[..]))
        } else {
            PingResponse::try_from_response_item(batch).is_ok()
        };
        if !matches {
            return Err("response did not confirm the requested capability");
        }
    }
    Ok((
        status,
        batch.result_reason().map(kmipkit::ResultReason::raw),
    ))
}

fn node(tag: u32, children: Vec<Item>) -> Result<Item> {
    item(tag, Value::structure(structure(tag, children)?))
}

// Encode the native generic tree for the read-only probes.
fn encode(item: &Item, output: &mut Vec<u8>) -> Result<()> {
    let start = output.len();
    output.extend_from_slice(&item.tag().raw().to_be_bytes()[1..]);
    output.extend_from_slice(&[0; 5]);
    let kind = item.with_value(|value| -> Result<u8> {
        Ok(match value {
            ValueView::Structure(value) => {
                for field in value.children() {
                    encode(field, output)?;
                }
                1
            }
            ValueView::Integer(value) => {
                output.extend_from_slice(&value.to_be_bytes());
                2
            }
            ValueView::Enumeration(value) => {
                output.extend_from_slice(&value.to_be_bytes());
                5
            }
            ValueView::ByteString(value) => {
                output.extend_from_slice(value);
                8
            }
            _ => return Err("fixture contains an unsupported request value"),
        })
    })?;
    let length = u32::try_from(output.len() - start - 8).map_err(|_| "fixture too large")?;
    output[start + 3] = kind;
    output[start + 4..start + 8].copy_from_slice(&length.to_be_bytes());
    output.resize(output.len().next_multiple_of(8), 0);
    Ok(())
}
