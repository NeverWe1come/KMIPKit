//! Read-only KMIP 2.x exchanges constructed using public raw-tag models.
use crate::driver::{item, structure, transport_config};
use crate::models::{Case, CaseResult, Connection, Result};
use kmipkit::RequestOptions;
use kmipkit::generic_ttlv::with_encoded_generic_ttlv;
use kmipkit::protocol::{
    DiscoverVersionsResponse, PingResponse, ProtocolVersion, QueryResponse, ResponseMessage,
};
use kmipkit::ttlv::{Item, Value, codec};
use kmipkit_transport::RawTlsTransport;

pub(crate) fn supports(case: &Case, connection: &Connection) -> bool {
    operation(case, connection).is_some()
        && connection.transport == "raw_tls"
        && connection.encoding == "TTLV"
        && connection.authentication.is_none()
        && case.execution.mutation == "read_only"
}

fn operation(case: &Case, connection: &Connection) -> Option<u32> {
    match (case.case_id.as_str(), connection.protocol_version.as_str()) {
        ("test-op-query", "2.0" | "2.1") => Some(0x18),
        ("test-op-discover-versions", "2.0" | "2.1") => Some(0x1e),
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
        0x18 => vec![item(0x0042_0074, Value::enumeration(1))?],
        0x1e => vec![version_item()?],
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
    let response = with_encoded_generic_ttlv(&request, &limits, |wire| {
        transport.exchange_with_options(
            wire,
            limits.max_message_bytes(),
            &RequestOptions::default(),
        )
    })
    .map_err(|_| "generic request failed TTLV encoding")?
    .map_err(|_| "generic TTLV exchange failed")?;
    let root =
        codec::decode(response.as_bytes()).map_err(|_| "generic response failed TTLV decoding")?;
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
        let matches = if operation == 0x18 {
            let query = QueryResponse::try_from_response_item(batch)
                .map_err(|_| "invalid Query response")?;
            query.operations().next().is_some()
        } else if operation == 0x1e {
            let discover =
                DiscoverVersionsResponse::try_from_response_item_for_offer(batch, &[version])
                    .map_err(|_| "invalid Discover Versions response")?;
            discover.supported_versions() == Some(&[version])
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
