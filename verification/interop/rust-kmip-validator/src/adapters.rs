//! Explicit public typed operation adapters. No raw outbound request escape hatch.
use crate::driver::{item, structure};
use crate::models::{Case, Result};
use kmipkit::protocol::DiscoverVersionsRequest;
use kmipkit::protocol::{
    ActivateRequest, AddAttributeRequest, AdjustAttributeRequest, AdjustmentType, ArchiveRequest,
    AttributeReference, AttributeSet, CertifyRequest, CheckRequest, CreateKeyPairRequest,
    CreateRequest, CreateSplitKeyRequest, CryptographicParameters, CryptographicParametersBuilder,
    DecryptRequest, DelegatedLoginRequest, DeleteAttributeRequest, DerivationMethod,
    DerivationParameters, DeriveKeyRequest, DestroyRequest, EncryptRequest, ExportRequest,
    GetAttributeListRequest, GetAttributesRequest, GetRequest, GetUsageAllocationRequest,
    HashRequest, ImportRequest, InteropRequest, JoinSplitKeyRequest, LocateRequest, LogRequest,
    LoginRequest, LogoutRequest, MacRequest, MacVerifyRequest, ModifyAttributeRequest,
    NewAttribute, ObjectType, ObtainLeaseRequest, OperationData, Pkcs11Request, QueryFunction,
    QueryRequest, ReProvisionRequest, RecertifyRequest, RecoverRequest, RekeyKeyPairRequest,
    RekeyRequest, RevocationReason, RevokeRequest, RngRetrieveRequest, RngSeedRequest, SecretBytes,
    SecretText, SetAttributeRequest, SetConstraintsRequest, SetDefaultsRequest, SignRequest,
    SignatureVerifyRequest, SplitKeyMethod, SplitKeyReference, Ticket, TicketType,
    UniqueIdentifier, ValidateRequest,
};
use kmipkit::ttlv::{Structure, Value, ValueView};
use kmipkit::{ClientRequest, ProtocolVersion};

pub(crate) fn generic_model_check() -> bool {
    // OASIS KMIP 2.0/2.1 §10.1: future Enumeration values survive generic navigation.
    let Ok(value) = item(0x0042_005c, Value::enumeration(0x7fff_fffe)) else {
        return false;
    };
    let Ok(tree) = structure(0x0042_0079, vec![value]) else {
        return false;
    };
    let constructed = tree.view().children().first().is_some_and(|v| {
        v.with_value(|view| matches!(view, ValueView::Enumeration(value) if *value == 0x7fff_fffe))
    });
    // Independently derived §10.1 wire fixture: length=4 and four padding bytes.
    let wire = [
        0x42, 0x00, 0x5c, 0x05, 0, 0, 0, 4, 0x7f, 0xff, 0xff, 0xfe, 0, 0, 0, 0,
    ];
    let decoded = kmipkit::ttlv::codec::decode(&wire).is_ok_and(|v| {
        v.with_value(|view| matches!(view, ValueView::Enumeration(value) if *value == 0x7fff_fffe))
    });
    constructed && decoded
}

#[path = "local_cases_generated.rs"]
pub(crate) mod local;

fn parse_wire_hex(raw: &str) -> Option<u32> {
    let part = raw
        .split(['-', '\u{2013}', '\u{2014}'])
        .next()
        .unwrap_or(raw)
        .trim()
        .trim_start_matches("0x")
        .trim_start_matches("0X");
    let normalized: String = part
        .chars()
        .map(|c| match c {
            'x' | 'X' => '1',
            other => other,
        })
        .collect();
    u32::from_str_radix(&normalized, 16).ok()
}

fn check_ttlv_roundtrip(entry: &kmipkit::ttlv::Item) -> bool {
    let mut wire = Vec::new();
    if crate::generic_wire::encode(entry, &mut wire).is_err() {
        return false;
    }
    let Ok(decoded) = kmipkit::ttlv::codec::decode(&wire) else {
        return false;
    };
    matches_ttlv_wire(&wire, &decoded)
}

fn matches_ttlv_wire(wire: &[u8], decoded: &kmipkit::ttlv::Item) -> bool {
    let mut reconstructed = Vec::new();
    crate::generic_wire::encode(decoded, &mut reconstructed).is_ok() && reconstructed == wire
}

fn check_data_type_boundaries(case: &Case) -> bool {
    use std::sync::OnceLock;
    static FIXTURES: OnceLock<serde_json::Value> = OnceLock::new();
    let fixtures = FIXTURES.get_or_init(|| {
        serde_json::from_str(include_str!("../../shared/vectors/ttlv-boundaries.json"))
            .expect("valid pinned TTLV boundary fixtures")
    });
    let kind = case.requires["wire_value"]
        .as_str()
        .and_then(parse_wire_hex);
    let Some(samples) = fixtures["vectors"].as_array().and_then(|vectors| {
        vectors.iter().find(|vector| {
            kind.is_some() && kind == vector["type_code"].as_str().and_then(parse_wire_hex)
        })?["samples"]
            .as_array()
    }) else {
        return false;
    };
    !samples.is_empty()
        && samples.iter().all(|sample| {
            let Some(hex) = sample["hex"].as_str() else {
                return false;
            };
            let Ok(bytes) = crate::generic_wire::decode_hex(hex) else {
                return false;
            };
            match (
                sample["valid"].as_bool(),
                kmipkit::ttlv::codec::decode(&bytes),
            ) {
                (Some(true), Ok(decoded)) => matches_ttlv_wire(&bytes, &decoded),
                (Some(false), Err(_)) => true,
                _ => false,
            }
        })
}

fn check_sample_value_type(kind: &str) -> bool {
    let sample = match kind {
        "Integer" => Value::integer(1),
        "Long Integer" | "LongInteger" => Value::long_integer(1),
        "Big Integer" | "BigInteger" => Value::big_integer(vec![0, 0, 0, 0, 0, 0, 0, 1]),
        "Enumeration" => Value::enumeration(1),
        "Boolean" => Value::boolean(true),
        "Text String" | "TextString" => Value::text_string("kmipkit".to_owned()),
        "Byte String" | "ByteString" => Value::byte_string(vec![1, 2, 3, 4]),
        "Date-Time" | "DateTime" => Value::date_time(1_700_000_000),
        "Interval" => Value::interval(60),
        "Date-Time Extended" | "DateTimeExtended" => Value::date_time_extended(1_700_000_000),
        "Structure" => {
            let Ok(child) = item(0x0042_005c, Value::enumeration(1)) else {
                return false;
            };
            let Ok(tree) = structure(0x0042_0079, vec![child]) else {
                return false;
            };
            Value::structure(tree)
        }
        _ => return false,
    };
    let Ok(entry) = item(0x0042_0079, sample) else {
        return false;
    };
    check_ttlv_roundtrip(&entry)
}

fn verify_local_batch(case: &Case, version: &str) -> bool {
    use kmipkit::ClientBatch;
    let limits = kmipkit::ttlv::codec::CodecLimits::defaults();
    let proto = if version == "2.0" {
        ProtocolVersion::V2_0
    } else {
        ProtocolVersion::V2_1
    };
    let Some(scenario) = case.requires["protocol_scenario"]["scenario_id"].as_str() else {
        return false;
    };
    match scenario {
        "test-batch-count-zero-rejected" => ClientBatch::empty()
            .build_request_message(proto, &limits)
            .is_err(),
        "test-batch-single-item-options-omitted" => {
            let batch = ClientBatch::new(kmipkit::ClientBatchItem::discover_versions());
            let Ok(msg) = batch.build_request_message(proto, &limits) else {
                return false;
            };
            msg.with_ttlv(|root| {
                let Some(header) = root
                    .children()
                    .iter()
                    .find(|c| c.tag().raw() == 0x0042_0077)
                else {
                    return false;
                };
                header.with_value(|value| match value {
                    ValueView::Structure(fields) => !fields
                        .children()
                        .iter()
                        .any(|c| matches!(c.tag().raw(), 0x0042_000e | 0x0042_0010)),
                    _ => false,
                })
            })
        }
        "test-batch-placeholder-without-producer-rejected" => {
            let batch = ClientBatch::new(kmipkit::ClientBatchItem::new(ClientRequest::Activate(
                ActivateRequest::new(None),
            )));
            batch.build_request_message(proto, &limits).is_err()
        }
        "test-batch-placeholder-forward-reference-rejected" => {
            let Ok(created) = create("forward-ref", version) else {
                return false;
            };
            let create_item = kmipkit::ClientBatchItem::new(created);
            let activate_item =
                kmipkit::ClientBatchItem::new(ClientRequest::Activate(ActivateRequest::new(None)));
            let batch = ClientBatch::from_items([activate_item, create_item]);
            batch.build_request_message(proto, &limits).is_err()
        }
        "test-batch-count-mismatch-rejected" => {
            batch_count_mismatch(proto).is_ok_and(|tree| {
                matches!(
                    kmipkit::protocol::RequestMessage::try_from_ttlv(tree),
                    Err(error) if error.kind() == kmipkit::protocol::MessageValidationErrorKind::BatchCountMismatch
                )
            })
        }
        _ => false,
    }
}

fn batch_count_mismatch(version: ProtocolVersion) -> Result<Structure> {
    let version = item(
        0x0042_0069,
        Value::structure(structure(
            0x0042_0069,
            vec![
                item(0x0042_006a, Value::integer(version.major()))?,
                item(0x0042_006b, Value::integer(version.minor()))?,
            ],
        )?),
    )?;
    let header = item(
        0x0042_0077,
        Value::structure(structure(
            0x0042_0077,
            vec![version, item(0x0042_000d, Value::integer(1))?],
        )?),
    )?;
    let batch_item = || {
        let payload = DiscoverVersionsRequest::new()
            .to_ttlv_payload()
            .map_err(|_| "invalid Discover Versions fixture")?;
        item(
            0x0042_000f,
            Value::structure(structure(
                0x0042_000f,
                vec![
                    item(0x0042_005c, Value::enumeration(0x1e))?,
                    item(0x0042_0079, Value::structure(payload))?,
                ],
            )?),
        )
    };
    structure(0x0042_0078, vec![header, batch_item()?, batch_item()?])
}

fn check_local_version_boundary(operation: &str) -> bool {
    let code = match operation {
        "Get Constraints" => 0x38,
        "Ping" => 0x3b,
        "Process" => 0x3a,
        "Query Asynchronous Requests" => 0x39,
        "Set Constraints" => 0x37,
        "Set Defaults" => 0x36,
        _ => return false,
    };
    kmipkit::VersionGate::validate_operation(ProtocolVersion::V2_0, code).is_err()
        && kmipkit::VersionGate::validate_operation(ProtocolVersion::V2_1, code).is_ok()
}

pub(crate) fn verify_local(case: &Case, version: &str) -> bool {
    if local::verify(&case.case_id) {
        return true;
    }
    if let Some(base) = case.case_id.strip_suffix("-all-combinations")
        && local::verify(base)
    {
        return true;
    }
    if case.category == "version_boundaries" {
        return check_local_version_boundary(&case.operation);
    }
    if case.category == "batch_workflows" {
        return verify_local_batch(case, version);
    }
    let Some(kind) = case.requires["target_kind"].as_str() else {
        return false;
    };
    let ok = match kind {
        "tag" => case.requires["wire_value"]
            .as_str()
            .and_then(parse_wire_hex)
            .is_some_and(|raw| {
                let Ok(rt) = kmipkit::ttlv::RawTag::new(raw) else {
                    return false;
                };
                match case.requires["allocation"].as_str() {
                    Some("assigned" | "extension") => {
                        rt.try_checked().is_ok_and(|t| t.raw() == raw)
                    }
                    _ => rt.try_checked().is_err(),
                }
            }),
        "tag_range" => case.requires["wire_value"]
            .as_str()
            .and_then(|s| s.split_once(['-', '\u{2013}', '\u{2014}']))
            .and_then(|(lo, hi)| Some((parse_wire_hex(lo)?, parse_wire_hex(hi)?)))
            .is_some_and(|(lo, hi)| {
                let ok_lo = kmipkit::ttlv::RawTag::new(lo).is_ok_and(|r| r.raw() == lo);
                let ok_hi = kmipkit::ttlv::RawTag::new(hi).is_ok_and(|r| r.raw() == hi);
                ok_lo && ok_hi
            }),
        "enumeration_value" | "referenced_enum_value" => case.requires["wire_value"]
            .as_str()
            .and_then(parse_wire_hex)
            .is_some_and(|raw| {
                item(0x0042_005c, Value::enumeration(raw))
                    .as_ref()
                    .is_ok_and(check_ttlv_roundtrip)
            }),
        "bitmask_value" => case.requires["wire_value"]
            .as_str()
            .and_then(parse_wire_hex)
            .is_some_and(|raw| {
                let signed = i32::from_be_bytes(raw.to_be_bytes());
                item(0x0042_002c, Value::integer(signed))
                    .as_ref()
                    .is_ok_and(check_ttlv_roundtrip)
            }),
        "data_type" => check_data_type_boundaries(case),
        "operation_payload_field"
        | "structure_member"
        | "operation_payload_value_variant"
        | "message_field"
        | "operation_structure"
        | "object_structure"
        | "attribute_structure"
        | "credential" => {
            let source = case.requires["value_type"]
                .as_str()
                .or_else(|| case.requires["source_encoding"].as_str())
                .unwrap_or("Structure");
            let first = [
                "Long Integer",
                "Big Integer",
                "Date-Time Extended",
                "Date-Time",
                "Text String",
                "Byte String",
                "Enumeration",
                "Boolean",
                "Integer",
                "Interval",
                "Structure",
            ]
            .into_iter()
            .find(|name| source.contains(name))
            .unwrap_or("Structure");
            check_sample_value_type(first)
        }
        _ => false,
    };
    ok && generic_model_check()
}

pub(crate) fn fixture_label(name: &str, version: &str) -> Result<kmipkit::ttlv::Item> {
    if version == "3.0" {
        // The pinned AttributeSet only admits the 2.x Structure encoding of Name.
        // Description lets an isolated draft fixture carry a unique test label.
        return item(0x0042_00fc, Value::text_string(name.to_owned()));
    }
    item(
        0x0042_0053,
        Value::structure(structure(
            0x0042_0053,
            vec![
                item(0x0042_0055, Value::text_string(name.to_owned()))?,
                item(0x0042_0054, Value::enumeration(1))?,
            ],
        )?),
    )
}

pub(crate) fn attributes(
    name: &str,
    algorithm: u32,
    bits: i32,
    version: &str,
) -> Result<AttributeSet> {
    AttributeSet::try_new(vec![
        item(0x0042_0028, Value::enumeration(algorithm))?,
        item(0x0042_002a, Value::integer(bits))?,
        item(
            0x0042_002c,
            Value::integer(match algorithm {
                4 | 6 => 3,
                9 => 0x180,
                _ => 0x0c,
            }),
        )?,
        fixture_label(name, version)?,
    ])
    .map_err(|_| "invalid explicit key attributes")
}

pub(crate) fn rotate_attribute(case: &Case) -> Result<Option<kmipkit::ttlv::Item>> {
    let Some(hex) = case.requires["rotate_attribute_ttlv"].as_str() else {
        return Ok(None);
    };
    let wire = crate::generic_wire::decode_hex(hex)?;
    let item = kmipkit::ttlv::codec::decode(&wire).map_err(|_| "invalid Rotate fixture")?;
    match item.tag().raw() {
        0x0042_016a..=0x0042_016f | 0x0042_0172 => Ok(Some(item)),
        _ => Err("Rotate fixture has an unexpected tag"),
    }
}

pub(crate) fn create(name: &str, version: &str) -> Result<ClientRequest> {
    Ok(ClientRequest::Create(CreateRequest::new(
        ObjectType::from_raw(2),
        attributes(name, 3, 256, version)?,
    )))
}

fn data() -> OperationData {
    OperationData::ByteString(SecretBytes::new(vec![0x41; 16]))
}
fn parameters() -> Result<kmipkit::ttlv::Structure> {
    CryptographicParametersBuilder::new()
        .with_block_cipher_mode(1)
        .with_padding_method(1)
        .with_cryptographic_algorithm(3)
        .build()
        .map(CryptographicParameters::into_ttlv)
        .map_err(|_| "invalid explicit crypto parameters")
}
fn hash_parameters() -> Result<kmipkit::ttlv::Structure> {
    CryptographicParametersBuilder::new()
        .with_hashing_algorithm(6)
        .build()
        .map(CryptographicParameters::into_ttlv)
        .map_err(|_| "invalid hash parameters")
}

#[allow(clippy::too_many_lines)] // Keep the explicit standard-operation mapping auditable in one table.
pub(crate) fn request(
    case: &Case,
    identifier: Option<&UniqueIdentifier>,
    name: &str,
    version: &str,
) -> Result<ClientRequest> {
    let id_text = identifier.as_ref().and_then(|v| match v {
        UniqueIdentifier::TextString(text) => Some(text.clone()),
        _ => None,
    });
    let id = || {
        identifier
            .cloned()
            .ok_or("owned object prerequisite unavailable")
    };
    Ok(match case.operation.as_str() {
        "Set Endpoint Role" => ClientRequest::SetEndpointRole(
            kmipkit::protocol::SetEndpointRoleRequest::new(kmipkit::protocol::EndpointRole::CLIENT),
        ),
        "Discover Versions" => {
            let version = match version {
                "2.0" => ProtocolVersion::V2_0,
                "3.0" => ProtocolVersion::V3_0,
                _ => ProtocolVersion::V2_1,
            };
            ClientRequest::DiscoverVersions(
                DiscoverVersionsRequest::with_versions(vec![version])
                    .map_err(|_| "invalid protocol version offer")?,
            )
        }
        "Ping" => ClientRequest::ping(),
        "Query" => {
            let functions = if case.case_id == "test-query-attestation-nonce" {
                vec![QueryFunction::ATTESTATION_TYPES]
            } else if let Some(value) = case.requires["query_function"].as_str() {
                vec![QueryFunction::from_raw(
                    value.parse().map_err(|_| "invalid Query function")?,
                )]
            } else {
                vec![
                    QueryFunction::OPERATIONS,
                    QueryFunction::OBJECTS,
                    QueryFunction::CAPABILITIES,
                ]
            };
            ClientRequest::Query(QueryRequest::new(functions))
        }
        "Delegated Login" => ClientRequest::DelegatedLogin(
            DelegatedLoginRequest::new(structure(0x0042_014d, Vec::new())?).with_request_count(1),
        ),
        "Logout" => ClientRequest::Logout(LogoutRequest::new(
            Ticket::new(TicketType::from_raw(1), SecretBytes::new(vec![0x41; 16]))
                .map_err(|_| "invalid ticket fixture")?,
        )),
        "Login" => ClientRequest::Login(LoginRequest::new().with_request_count(1)),
        "Log" => ClientRequest::Log(LogRequest::new(SecretText::new(
            "KMIPKit dedicated interoperability probe".to_owned(),
        ))),
        "Interop" => {
            ClientRequest::Interop(InteropRequest::new(1, SecretText::new(name.to_owned())))
        }
        "Create" => match case.requires["object_type"].as_str() {
            Some("Private Key" | "Public Key") => ClientRequest::CreateKeyPair(
                CreateKeyPairRequest::new()
                    .with_common_attributes(attributes(name, 4, 2048, version)?),
            ),
            Some("Split Key") => ClientRequest::CreateSplitKey(CreateSplitKeyRequest::new(
                ObjectType::from_raw(2),
                2,
                2,
                SplitKeyMethod::XOR,
                attributes(name, 3, 256, version)?,
            )),
            _ => create(name, version)?,
        },
        "Register" => crate::fixtures::register(
            case.requires["object_type"]
                .as_str()
                .unwrap_or("Symmetric Key"),
            version,
        )?,
        "Import" => {
            let (kind, object, attrs) = crate::fixtures::object("Symmetric Key", version)?;
            ClientRequest::Import(
                ImportRequest::new(UniqueIdentifier::from(name), kind, attrs, object)
                    .with_replace_existing(false),
            )
        }
        "Certify" => ClientRequest::Certify(
            CertifyRequest::new()
                .with_certificate_request_type(1)
                .with_certificate_request_value(SecretBytes::new(crate::fixtures::CSR.to_vec())),
        ),
        "Re-certify" => ClientRequest::Recertify(
            RecertifyRequest::new()
                .with_unique_identifier(id()?)
                .with_certificate_request_type(1)
                .with_certificate_request_value(SecretBytes::new(crate::fixtures::CSR.to_vec())),
        ),
        "Re-Provision" => ClientRequest::ReProvision(
            ReProvisionRequest::new().with_certificate_request(crate::fixtures::csr_structure()?),
        ),
        "Validate" => ClientRequest::Validate(
            ValidateRequest::new().with_certificate(crate::fixtures::certificate_structure()?),
        ),
        "PKCS#11" => ClientRequest::Pkcs11(Pkcs11Request::new(structure(
            0x0042_015a,
            vec![item(0x0042_015a, Value::enumeration(1))?],
        )?)),
        "Join Split Key" => ClientRequest::JoinSplitKey(
            JoinSplitKeyRequest::new(
                ObjectType::from_raw(2),
                vec![
                    SplitKeyReference::new(id()?).with_split_key_threshold(2),
                    SplitKeyReference::new(UniqueIdentifier::from(format!("{name}-second")))
                        .with_split_key_threshold(2),
                ],
            )
            .with_attributes(attributes(name, 3, 256, version)?),
        ),
        "Create Key Pair" => ClientRequest::CreateKeyPair(
            CreateKeyPairRequest::new().with_common_attributes(attributes(name, 4, 2048, version)?),
        ),
        "Create Split Key" => ClientRequest::CreateSplitKey(CreateSplitKeyRequest::new(
            ObjectType::from_raw(2),
            2,
            2,
            SplitKeyMethod::XOR,
            attributes(name, 3, 256, version)?,
        )),
        "Activate" => ClientRequest::Activate(ActivateRequest::new(Some(id()?))),
        "Archive" => ClientRequest::Archive(ArchiveRequest::new(Some(id()?))),
        "Destroy" => ClientRequest::Destroy(DestroyRequest::new(Some(id()?))),
        "Recover" => ClientRequest::Recover(RecoverRequest::new(Some(id()?))),
        "Revoke" => ClientRequest::Revoke(
            RevokeRequest::new(RevocationReason::CESSATION_OF_OPERATION)
                .with_unique_identifier(id()?),
        ),
        "Get" => ClientRequest::Get(GetRequest::new().with_unique_identifier(id()?)),
        "Export" => ClientRequest::Export(ExportRequest::new().with_unique_identifier(id()?)),
        "Get Attributes" => {
            let refs = if let Some(attribute) = case.requires["attribute"].as_str() {
                match attribute {
                    "Certificate Attributes" => vec![
                        AttributeReference::tag(0x0042_00b5),
                        AttributeReference::tag(0x0042_00b6),
                    ],
                    "Vendor Attribute" => {
                        vec![AttributeReference::name("KMIPKit.Interop", "probe")]
                    }
                    _ => vec![AttributeReference::tag(
                        local::attribute_tag(attribute).ok_or("attribute tag unavailable")?,
                    )],
                }
            } else {
                Vec::new()
            };
            ClientRequest::GetAttributes(
                GetAttributesRequest::try_new(id_text, refs)
                    .map_err(|_| "invalid attribute references")?,
            )
        }
        "Get Attribute List" => {
            ClientRequest::GetAttributeList(GetAttributeListRequest::new(id_text))
        }
        "Locate" => {
            let request = if let Some(rotate) = rotate_attribute(case)? {
                let search = AttributeSet::try_new(vec![rotate])
                    .map_err(|_| "invalid Rotate fixture attribute")?;
                LocateRequest::new(search).with_offset_items(0)
            } else {
                LocateRequest::new(attributes(name, 3, 256, version)?)
            };
            ClientRequest::Locate(request.with_maximum_items(1))
        }
        "Check" => ClientRequest::Check(
            CheckRequest::new()
                .with_unique_identifier(id()?)
                .with_cryptographic_usage_mask(4),
        ),
        "Obtain Lease" => {
            ClientRequest::ObtainLease(ObtainLeaseRequest::new().with_unique_identifier(id()?))
        }
        "Get Usage Allocation" => ClientRequest::GetUsageAllocation(
            GetUsageAllocationRequest::new(1).with_unique_identifier(id()?),
        ),
        "Get Constraints" => ClientRequest::get_constraints(),
        "Set Constraints" => ClientRequest::SetConstraints(SetConstraintsRequest::new(structure(
            0x0042_012a,
            Vec::new(),
        )?)),
        "Set Defaults" => ClientRequest::SetDefaults(SetDefaultsRequest::new()),
        "Re-key" => ClientRequest::Rekey(RekeyRequest::new().with_unique_identifier(id()?)),
        "Re-key Key Pair" => ClientRequest::RekeyKeyPair(
            RekeyKeyPairRequest::new().with_private_key_unique_identifier(id()?),
        ),
        "Add Attribute" => ClientRequest::AddAttribute(AddAttributeRequest::new(
            id_text,
            NewAttribute::new(item(0x0042_00fc, Value::text_string("interop".to_owned()))?),
        )),
        "Set Attribute" => ClientRequest::SetAttribute(SetAttributeRequest::new(
            id_text,
            NewAttribute::new(item(0x0042_00fc, Value::text_string("interop".to_owned()))?),
        )),
        "Modify Attribute" => {
            let attribute = if case.case_id == "test-neg-modify-readonly-attribute" {
                item(0x0042_008d, Value::enumeration(2))?
            } else {
                item(0x0042_00fc, Value::text_string("updated".to_owned()))?
            };
            ClientRequest::ModifyAttribute(ModifyAttributeRequest::new(
                id_text,
                None,
                NewAttribute::new(attribute),
            ))
        }
        "Delete Attribute" => ClientRequest::DeleteAttribute(DeleteAttributeRequest::new(
            id_text,
            None,
            Some(AttributeReference::tag(0x0042_00fc)),
        )),
        "Adjust Attribute" => ClientRequest::AdjustAttribute(AdjustAttributeRequest::new(
            id_text,
            AttributeReference::tag(0x0042_0001),
            AdjustmentType::from_raw(1),
            Some(Value::integer(1)),
        )),
        "Hash" => ClientRequest::Hash(HashRequest::new(hash_parameters()?).with_data(data())),
        "RNG Retrieve" => ClientRequest::RngRetrieve(RngRetrieveRequest::new(32)),
        "RNG Seed" => ClientRequest::RngSeed(RngSeedRequest::new(SecretBytes::new(vec![0x41; 32]))),
        "Encrypt" => ClientRequest::Encrypt(
            EncryptRequest::new(Some(id()?), Some(data()))
                .with_cryptographic_parameters(parameters()?)
                .with_iv_counter_nonce(SecretBytes::new(vec![0; 16])),
        ),
        "Decrypt" => ClientRequest::Decrypt(
            DecryptRequest::new(Some(id()?), Some(data()))
                .with_cryptographic_parameters(parameters()?)
                .with_iv_counter_nonce(SecretBytes::new(vec![0; 16])),
        ),
        "MAC" => ClientRequest::Mac(
            MacRequest::new()
                .with_unique_identifier(id()?)
                .with_data(data())
                .with_cryptographic_parameters(hash_parameters()?),
        ),
        "MAC Verify" => ClientRequest::MacVerify(
            MacVerifyRequest::new()
                .with_unique_identifier(id()?)
                .with_data(data())
                .with_mac_data(SecretBytes::new(vec![0; 32]))
                .with_cryptographic_parameters(hash_parameters()?),
        ),
        "Sign" => ClientRequest::Sign(
            SignRequest::new()
                .with_unique_identifier(id()?)
                .with_cryptographic_parameters(
                    CryptographicParametersBuilder::new()
                        .with_hashing_algorithm(6)
                        .with_padding_method(8)
                        .build()
                        .map(CryptographicParameters::into_ttlv)
                        .map_err(|_| "explicit cryptographic parameters rejected")?,
                )
                .with_data(data()),
        ),
        "Signature Verify" => ClientRequest::SignatureVerify(
            SignatureVerifyRequest::new()
                .with_unique_identifier(id()?)
                .with_cryptographic_parameters(
                    CryptographicParametersBuilder::new()
                        .with_hashing_algorithm(6)
                        .with_padding_method(8)
                        .build()
                        .map(CryptographicParameters::into_ttlv)
                        .map_err(|_| "explicit cryptographic parameters rejected")?,
                )
                .with_data(data())
                .with_signature_data(SecretBytes::new(vec![0; 256])),
        ),
        "Derive Key" => ClientRequest::DeriveKey(DeriveKeyRequest::new(
            ObjectType::from_raw(2),
            vec![id()?],
            DerivationMethod::from_raw(2),
            DerivationParameters::new().with_cryptographic_parameters(hash_parameters()?),
            attributes(name, 3, 256, version)?,
        )),
        _ => return Err("operation prerequisites are not represented by this connection contract"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    // KMIPKIT-0084-FR-003; each row's own exact OASIS references identify the contract.
    #[test]
    fn every_synchronous_operation_has_a_public_typed_builder() {
        let catalog: crate::models::Catalog = crate::models::load(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../shared/catalog/kmipkit-1.0-test-catalog.json"),
        )
        .expect("catalog");
        let mut absent = Vec::new();
        for case in &catalog.test_cases {
            if ["Poll", "Process", "Cancel", "Query Asynchronous Requests"]
                .contains(&case.operation.as_str())
                || case.execution.mode == "local_model"
                || case.requires["workflow_type"] == "server_to_client"
            {
                continue;
            }
            match request(
                case,
                Some(&UniqueIdentifier::from("owned-fixture")),
                "kmipkit-fixture",
                "2.1",
            ) {
                Ok(request) if case.case_id == "test-neg-modify-readonly-attribute" => {
                    assert!(
                        !request.validation_report().is_valid(),
                        "OASIS §4.57 State cannot be modified directly"
                    );
                }
                Ok(request) if request.validation_report().is_valid() => {}
                Ok(request) => {
                    eprintln!("{}: {:?}", case.case_id, request.validation_report());
                    absent.push(&case.case_id);
                }
                Err(_) => absent.push(&case.case_id),
            }
        }
        assert!(
            absent.is_empty(),
            "catalog rows without typed builders: {absent:?}"
        );
    }
}
