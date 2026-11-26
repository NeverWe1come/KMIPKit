//! KMIP 2.x exchanges constructed using public raw-tag models.
use crate::driver::{item, structure, transport_config};
use crate::generic_wire::encode;
use crate::models::{Case, CaseResult, Connection, Result};
use kmipkit::RequestOptions;
use kmipkit::protocol::{
    DiscoverVersionsResponse, HashResponse, LocateResponse, PingResponse, ProtocolVersion,
    ResponseBatchItemView, ResponseMessage, RngRetrieveResponse,
};
use kmipkit::ttlv::{Item, Value, ValueView, codec};
use kmipkit_transport::RawTlsTransport;
use sha2::{Digest, Sha256};

const TEXT_ATTRIBUTE_FIXTURE: &str = "kmipkit-attribute-fixture";
const HASH_INPUT: &[u8] = b"abc";
const RNG_SEED_INPUT: [u8; 32] = [b'A'; 32];
const CREATE: u32 = 1;
const CREATE_KEY_PAIR: u32 = 2;
const REGISTER: u32 = 3;
const RE_KEY: u32 = 4;
const DERIVE_KEY: u32 = 5;
const CERTIFY: u32 = 6;
const RE_CERTIFY: u32 = 7;
const CHECK: u32 = 9;
const GET: u32 = 10;
const GET_ATTRIBUTES: u32 = 11;
const GET_ATTRIBUTE_LIST: u32 = 12;
const ADD_ATTRIBUTE: u32 = 13;
const MODIFY_ATTRIBUTE: u32 = 14;
const DELETE_ATTRIBUTE: u32 = 15;
const OBTAIN_LEASE: u32 = 16;
const GET_USAGE_ALLOCATION: u32 = 0x11;
const ACTIVATE: u32 = 18;
const REVOKE: u32 = 19;
const DESTROY: u32 = 20;
const ARCHIVE: u32 = 21;
const RECOVER: u32 = 22;
const VALIDATE: u32 = 0x17;
const CANCEL: u32 = 0x19;
const POLL: u32 = 0x1a;
const RE_KEY_KEY_PAIR: u32 = 0x1d;
const ENCRYPT: u32 = 0x1f;
const DECRYPT: u32 = 0x20;
const SIGN: u32 = 0x21;
const SIGNATURE_VERIFY: u32 = 0x22;
const MAC: u32 = 0x23;
const MAC_VERIFY: u32 = 0x24;
const CREATE_SPLIT_KEY: u32 = 0x28;
const JOIN_SPLIT_KEY: u32 = 0x29;
const IMPORT: u32 = 0x2a;
const EXPORT: u32 = 0x2b;
const SET_ATTRIBUTE: u32 = 0x31;
const DESCRIPTION: u32 = 0x0042_00fc;
const CHANGED_DESCRIPTION: &str = "kmipkit-changed-description";

fn changes_description(operation: u32) -> bool {
    matches!(
        operation,
        ADD_ATTRIBUTE | MODIFY_ATTRIBUTE | DELETE_ATTRIBUTE | SET_ATTRIBUTE
    )
}

fn uses_owned_key(operation: u32) -> bool {
    matches!(
        operation,
        CREATE
            | CHECK
            | GET
            | GET_ATTRIBUTES
            | GET_ATTRIBUTE_LIST
            | OBTAIN_LEASE
            | ACTIVATE
            | REVOKE
            | DESTROY
            | ARCHIVE
            | RECOVER
    ) || changes_description(operation)
}

fn is_extended_mutating_op(operation: u32) -> bool {
    matches!(
        operation,
        CREATE_KEY_PAIR
            | REGISTER
            | RE_KEY
            | DERIVE_KEY
            | CERTIFY
            | RE_CERTIFY
            | GET_USAGE_ALLOCATION
            | VALIDATE
            | CANCEL
            | POLL
            | RE_KEY_KEY_PAIR
            | ENCRYPT
            | DECRYPT
            | SIGN
            | SIGNATURE_VERIFY
            | MAC
            | MAC_VERIFY
            | CREATE_SPLIT_KEY
            | JOIN_SPLIT_KEY
            | IMPORT
            | EXPORT
    )
}

pub(crate) fn supports(case: &Case, connection: &Connection) -> bool {
    connection.transport == "raw_tls"
        && connection.encoding == "TTLV"
        && connection.authentication.is_none()
        && operation(case, connection).is_some_and(|operation| {
            if uses_owned_key(operation) || is_extended_mutating_op(operation) {
                case.execution.mutation == "mutating"
            } else {
                case.execution.mutation == "read_only"
                    || matches!(operation, 0x25 | 0x26)
                    || case.case_id == "test-multipart-hash-sha256"
            }
        })
}

fn operation(case: &Case, connection: &Connection) -> Option<u32> {
    match (case.case_id.as_str(), connection.protocol_version.as_str()) {
        ("test-op-create", "2.0" | "2.1") => Some(CREATE),
        ("test-op-create-key-pair", "2.0" | "2.1") => Some(CREATE_KEY_PAIR),
        ("test-op-register", "2.0" | "2.1") => Some(REGISTER),
        ("test-op-re-key", "2.0" | "2.1") => Some(RE_KEY),
        ("test-op-derive-key", "2.0" | "2.1") => Some(DERIVE_KEY),
        ("test-op-certify", "2.0" | "2.1") => Some(CERTIFY),
        ("test-op-re-certify", "2.0" | "2.1") => Some(RE_CERTIFY),
        ("test-op-check", "2.0" | "2.1") => Some(CHECK),
        ("test-op-get", "2.0" | "2.1") => Some(GET),
        (id, "2.0" | "2.1") if key_attribute_reference(id).is_some() => Some(GET_ATTRIBUTES),
        ("test-op-get-attribute-list", "2.0" | "2.1") => Some(GET_ATTRIBUTE_LIST),
        ("test-op-obtain-lease", "2.0" | "2.1") => Some(OBTAIN_LEASE),
        ("test-op-get-usage-allocation", "2.0" | "2.1") => Some(GET_USAGE_ALLOCATION),
        ("test-op-add-attribute", "2.0" | "2.1") => Some(ADD_ATTRIBUTE),
        ("test-op-modify-attribute", "2.0" | "2.1") => Some(MODIFY_ATTRIBUTE),
        ("test-op-delete-attribute", "2.0" | "2.1") => Some(DELETE_ATTRIBUTE),
        ("test-op-set-attribute", "2.0" | "2.1") => Some(SET_ATTRIBUTE),
        ("test-op-activate", "2.0" | "2.1") => Some(ACTIVATE),
        ("test-op-revoke", "2.0" | "2.1") => Some(REVOKE),
        ("test-op-destroy", "2.0" | "2.1") => Some(DESTROY),
        ("test-op-archive", "2.0" | "2.1") => Some(ARCHIVE),
        ("test-op-recover", "2.0" | "2.1") => Some(RECOVER),
        ("test-op-validate", "2.0" | "2.1") => Some(VALIDATE),
        ("test-op-cancel", "2.0" | "2.1") => Some(CANCEL),
        ("test-op-poll", "2.0" | "2.1") => Some(POLL),
        ("test-op-re-key-key-pair", "2.0" | "2.1") => Some(RE_KEY_KEY_PAIR),
        ("test-op-locate", "2.0" | "2.1") => Some(0x08),
        (_, "2.0" | "2.1")
            if case.operation == "Locate" && case.requires["rotate_attribute_ttlv"].is_string() =>
        {
            Some(0x08)
        }
        ("test-op-query", "2.0" | "2.1") => Some(0x18),
        ("test-op-discover-versions", "2.0" | "2.1") => Some(0x1e),
        ("test-op-rng-retrieve" | "test-crypto-rng-retrieve-rng-32", "2.0" | "2.1") => Some(0x25),
        ("test-op-rng-seed" | "test-crypto-rng-seed-rng-32", "2.0" | "2.1") => Some(0x26),
        (
            "test-op-hash" | "test-crypto-hash-sha_256-0" | "test-multipart-hash-sha256",
            "2.0" | "2.1",
        ) => Some(0x27),
        ("test-op-encrypt" | "test-crypto-encrypt-aes-256-cbc", "2.0" | "2.1") => Some(ENCRYPT),
        ("test-op-decrypt" | "test-crypto-decrypt-aes-256-cbc", "2.0" | "2.1") => Some(DECRYPT),
        (
            "test-op-sign"
            | "test-crypto-sign-rsa-2048-pkcs1_v1_5"
            | "test-crypto-sign-ecdsa-256-p_256",
            "2.0" | "2.1",
        ) => Some(SIGN),
        (
            "test-op-signature-verify"
            | "test-crypto-signature-verify-rsa-2048-pkcs1_v1_5"
            | "test-crypto-signature-verify-ecdsa-256-p_256",
            "2.0" | "2.1",
        ) => Some(SIGNATURE_VERIFY),
        ("test-op-mac" | "test-crypto-mac-hmac_sha256-256", "2.0" | "2.1") => Some(MAC),
        ("test-op-mac-verify" | "test-crypto-mac-verify-hmac_sha256-256", "2.0" | "2.1") => {
            Some(MAC_VERIFY)
        }
        ("test-op-create-split-key", "2.0" | "2.1") => Some(CREATE_SPLIT_KEY),
        ("test-op-join-split-key", "2.0" | "2.1") => Some(JOIN_SPLIT_KEY),
        ("test-op-import", "2.0" | "2.1") => Some(IMPORT),
        ("test-op-export", "2.0" | "2.1") => Some(EXPORT),
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
        Some(operation) if is_extended_mutating_op(operation) => {
            run_extended_operation(case, connection, version, operation)
        }
        Some(0x08) if case.requires["rotate_attribute_ttlv"].is_string() => {
            locate_rotate_attribute(case, connection, version)
        }
        Some(0x27) if case.case_id == "test-multipart-hash-sha256" => {
            outcome(case, multipart_hash(connection, version))
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

fn locate_rotate_attribute(
    case: &Case,
    connection: &Connection,
    version: ProtocolVersion,
) -> CaseResult {
    let Ok(Some(attribute)) = crate::adapters::rotate_attribute(case) else {
        return outcome(case, Err("invalid Rotate fixture"));
    };
    let allowed = kmipkit::VersionGate::validate_tag(version, attribute.tag().raw()).is_ok();
    if case.requires["expected_status"] == "NotSent" {
        let status = if allowed { "failed" } else { "passed" };
        let mut result = CaseResult::non_run(
            case,
            status,
            "checked the Rotate tag version before transport".to_owned(),
        );
        result.library_correctness = status;
        return result;
    }
    let fields = (|| {
        Ok(vec![
            item(0x0042_004f, Value::integer(1))?,
            item(0x0042_00d4, Value::integer(0))?,
            node(0x0042_0125, vec![attribute])?,
        ])
    })();
    outcome(
        case,
        fields.and_then(|fields| {
            exchange_payload(connection, version, 0x08, fields, |batch| {
                check_probe(batch, 0x08, version)
            })
        }),
    )
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
    let create_reference = if changes_description(operation) && operation != ADD_ATTRIBUTE {
        DESCRIPTION
    } else {
        key_attribute_reference(&case.case_id).unwrap_or_default()
    };
    let create = exchange_with_reference(
        connection,
        version,
        CREATE,
        None,
        create_reference,
        |batch| {
            identifier = text_identifier(batch);
            Ok(identifier.is_some()
                && has_field(batch, 0x0042_0057, |v| {
                    matches!(v, ValueView::Enumeration(2))
                }))
        },
    );
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
            result = outcome(case, get_key(connection, version, &id));
        }
        if operation == OBTAIN_LEASE {
            result = outcome(
                case,
                exchange(connection, version, operation, Some(&id), |batch| {
                    Ok(same_id(batch)?
                        && has_field(batch, 0x0042_0049, |value| {
                            matches!(value, ValueView::Interval(_))
                        })
                        && has_field(batch, 0x0042_0048, |value| {
                            matches!(value, ValueView::DateTime(_))
                        }))
                }),
            );
        }
        if operation == GET_ATTRIBUTES {
            result = outcome(case, get_attributes(case, connection, version, &id));
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
        if changes_description(operation) {
            result = outcome(
                case,
                mutate_description(connection, version, operation, &id),
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

fn get_key(connection: &Connection, version: ProtocolVersion, id: &str) -> ExchangeResult {
    exchange(connection, version, GET, Some(id), |batch| {
        Ok(text_identifier(batch).as_deref() == Some(id)
            && has_field(batch, 0x0042_0057, |value| {
                matches!(value, ValueView::Enumeration(2))
            })
            && has_field(batch, 0x0042_008f, |value| {
                matches!(value, ValueView::Structure(_))
            }))
    })
}

fn key_attribute_reference(case_id: &str) -> Option<u32> {
    match case_id {
        "test-attr-cryptographic-algorithm" => Some(0x0042_0028),
        "test-op-get-attributes" | "test-attr-cryptographic-length" => Some(0x0042_002a),
        "test-attr-cryptographic-usage-mask" => Some(0x0042_002c),
        "test-attr-object-type" => Some(0x0042_0057),
        "test-attr-state" => Some(0x0042_008d),
        "test-attr-contact-information" => Some(0x0042_0022),
        "test-attr-description" => Some(0x0042_00fc),
        "test-attr-comment" => Some(0x0042_00fd),
        "test-attr-sensitive" => Some(0x0042_0120),
        "test-attr-always-sensitive" => Some(0x0042_0121),
        "test-attr-extractable" => Some(0x0042_0122),
        "test-attr-never-extractable" => Some(0x0042_0123),
        _ => None,
    }
}

fn get_attributes(
    case: &Case,
    connection: &Connection,
    version: ProtocolVersion,
    id: &str,
) -> ExchangeResult {
    let Some(tag) = key_attribute_reference(&case.case_id) else {
        return Err("no direct-tag attribute recipe");
    };
    exchange_with_reference(
        connection,
        version,
        GET_ATTRIBUTES,
        Some(id),
        tag,
        |batch| {
            Ok(text_identifier(batch).as_deref() == Some(id)
                && has_field(batch, 0x0042_0125, |value| {
                    let ValueView::Structure(attrs) = value else {
                        return false;
                    };
                    let [attribute] = attrs.children() else {
                        return false;
                    };
                    attribute.tag().raw() == tag
                        && attribute.with_value(|value| {
                            if matches!(tag, 0x0042_0022 | 0x0042_00fc | 0x0042_00fd) {
                                return matches!(value, ValueView::TextString(text) if text == TEXT_ATTRIBUTE_FIXTURE);
                            }
                            matches!(
                                (tag, value),
                                (0x0042_0028, ValueView::Enumeration(3))
                                    | (0x0042_002a, ValueView::Integer(256))
                                    | (0x0042_002c, ValueView::Integer(4))
                                    | (0x0042_0057, ValueView::Enumeration(2))
                                    | (0x0042_008d, ValueView::Enumeration(1))
                                    | (
                                        0x0042_0120 | 0x0042_0121 | 0x0042_0123,
                                        ValueView::Boolean(true)
                                    )
                                    | (0x0042_0122, ValueView::Boolean(false))
                            )
                        })
                }))
        },
    )
}

fn mutate_description(
    connection: &Connection,
    version: ProtocolVersion,
    operation: u32,
    id: &str,
) -> ExchangeResult {
    let read = |expected| {
        exchange_with_reference(
            connection,
            version,
            GET_ATTRIBUTES,
            Some(id),
            DESCRIPTION,
            |batch| {
                Ok(text_identifier(batch).as_deref() == Some(id)
                    && description_matches(batch, expected))
            },
        )
    };
    let initial = (operation != ADD_ATTRIBUTE).then_some(TEXT_ATTRIBUTE_FIXTURE);
    if read(initial)? != (0, None) {
        return Err("could not read the initial Description");
    }

    let mut fields = vec![item(0x0042_0094, Value::text_string(id.to_owned()))?];
    if operation == MODIFY_ATTRIBUTE {
        fields.push(description_wrapper(0x0042_013c, TEXT_ATTRIBUTE_FIXTURE)?);
    }
    if operation == DELETE_ATTRIBUTE {
        fields.push(item(0x0042_013b, Value::enumeration(DESCRIPTION))?);
    } else {
        fields.push(description_wrapper(0x0042_013d, CHANGED_DESCRIPTION)?);
    }
    let observed = exchange_payload(connection, version, operation, fields, |batch| {
        Ok(text_identifier(batch).as_deref() == Some(id))
    })?;
    if observed == (0, None) {
        let expected = (operation != DELETE_ATTRIBUTE).then_some(CHANGED_DESCRIPTION);
        if read(expected)? != (0, None) {
            return Err("could not verify the Description after mutation");
        }
    }
    Ok(observed)
}

fn description_wrapper(tag: u32, text: &str) -> Result<Item> {
    node(
        tag,
        vec![item(DESCRIPTION, Value::text_string(text.to_owned()))?],
    )
}

fn description_matches(batch: ResponseBatchItemView<'_>, expected: Option<&str>) -> bool {
    let Some(expected) = expected else {
        return batch
            .with_response_payload(|payload| payload.children().len() == 1)
            .unwrap_or(false);
    };
    batch
        .with_response_payload(|payload| payload.children().len() == 2)
        .unwrap_or(false)
        && has_field(batch, 0x0042_0125, |value| {
            let ValueView::Structure(attributes) = value else {
                return false;
            };
            let [description] = attributes.children() else {
                return false;
            };
            description.tag().raw() == DESCRIPTION
                && description.with_value(
                    |value| matches!(value, ValueView::TextString(text) if text == expected),
                )
        })
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

fn multipart_hash(connection: &Connection, version: ProtocolVersion) -> ExchangeResult {
    let mut correlation = Vec::new();
    let digest = Sha256::digest(b"KMIP Hash!");
    for (stage, chunk) in [b"KMIP ".as_slice(), b"Hash", b"!"].into_iter().enumerate() {
        let mut fields = hash_fields(chunk)?;
        if stage == 0 {
            fields.push(item(0x0042_00d7, Value::boolean(true))?);
        } else {
            fields.push(item(0x0042_00d6, Value::byte_string(correlation.clone()))?);
        }
        if stage == 2 {
            fields.push(item(0x0042_00d8, Value::boolean(true))?);
        }
        let response = exchange_payload(connection, version, 0x27, fields, |batch| {
            Ok(batch
                .with_response_payload(|payload| {
                    let children = payload.children();
                    if stage == 1 {
                        return children.is_empty();
                    }
                    let [field] = children else { return false };
                    let expected_tag = if stage == 0 { 0x0042_00d6 } else { 0x0042_00c2 };
                    field.tag().raw() == expected_tag
                        && field.with_value(|value| {
                            let ValueView::ByteString(bytes) = value else {
                                return false;
                            };
                            if stage == 0 {
                                correlation.extend_from_slice(bytes);
                                !bytes.is_empty()
                            } else {
                                bytes == &digest[..]
                            }
                        })
                })
                .unwrap_or_default())
        })?;
        if response != (0, None) || stage == 2 {
            return Ok(response);
        }
    }
    unreachable!("Hash stages always include Final")
}

fn hash_fields(data: &[u8]) -> Result<Vec<Item>> {
    Ok(vec![
        node(0x0042_002b, vec![item(0x0042_0038, Value::enumeration(6))?])?,
        item(0x0042_00c2, Value::byte_string(data.to_vec()))?,
    ])
}

fn exchange(
    connection: &Connection,
    version: ProtocolVersion,
    operation: u32,
    id: Option<&str>,
    check: impl FnOnce(ResponseBatchItemView<'_>) -> Result<bool>,
) -> ExchangeResult {
    exchange_with_reference(connection, version, operation, id, 0, check)
}

fn exchange_with_reference(
    connection: &Connection,
    version: ProtocolVersion,
    operation: u32,
    id: Option<&str>,
    attribute_reference: u32,
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
        CREATE => {
            let mut attrs = vec![
                item(0x0042_0028, Value::enumeration(3))?,
                item(0x0042_002a, Value::integer(256))?,
                item(0x0042_002c, Value::integer(4))?,
            ];
            if (0x0042_0120..=0x0042_0123).contains(&attribute_reference) {
                attrs.push(item(0x0042_0120, Value::boolean(true))?);
                attrs.push(item(0x0042_0122, Value::boolean(false))?);
            }
            if matches!(attribute_reference, 0x0042_0022 | 0x0042_00fc | 0x0042_00fd) {
                attrs.push(item(
                    attribute_reference,
                    Value::text_string(TEXT_ATTRIBUTE_FIXTURE.to_owned()),
                )?);
            }
            vec![
                item(0x0042_0057, Value::enumeration(2))?,
                node(0x0042_0125, attrs)?,
            ]
        }
        GET_ATTRIBUTES => vec![
            item(
                0x0042_0094,
                Value::text_string(id.ok_or("object identifier missing")?.to_owned()),
            )?,
            item(0x0042_013b, Value::enumeration(attribute_reference))?,
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
        GET | GET_ATTRIBUTE_LIST | OBTAIN_LEASE | ACTIVATE | DESTROY | ARCHIVE | RECOVER => {
            vec![item(
                0x0042_0094,
                Value::text_string(id.ok_or("object identifier missing")?.to_owned()),
            )?]
        }
        0x08 => vec![
            item(0x0042_004f, Value::integer(1))?,
            item(0x0042_00d4, Value::integer(0))?,
        ],
        0x18 => vec![item(0x0042_0074, Value::enumeration(1))?],
        0x1e => vec![version_item()?],
        0x25 => vec![item(0x0042_00c4, Value::integer(32))?],
        0x26 => vec![item(
            0x0042_00c2,
            Value::byte_string(RNG_SEED_INPUT.to_vec()),
        )?],
        0x27 => hash_fields(HASH_INPUT)?,
        _ => Vec::new(),
    };
    exchange_payload(connection, version, operation, payload_fields, check)
}

fn exchange_payload(
    connection: &Connection,
    version: ProtocolVersion,
    operation: u32,
    payload_fields: Vec<Item>,
    check: impl FnOnce(ResponseBatchItemView<'_>) -> Result<bool>,
) -> ExchangeResult {
    let protocol = node(
        0x0042_0069,
        vec![
            item(0x0042_006a, Value::integer(2))?,
            item(
                0x0042_006b,
                Value::integer(i32::from(connection.protocol_version == "2.1")),
            )?,
        ],
    )?;
    let request = node(
        0x0042_0078,
        vec![
            node(
                0x0042_0077,
                vec![protocol, item(0x0042_000d, Value::integer(1))?],
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
        batch
            .with_response_payload(|payload| {
                let mut operations = payload
                    .children()
                    .iter()
                    .filter(|field| field.tag().raw() == 0x0042_005c)
                    .peekable();
                operations.peek().is_some()
                    && operations.all(|field| {
                        field.with_value(
                            |value| matches!(value, ValueView::Enumeration(code) if *code != 0),
                        )
                    })
            })
            .unwrap_or(false)
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
    } else if operation == 0x26 {
        batch
            .with_response_payload(|payload| payload.children().len() == 1)
            .unwrap_or(false)
            && has_field(batch, 0x0042_00c4, |value| {
                matches!(value, ValueView::Integer(0..=32))
            })
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

fn field_text(batch: ResponseBatchItemView<'_>, tag: u32) -> Option<String> {
    batch
        .with_response_payload(|payload| {
            payload
                .children()
                .iter()
                .find(|item| item.tag().raw() == tag)
                .and_then(|item| {
                    item.with_value(|value| match value {
                        ValueView::TextString(text) if !text.is_empty() => Some(text.to_string()),
                        _ => None,
                    })
                })
        })
        .flatten()
}

fn all_text_identifiers(batch: ResponseBatchItemView<'_>) -> Vec<String> {
    batch
        .with_response_payload(|payload| {
            payload
                .children()
                .iter()
                .filter(|item| item.tag().raw() == 0x0042_0094)
                .filter_map(|item| {
                    item.with_value(|value| match value {
                        ValueView::TextString(text) if !text.is_empty() => Some(text.to_string()),
                        _ => None,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn field_bytes(batch: ResponseBatchItemView<'_>, tag: u32) -> Option<Vec<u8>> {
    batch
        .with_response_payload(|payload| {
            payload
                .children()
                .iter()
                .find(|item| item.tag().raw() == tag)
                .and_then(|item| {
                    item.with_value(|value| match value {
                        ValueView::ByteString(bytes) if !bytes.is_empty() => Some(bytes.to_vec()),
                        _ => None,
                    })
                })
        })
        .flatten()
}

fn destroy_ids(connection: &Connection, version: ProtocolVersion, ids: &[String]) {
    for id in ids {
        let _ = exchange(connection, version, REVOKE, Some(id), |_| Ok(true));
        let _ = exchange(connection, version, DESTROY, Some(id), |_| Ok(true));
    }
}

fn create_aes_key(
    connection: &Connection,
    version: ProtocolVersion,
    usage_mask: i32,
    extractable: bool,
) -> std::result::Result<(String, ExchangeResult), &'static str> {
    let mut attrs = vec![
        item(0x0042_0028, Value::enumeration(3))?,
        item(0x0042_002a, Value::integer(256))?,
        item(0x0042_002c, Value::integer(usage_mask))?,
    ];
    if extractable {
        attrs.push(item(0x0042_0122, Value::boolean(true))?);
    }
    let fields = vec![
        item(0x0042_0057, Value::enumeration(2))?,
        node(0x0042_0125, attrs)?,
    ];
    let mut created = None;
    let res = exchange_payload(connection, version, CREATE, fields, |batch| {
        created = text_identifier(batch);
        Ok(created.is_some())
    })?;
    Ok((created.unwrap_or_default(), Ok(res)))
}

fn create_hmac_key(
    connection: &Connection,
    version: ProtocolVersion,
    usage_mask: i32,
) -> std::result::Result<(String, ExchangeResult), &'static str> {
    let attrs = vec![
        item(0x0042_0028, Value::enumeration(9))?,
        item(0x0042_002a, Value::integer(256))?,
        item(0x0042_002c, Value::integer(usage_mask))?,
    ];
    let fields = vec![
        item(0x0042_0057, Value::enumeration(2))?,
        node(0x0042_0125, attrs)?,
    ];
    let mut created = None;
    let res = exchange_payload(connection, version, CREATE, fields, |batch| {
        created = text_identifier(batch);
        Ok(created.is_some())
    })?;
    Ok((created.unwrap_or_default(), Ok(res)))
}

fn create_rsa_key_pair(
    connection: &Connection,
    version: ProtocolVersion,
) -> std::result::Result<(String, String, ExchangeResult), &'static str> {
    let fields = vec![
        node(
            0x0042_0126,
            vec![
                item(0x0042_0028, Value::enumeration(4))?,
                item(0x0042_002a, Value::integer(2048))?,
            ],
        )?,
        node(0x0042_0127, vec![item(0x0042_002c, Value::integer(1))?])?,
        node(0x0042_0128, vec![item(0x0042_002c, Value::integer(2))?])?,
    ];
    let mut priv_id = None;
    let mut pub_id = None;
    let res = exchange_payload(connection, version, CREATE_KEY_PAIR, fields, |batch| {
        priv_id = field_text(batch, 0x0042_0066);
        pub_id = field_text(batch, 0x0042_006f);
        Ok(priv_id.is_some() && pub_id.is_some())
    })?;
    Ok((
        priv_id.unwrap_or_default(),
        pub_id.unwrap_or_default(),
        Ok(res),
    ))
}

fn run_extended_operation(
    case: &Case,
    connection: &Connection,
    version: ProtocolVersion,
    operation: u32,
) -> CaseResult {
    outcome(case, run_extended_inner(connection, version, operation))
}

fn run_extended_inner(
    connection: &Connection,
    version: ProtocolVersion,
    operation: u32,
) -> ExchangeResult {
    let mut cleanup: Vec<String> = Vec::new();
    let res = match operation {
        POLL | CANCEL => {
            let fields = vec![item(
                0x0042_0006,
                Value::byte_string(vec![1, 2, 3, 4, 5, 6]),
            )?];
            exchange_payload(connection, version, operation, fields, |batch| {
                Ok(field_bytes(batch, 0x0042_0006).is_some())
            })
        }
        REGISTER | IMPORT => extended_register_import(connection, version, operation, &mut cleanup),
        CREATE_KEY_PAIR | RE_KEY_KEY_PAIR => {
            extended_key_pair_ops(connection, version, operation, &mut cleanup)
        }
        RE_KEY | EXPORT | GET_USAGE_ALLOCATION | DERIVE_KEY | CREATE_SPLIT_KEY | JOIN_SPLIT_KEY => {
            extended_sym_ops(connection, version, operation, &mut cleanup)
        }
        CERTIFY | RE_CERTIFY | VALIDATE => {
            extended_cert_ops(connection, version, operation, &mut cleanup)
        }
        ENCRYPT | DECRYPT => extended_encrypt_ops(connection, version, operation, &mut cleanup),
        MAC | MAC_VERIFY => extended_mac_ops(connection, version, operation, &mut cleanup),
        SIGN | SIGNATURE_VERIFY => extended_sign_ops(connection, version, operation, &mut cleanup),
        _ => Err("unsupported extended generic operation"),
    };
    destroy_ids(connection, version, &cleanup);
    res
}

fn extended_register_import(
    connection: &Connection,
    version: ProtocolVersion,
    operation: u32,
    cleanup: &mut Vec<String>,
) -> ExchangeResult {
    let mut fields = Vec::new();
    if operation == IMPORT {
        fields.push(item(
            0x0042_0094,
            Value::text_string("kmipkit-generic-import".to_owned()),
        )?);
    }
    fields.push(item(0x0042_0057, Value::enumeration(2))?);
    if operation == IMPORT {
        fields.push(item(0x0042_0124, Value::boolean(false))?);
    }
    fields.push(node(
        0x0042_0125,
        vec![
            item(0x0042_0028, Value::enumeration(3))?,
            item(0x0042_002a, Value::integer(128))?,
            item(0x0042_002c, Value::integer(4))?,
        ],
    )?);
    fields.push(node(
        0x0042_008f,
        vec![node(
            0x0042_0040,
            vec![
                item(0x0042_0042, Value::enumeration(1))?,
                node(
                    0x0042_0045,
                    vec![item(0x0042_0043, Value::byte_string(vec![0x11; 16]))?],
                )?,
            ],
        )?],
    )?);
    exchange_payload(connection, version, operation, fields, |batch| {
        if let Some(id) = text_identifier(batch) {
            cleanup.push(id);
            return Ok(true);
        }
        Ok(false)
    })
}

fn extended_key_pair_ops(
    connection: &Connection,
    version: ProtocolVersion,
    operation: u32,
    cleanup: &mut Vec<String>,
) -> ExchangeResult {
    let (priv_id, pub_id, kp_res) = create_rsa_key_pair(connection, version)?;
    if !priv_id.is_empty() {
        cleanup.push(priv_id.clone());
    }
    if !pub_id.is_empty() {
        cleanup.push(pub_id);
    }
    if operation == CREATE_KEY_PAIR || kp_res != Ok((0, None)) {
        return kp_res;
    }
    let fields = vec![
        item(0x0042_0066, Value::text_string(priv_id))?,
        item(0x0042_0058, Value::interval(3600))?,
    ];
    exchange_payload(connection, version, RE_KEY_KEY_PAIR, fields, |batch| {
        let new_priv = field_text(batch, 0x0042_0066);
        let new_pub = field_text(batch, 0x0042_006f);
        if let Some(id) = &new_priv {
            cleanup.push(id.clone());
        }
        if let Some(id) = &new_pub {
            cleanup.push(id.clone());
        }
        Ok(new_priv.is_some() && new_pub.is_some())
    })
}

fn extended_sym_ops(
    connection: &Connection,
    version: ProtocolVersion,
    operation: u32,
    cleanup: &mut Vec<String>,
) -> ExchangeResult {
    let usage = if operation == DERIVE_KEY { 512 } else { 12 };
    let (key_id, key_res) = create_aes_key(connection, version, usage, operation == EXPORT)?;
    if !key_id.is_empty() {
        cleanup.push(key_id.clone());
    }
    if key_res != Ok((0, None)) {
        return key_res;
    }
    match operation {
        RE_KEY => {
            let fields = vec![
                item(0x0042_0094, Value::text_string(key_id.clone()))?,
                item(0x0042_0058, Value::interval(3600))?,
            ];
            exchange_payload(connection, version, RE_KEY, fields, |batch| {
                let new_id = text_identifier(batch);
                if let Some(id) = &new_id {
                    cleanup.push(id.clone());
                }
                Ok(new_id.is_some_and(|id| id != key_id))
            })
        }
        EXPORT => {
            let fields = vec![
                item(0x0042_0094, Value::text_string(key_id.clone()))?,
                item(0x0042_0042, Value::enumeration(1))?,
            ];
            exchange_payload(connection, version, EXPORT, fields, |batch| {
                Ok(text_identifier(batch).as_deref() == Some(key_id.as_str())
                    && has_field(batch, 0x0042_008f, |v| matches!(v, ValueView::Structure(_))))
            })
        }
        GET_USAGE_ALLOCATION => {
            let fields = vec![
                item(0x0042_0094, Value::text_string(key_id.clone()))?,
                item(0x0042_0096, Value::long_integer(32))?,
            ];
            exchange_payload(connection, version, GET_USAGE_ALLOCATION, fields, |batch| {
                Ok(text_identifier(batch).as_deref() == Some(key_id.as_str()))
            })
        }
        DERIVE_KEY => {
            let _ = exchange(connection, version, ACTIVATE, Some(&key_id), |_| Ok(true));
            let fields = vec![
                item(0x0042_0057, Value::enumeration(2))?,
                item(0x0042_0094, Value::text_string(key_id))?,
                item(0x0042_0031, Value::enumeration(2))?,
                node(0x0042_0032, vec![item(0x0042_0038, Value::enumeration(6))?])?,
                node(
                    0x0042_0125,
                    vec![
                        item(0x0042_0028, Value::enumeration(3))?,
                        item(0x0042_002a, Value::integer(128))?,
                        item(0x0042_002c, Value::integer(4))?,
                    ],
                )?,
            ];
            exchange_payload(connection, version, DERIVE_KEY, fields, |batch| {
                if let Some(id) = text_identifier(batch) {
                    cleanup.push(id);
                    return Ok(true);
                }
                Ok(false)
            })
        }
        _ => extended_split_ops(connection, version, operation, key_id, cleanup),
    }
}

fn extended_split_ops(
    connection: &Connection,
    version: ProtocolVersion,
    operation: u32,
    key_id: String,
    cleanup: &mut Vec<String>,
) -> ExchangeResult {
    let split_fields = vec![
        item(0x0042_0057, Value::enumeration(2))?,
        item(0x0042_0094, Value::text_string(key_id))?,
        item(0x0042_008b, Value::integer(2))?,
        item(0x0042_008c, Value::integer(2))?,
        item(0x0042_008a, Value::enumeration(1))?,
        node(
            0x0042_0125,
            vec![
                item(0x0042_0028, Value::enumeration(3))?,
                item(0x0042_002a, Value::integer(256))?,
                item(0x0042_002c, Value::integer(12))?,
            ],
        )?,
    ];
    let mut parts = Vec::new();
    let split_res = exchange_payload(
        connection,
        version,
        CREATE_SPLIT_KEY,
        split_fields,
        |batch| {
            parts = all_text_identifiers(batch);
            cleanup.extend(parts.iter().cloned());
            Ok(parts.len() == 2)
        },
    );
    if operation == CREATE_SPLIT_KEY || split_res != Ok((0, None)) {
        return split_res;
    }
    let join_fields = vec![
        item(0x0042_0057, Value::enumeration(2))?,
        item(0x0042_0094, Value::text_string(parts[0].clone()))?,
        item(0x0042_0094, Value::text_string(parts[1].clone()))?,
        node(
            0x0042_0125,
            vec![
                item(0x0042_0028, Value::enumeration(3))?,
                item(0x0042_002a, Value::integer(256))?,
                item(0x0042_002c, Value::integer(12))?,
            ],
        )?,
    ];
    exchange_payload(connection, version, JOIN_SPLIT_KEY, join_fields, |batch| {
        if let Some(id) = text_identifier(batch) {
            cleanup.push(id);
            return Ok(true);
        }
        Ok(false)
    })
}

fn extended_cert_ops(
    connection: &Connection,
    version: ProtocolVersion,
    operation: u32,
    cleanup: &mut Vec<String>,
) -> ExchangeResult {
    let (priv_id, pub_id, kp_res) = create_rsa_key_pair(connection, version)?;
    if !priv_id.is_empty() {
        cleanup.push(priv_id);
    }
    if !pub_id.is_empty() {
        cleanup.push(pub_id.clone());
    }
    if kp_res != Ok((0, None)) {
        return kp_res;
    }
    let certify_fields = vec![
        item(0x0042_0094, Value::text_string(pub_id))?,
        node(0x0042_0125, vec![item(0x0042_001d, Value::enumeration(1))?])?,
    ];
    let mut cert_id = None;
    let cert_res = exchange_payload(connection, version, CERTIFY, certify_fields, |batch| {
        cert_id = text_identifier(batch);
        if let Some(id) = &cert_id {
            cleanup.push(id.clone());
        }
        Ok(cert_id.is_some())
    });
    if operation == CERTIFY {
        return cert_res;
    }
    if cert_res? != (0, None) {
        return Err("the server rejected the prerequisite Certify request");
    }
    let target_cert = cert_id.ok_or("Certify did not return an owned certificate")?;
    let fields = vec![item(0x0042_0094, Value::text_string(target_cert))?];
    if operation == RE_CERTIFY {
        exchange_payload(connection, version, RE_CERTIFY, fields, |batch| {
            if let Some(id) = text_identifier(batch) {
                cleanup.push(id);
                return Ok(true);
            }
            Ok(false)
        })
    } else {
        exchange_payload(connection, version, VALIDATE, fields, |batch| {
            Ok(has_field(batch, 0x0042_009b, |v| {
                matches!(v, ValueView::Enumeration(1))
            }))
        })
    }
}

fn extended_encrypt_ops(
    connection: &Connection,
    version: ProtocolVersion,
    operation: u32,
    cleanup: &mut Vec<String>,
) -> ExchangeResult {
    let (key_id, key_res) = create_aes_key(connection, version, 12, false)?;
    if !key_id.is_empty() {
        cleanup.push(key_id.clone());
    }
    if key_res != Ok((0, None)) {
        return key_res;
    }
    let act = exchange(connection, version, ACTIVATE, Some(&key_id), |_| Ok(true))?;
    if act != (0, None) {
        return Ok(act);
    }
    let iv = vec![0x01; 16];
    let plaintext = b"0123456789abcdef".to_vec();
    let enc_fields = vec![
        item(0x0042_0094, Value::text_string(key_id.clone()))?,
        node(
            0x0042_002b,
            vec![
                item(0x0042_0011, Value::enumeration(1))?,
                item(0x0042_005f, Value::enumeration(3))?,
            ],
        )?,
        item(0x0042_00c2, Value::byte_string(plaintext.clone()))?,
        item(0x0042_003d, Value::byte_string(iv.clone()))?,
    ];
    let mut cipher = None;
    let enc_res = exchange_payload(connection, version, ENCRYPT, enc_fields, |batch| {
        cipher = field_bytes(batch, 0x0042_00c2);
        Ok(text_identifier(batch).as_deref() == Some(key_id.as_str()) && cipher.is_some())
    });
    if operation == ENCRYPT || enc_res != Ok((0, None)) {
        return enc_res;
    }
    let dec_fields = vec![
        item(0x0042_0094, Value::text_string(key_id.clone()))?,
        node(
            0x0042_002b,
            vec![
                item(0x0042_0011, Value::enumeration(1))?,
                item(0x0042_005f, Value::enumeration(3))?,
            ],
        )?,
        item(0x0042_00c2, Value::byte_string(cipher.unwrap_or_default()))?,
        item(0x0042_003d, Value::byte_string(iv))?,
    ];
    exchange_payload(connection, version, DECRYPT, dec_fields, |batch| {
        Ok(text_identifier(batch).as_deref() == Some(key_id.as_str())
            && field_bytes(batch, 0x0042_00c2).as_deref() == Some(plaintext.as_slice()))
    })
}

fn extended_mac_ops(
    connection: &Connection,
    version: ProtocolVersion,
    operation: u32,
    cleanup: &mut Vec<String>,
) -> ExchangeResult {
    let (key_id, key_res) = create_hmac_key(connection, version, 384)?;
    if !key_id.is_empty() {
        cleanup.push(key_id.clone());
    }
    if key_res != Ok((0, None)) {
        return key_res;
    }
    let act = exchange(connection, version, ACTIVATE, Some(&key_id), |_| Ok(true))?;
    if act != (0, None) {
        return Ok(act);
    }
    let mac_fields = vec![
        item(0x0042_0094, Value::text_string(key_id.clone()))?,
        node(0x0042_002b, vec![item(0x0042_0038, Value::enumeration(6))?])?,
        item(0x0042_00c2, Value::byte_string(HASH_INPUT.to_vec()))?,
    ];
    let mut tag = None;
    let mac_res = exchange_payload(connection, version, MAC, mac_fields, |batch| {
        tag = field_bytes(batch, 0x0042_00c6);
        Ok(text_identifier(batch).as_deref() == Some(key_id.as_str()) && tag.is_some())
    });
    if operation == MAC || mac_res != Ok((0, None)) {
        return mac_res;
    }
    let verify_fields = vec![
        item(0x0042_0094, Value::text_string(key_id))?,
        node(0x0042_002b, vec![item(0x0042_0038, Value::enumeration(6))?])?,
        item(0x0042_00c2, Value::byte_string(HASH_INPUT.to_vec()))?,
        item(0x0042_00c6, Value::byte_string(tag.unwrap_or_default()))?,
    ];
    exchange_payload(connection, version, MAC_VERIFY, verify_fields, |batch| {
        Ok(has_field(batch, 0x0042_009b, |v| {
            matches!(v, ValueView::Enumeration(1))
        }))
    })
}

fn extended_sign_ops(
    connection: &Connection,
    version: ProtocolVersion,
    operation: u32,
    cleanup: &mut Vec<String>,
) -> ExchangeResult {
    let (priv_id, pub_id, kp_res) = create_rsa_key_pair(connection, version)?;
    if !priv_id.is_empty() {
        cleanup.push(priv_id.clone());
    }
    if !pub_id.is_empty() {
        cleanup.push(pub_id.clone());
    }
    if kp_res != Ok((0, None)) {
        return kp_res;
    }
    let act_priv = exchange(connection, version, ACTIVATE, Some(&priv_id), |_| Ok(true))?;
    let act_pub = exchange(connection, version, ACTIVATE, Some(&pub_id), |_| Ok(true))?;
    if act_priv != (0, None) {
        return Ok(act_priv);
    }
    if act_pub != (0, None) {
        return Ok(act_pub);
    }
    let sign_fields = vec![
        item(0x0042_0094, Value::text_string(priv_id.clone()))?,
        node(
            0x0042_002b,
            vec![
                item(0x0042_0038, Value::enumeration(6))?,
                item(0x0042_005f, Value::enumeration(8))?,
            ],
        )?,
        item(0x0042_00c2, Value::byte_string(HASH_INPUT.to_vec()))?,
    ];
    let mut sig = None;
    let sign_res = exchange_payload(connection, version, SIGN, sign_fields, |batch| {
        sig = field_bytes(batch, 0x0042_00c3);
        Ok(text_identifier(batch).as_deref() == Some(priv_id.as_str()) && sig.is_some())
    });
    if operation == SIGN || sign_res != Ok((0, None)) {
        return sign_res;
    }
    let verify_fields = vec![
        item(0x0042_0094, Value::text_string(pub_id))?,
        node(
            0x0042_002b,
            vec![
                item(0x0042_0038, Value::enumeration(6))?,
                item(0x0042_005f, Value::enumeration(8))?,
            ],
        )?,
        item(0x0042_00c2, Value::byte_string(HASH_INPUT.to_vec()))?,
        item(0x0042_00c3, Value::byte_string(sig.unwrap_or_default()))?,
    ];
    exchange_payload(
        connection,
        version,
        SIGNATURE_VERIFY,
        verify_fields,
        |batch| {
            Ok(has_field(batch, 0x0042_009b, |v| {
                matches!(v, ValueView::Enumeration(1))
            }))
        },
    )
}

fn node(tag: u32, children: Vec<Item>) -> Result<Item> {
    item(tag, Value::structure(structure(tag, children)?))
}
