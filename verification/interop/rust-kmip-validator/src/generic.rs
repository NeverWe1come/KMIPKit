//! KMIP 2.x exchanges constructed using public raw-tag models.
use crate::driver::{item, structure, transport_config};
use crate::models::{Case, CaseResult, Connection, Result};
use kmipkit::RequestOptions;
use kmipkit::protocol::{
    DiscoverVersionsResponse, HashResponse, LocateResponse, PingResponse, ProtocolVersion,
    QueryResponse, ResponseBatchItemView, ResponseMessage, RngRetrieveResponse,
};
use kmipkit::ttlv::{Item, Value, ValueView, codec};
use kmipkit_transport::RawTlsTransport;
use sha2::{Digest, Sha256};

const HASH_INPUT: &[u8] = b"abc";
const CREATE: u32 = 1;
const CHECK: u32 = 9;
const GET: u32 = 10;
const GET_ATTRIBUTES: u32 = 11;
const GET_ATTRIBUTE_LIST: u32 = 12;
const ACTIVATE: u32 = 18;
const REVOKE: u32 = 19;
const DESTROY: u32 = 20;
const ARCHIVE: u32 = 21;
const RECOVER: u32 = 22;

fn uses_owned_key(operation: u32) -> bool {
    matches!(
        operation,
        CREATE
            | CHECK
            | GET
            | GET_ATTRIBUTES
            | GET_ATTRIBUTE_LIST
            | ACTIVATE
            | REVOKE
            | DESTROY
            | ARCHIVE
            | RECOVER
    )
}

pub(crate) fn supports(case: &Case, connection: &Connection) -> bool {
    connection.transport == "raw_tls"
        && connection.encoding == "TTLV"
        && connection.authentication.is_none()
        && operation(case, connection).is_some_and(|operation| {
            if uses_owned_key(operation) {
                case.execution.mutation == "mutating"
            } else {
                case.execution.mutation == "read_only" || operation == 0x25
            }
        })
}

fn operation(case: &Case, connection: &Connection) -> Option<u32> {
    match (case.case_id.as_str(), connection.protocol_version.as_str()) {
        ("test-op-create", "2.0" | "2.1") => Some(CREATE),
        ("test-op-check", "2.0" | "2.1") => Some(CHECK),
        ("test-op-get", "2.0" | "2.1") => Some(GET),
        ("test-op-get-attributes", "2.0" | "2.1") => Some(GET_ATTRIBUTES),
        ("test-op-get-attribute-list", "2.0" | "2.1") => Some(GET_ATTRIBUTE_LIST),
        ("test-op-activate", "2.0" | "2.1") => Some(ACTIVATE),
        ("test-op-revoke", "2.0" | "2.1") => Some(REVOKE),
        ("test-op-destroy", "2.0" | "2.1") => Some(DESTROY),
        ("test-op-archive", "2.0" | "2.1") => Some(ARCHIVE),
        ("test-op-recover", "2.0" | "2.1") => Some(RECOVER),
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
    let version = ProtocolVersion::from_raw(2, i32::from(connection.protocol_version == "2.1"));
    let mut result = match operation(case, connection) {
        Some(operation) if uses_owned_key(operation) => {
            run_owned_key(case, connection, version, operation)
        }
        Some(operation) => outcome(
            case,
            exchange(connection, version, operation, None, |batch| {
                check_probe(batch, operation, version)
            }),
        ),
        None => CaseResult::non_run(case, "not_run", "no generic exchange recipe".to_owned()),
    };
    result.duration_ms = start.elapsed().as_secs_f64() * 1000.0;
    result
}

type ExchangeResult = Result<(u32, Option<u32>)>;

fn outcome(case: &Case, result: ExchangeResult) -> CaseResult {
    match result {
        Ok((status, reason)) => {
            let mut result = crate::runner::observed_result(case, status, reason);
            "generic TTLV request and response validated".clone_into(&mut result.diagnostic);
            result
        }
        Err(problem) => CaseResult::non_run(case, "failed", problem.to_owned()),
    }
}

fn run_owned_key(
    case: &Case,
    connection: &Connection,
    version: ProtocolVersion,
    operation: u32,
) -> CaseResult {
    let mut identifier = None;
    let create = exchange(connection, version, CREATE, None, |batch| {
        identifier = text_identifier(batch);
        Ok(identifier.is_some()
            && has_field(batch, 0x0042_0057, |v| {
                matches!(v, ValueView::Enumeration(2))
            }))
    });
    let Some(id) = identifier else {
        if operation != CREATE && matches!(create, Ok((1, _))) {
            return CaseResult::non_run(
                case,
                "not_run",
                "server rejected isolated Create setup".to_owned(),
            );
        }
        return outcome(case, create);
    };
    let same_id = |batch: ResponseBatchItemView<'_>| {
        Ok(text_identifier(batch).as_deref() == Some(id.as_str()))
    };
    let send = |operation| exchange(connection, version, operation, Some(&id), same_id);

    if create == Ok((0, None)) && operation == DESTROY {
        return outcome(case, send(DESTROY));
    }
    if create == Ok((0, None)) && matches!(operation, ACTIVATE | REVOKE | CHECK) {
        let target = activate_and_revoke(case, operation, connection, version, &id);
        return if send(DESTROY) == Ok((0, None)) {
            target
        } else {
            outcome(case, Err("cleanup Destroy failed for the isolated key"))
        };
    }
    let mut result = outcome(case, create);
    if create == Ok((0, None)) {
        if operation == GET {
            result = outcome(
                case,
                exchange(connection, version, GET, Some(&id), |batch| {
                    Ok(same_id(batch)?
                        && has_field(batch, 0x0042_0057, |v| {
                            matches!(v, ValueView::Enumeration(2))
                        })
                        && has_field(batch, 0x0042_008f, |v| matches!(v, ValueView::Structure(_))))
                }),
            );
        }
        if operation == GET_ATTRIBUTES {
            result = outcome(
                case,
                exchange(connection, version, GET_ATTRIBUTES, Some(&id), |batch| {
                    Ok(same_id(batch)?
                        && has_field(batch, 0x0042_0125, |value| match value {
                            ValueView::Structure(attrs) => match attrs.children() {
                                [length] if length.tag().raw() == 0x0042_002a => length
                                    .with_value(|value| matches!(value, ValueView::Integer(256))),
                                _ => false,
                            },
                            _ => false,
                        }))
                }),
            );
        }
        if operation == GET_ATTRIBUTE_LIST {
            result = outcome(
                case,
                exchange(connection, version, operation, Some(&id), |batch| {
                    Ok(same_id(batch)?
                        && batch
                            .with_response_payload(|payload| {
                                payload.children().iter().any(|item| {
                                    item.tag().raw() == 0x0042_013b
                                        && item.with_value(|v| {
                                            matches!(v, ValueView::Enumeration(0x0042_002a))
                                        })
                                })
                            })
                            .unwrap_or(false))
                }),
            );
        }
        if matches!(operation, ARCHIVE | RECOVER) {
            result = archive_and_recover(case, operation, connection, version, &id);
        }
    }
    if send(DESTROY) == Ok((0, None)) {
        result
    } else {
        outcome(case, Err("cleanup Destroy failed for the isolated key"))
    }
}

fn archive_and_recover(
    case: &Case,
    operation: u32,
    connection: &Connection,
    version: ProtocolVersion,
    id: &str,
) -> CaseResult {
    let same_id =
        |batch: ResponseBatchItemView<'_>| Ok(text_identifier(batch).as_deref() == Some(id));
    let mut archived = false;
    let archive = exchange(connection, version, ARCHIVE, Some(id), |batch| {
        archived = true;
        same_id(batch)
    });
    if archived {
        let recover = exchange(connection, version, RECOVER, Some(id), same_id);
        if archive == Ok((0, None)) {
            if operation == RECOVER {
                return outcome(case, recover);
            }
            if recover != Ok((0, None)) {
                return outcome(case, Err("cleanup Recover failed"));
            }
        }
    } else if operation == RECOVER && matches!(archive, Ok((1, _))) {
        return CaseResult::non_run(case, "not_run", "server rejected Archive setup".to_owned());
    }
    outcome(case, archive)
}

fn activate_and_revoke(
    case: &Case,
    operation: u32,
    connection: &Connection,
    version: ProtocolVersion,
    id: &str,
) -> CaseResult {
    let same_id =
        |batch: ResponseBatchItemView<'_>| Ok(text_identifier(batch).as_deref() == Some(id));
    let activation = exchange(connection, version, ACTIVATE, Some(id), same_id);
    if matches!(activation, Ok((1, _))) && operation != ACTIVATE {
        return CaseResult::non_run(
            case,
            "not_run",
            "server rejected isolated Activate setup".to_owned(),
        );
    }
    let target = if activation == Ok((0, None)) && operation == CHECK {
        exchange(connection, version, CHECK, Some(id), |batch| {
            let response = kmipkit::protocol::CheckResponse::try_from_response_item(batch)
                .map_err(|_| "invalid Check response")?;
            Ok(same_id(batch)?
                && response
                    .cryptographic_usage_mask()
                    .is_none_or(|mask| mask & 4 == 4))
        })
    } else {
        activation
    };
    if activation == Ok((0, None)) || activation.is_err() {
        let revocation = exchange(connection, version, REVOKE, Some(id), same_id);
        if activation == Ok((0, None)) {
            if operation == REVOKE {
                return outcome(case, revocation);
            }
            if revocation != Ok((0, None)) {
                return outcome(case, Err("cleanup Revoke failed for the activated key"));
            }
        }
    }
    outcome(case, target)
}

fn text_identifier(batch: ResponseBatchItemView<'_>) -> Option<String> {
    batch
        .with_response_payload(|payload| {
            let mut fields = payload
                .children()
                .iter()
                .filter(|item| item.tag().raw() == 0x0042_0094);
            let field = fields.next()?;
            if fields.next().is_some() {
                return None;
            }
            field.with_value(|value| match value {
                ValueView::TextString(text) if !text.is_empty() => Some(text.to_string()),
                _ => None,
            })
        })
        .flatten()
}

fn has_field(
    batch: ResponseBatchItemView<'_>,
    tag: u32,
    check: impl FnOnce(ValueView<'_>) -> bool,
) -> bool {
    batch
        .with_response_payload(|payload| {
            let mut fields = payload
                .children()
                .iter()
                .filter(|item| item.tag().raw() == tag);
            fields
                .next()
                .is_some_and(|field| fields.next().is_none() && field.with_value(check))
        })
        .unwrap_or_default()
}

fn exchange(
    connection: &Connection,
    version: ProtocolVersion,
    operation: u32,
    id: Option<&str>,
    check: impl FnOnce(ResponseBatchItemView<'_>) -> Result<bool>,
) -> ExchangeResult {
    let minor = i32::from(connection.protocol_version == "2.1");
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
        CREATE => vec![
            item(0x0042_0057, Value::enumeration(2))?,
            node(
                0x0042_0125,
                vec![
                    item(0x0042_0028, Value::enumeration(3))?,
                    item(0x0042_002a, Value::integer(256))?,
                    item(0x0042_002c, Value::integer(4))?,
                ],
            )?,
        ],
        GET_ATTRIBUTES => vec![
            item(
                0x0042_0094,
                Value::text_string(id.ok_or("object identifier missing")?.to_owned()),
            )?,
            item(0x0042_013b, Value::enumeration(0x0042_002a))?,
        ],
        CHECK => vec![
            item(
                0x0042_0094,
                Value::text_string(id.ok_or("object identifier missing")?.to_owned()),
            )?,
            item(0x0042_002c, Value::integer(4))?,
        ],
        REVOKE => vec![
            item(
                0x0042_0094,
                Value::text_string(id.ok_or("object identifier missing")?.to_owned()),
            )?,
            node(0x0042_0081, vec![item(0x0042_0082, Value::enumeration(6))?])?,
        ],
        GET | GET_ATTRIBUTE_LIST | ACTIVATE | DESTROY | ARCHIVE | RECOVER => vec![item(
            0x0042_0094,
            Value::text_string(id.ok_or("object identifier missing")?.to_owned()),
        )?],
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
    check_response(response.as_bytes(), operation, version, check)
}

fn check_response(
    wire: &[u8],
    operation: u32,
    version: ProtocolVersion,
    check: impl FnOnce(ResponseBatchItemView<'_>) -> Result<bool>,
) -> ExchangeResult {
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
    if status == 0 && !check(batch)? {
        return Err("generic success payload did not satisfy the case");
    }
    Ok((
        status,
        batch.result_reason().map(kmipkit::ResultReason::raw),
    ))
}

fn check_probe(
    batch: ResponseBatchItemView<'_>,
    operation: u32,
    version: ProtocolVersion,
) -> Result<bool> {
    Ok(if operation == 0x08 {
        let locate =
            LocateResponse::try_from_response_item(batch).map_err(|_| "invalid Locate response")?;
        let returned = locate.unique_identifiers().len();
        returned <= 1
            && locate
                .located_items()
                .is_none_or(|total| usize::try_from(total).is_ok_and(|total| total >= returned))
    } else if operation == 0x18 {
        let query =
            QueryResponse::try_from_response_item(batch).map_err(|_| "invalid Query response")?;
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
    })
}

fn node(tag: u32, children: Vec<Item>) -> Result<Item> {
    item(tag, Value::structure(structure(tag, children)?))
}

// Encode the native generic tree for the exchange recipes.
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
            ValueView::TextString(value) => {
                output.extend_from_slice(value.as_bytes());
                7
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
