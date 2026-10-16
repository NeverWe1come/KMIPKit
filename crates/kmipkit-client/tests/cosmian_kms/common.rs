//! Shared, sanitized helpers for case-isolated Cosmian lifecycle tests.

use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

use kmipkit_client::{Client, ClientBatch, ClientBatchItem, ClientRequest, ClientResponseView};
use kmipkit_protocol::{
    ActivateRequest, AddAttributeRequest, AttributeReference, AttributeSet, CheckRequest,
    CreateKeyPairRequest, CreateRequest, CreateSplitKeyRequest, CurrentAttribute,
    DeleteAttributeRequest, DestroyRequest, GetAttributeListRequest, GetAttributesRequest,
    LocateRequest, ModifyAttributeRequest, NewAttribute, ObjectType, OpaqueObjectContext,
    RegisterRequest, RevocationReason, RevokeRequest, SecretBytes, SetAttributeRequest,
    SplitKeyMethod, UniqueIdentifier, ValidateRequest,
};
use kmipkit_ttlv::{Item, Structure, Value};

use super::{
    AES, CRYPTOGRAPHIC_ALGORITHM, CRYPTOGRAPHIC_LENGTH, CRYPTOGRAPHIC_USAGE_MASK, RSA, SUCCESS,
    certificate_directory, client, comment_item, kmip_item, structure, text_identifier,
};

const NAME: u32 = 0x0042_0053;
const NAME_TYPE: u32 = 0x0042_0054;
const NAME_VALUE: u32 = 0x0042_0055;
const CERTIFICATE: u32 = 0x0042_0013;
const CERTIFICATE_TYPE: u32 = 0x0042_001D;
const CERTIFICATE_VALUE: u32 = 0x0042_001E;
const CERTIFICATE_LENGTH: u32 = 0x0042_00AD;
const OPAQUE_OBJECT: u32 = 0x0042_005B;
const OPAQUE_DATA_TYPE: u32 = 0x0042_0059;
const OPAQUE_DATA_VALUE: u32 = 0x0042_005A;
const SECRET_DATA: u32 = 0x0042_0085;
const SECRET_DATA_TYPE: u32 = 0x0042_0086;
const KEY_BLOCK: u32 = 0x0042_0040;
const KEY_FORMAT_TYPE: u32 = 0x0042_0042;
const KEY_VALUE: u32 = 0x0042_0045;
const KEY_MATERIAL: u32 = 0x0042_0043;
const ACTIVATION_DATE: u32 = 0x0042_0001;
const STATE: u32 = 0x0042_0091;
const COMMENT: u32 = 0x0042_00FD;

struct OwnedObjects {
    primary: String,
    _linked: Vec<String>,
}

/// Executes one manifest case using a fresh mTLS client and fresh server object.
pub(super) fn run_lifecycle_case(case_id: &str) -> Result<(), Box<dyn Error>> {
    let row = manifest_row(case_id)?;
    let operation = required_text(&row, "operation")?;
    let object_type = required_text(&row, "object_type")?;
    let mut client = client()?;

    if operation == "Validate" {
        return run_validate_case(&mut client, case_id, &row);
    }

    let objects = provision_object(&mut client, object_type, case_id)?;
    run_lifecycle_operation(&mut client, case_id, operation, object_type, &objects)?;

    // Split-key setup can create a linked source plus multiple parts. Its IDs
    // stay private to this case; Compose teardown disposes the temporary DB.
    println!("case_id={case_id} classification=typed_success");
    Ok(())
}

fn run_lifecycle_operation(
    client: &mut Client,
    case_id: &str,
    operation: &str,
    object_type: &str,
    objects: &OwnedObjects,
) -> Result<(), Box<dyn Error>> {
    match operation {
        "Activate" => run_activate(client, case_id, object_type, &objects.primary),
        "Add Attribute" => run_add_attribute(client, case_id, &objects.primary),
        "Delete Attribute" => run_delete_attribute(client, case_id, &objects.primary),
        "Get Attribute List" => run_get_attribute_list(client, case_id, &objects.primary),
        "Get Attributes" => run_get_attributes(client, case_id, &objects.primary),
        "Locate" => run_locate(client, case_id, object_type, &objects.primary),
        "Check" => run_check(client, case_id, &objects.primary),
        "Destroy" => run_destroy(client, case_id, object_type, &objects.primary),
        "Revoke" => run_revoke(client, case_id, object_type, &objects.primary),
        "Modify Attribute" => run_modify_attribute(client, case_id, &objects.primary),
        "Set Attribute" => run_set_attribute(client, case_id, &objects.primary),
        unsupported => {
            Err(format!("case {case_id} has operation outside Task 4: {unsupported}").into())
        }
    }
}

fn run_activate(
    client: &mut Client,
    case_id: &str,
    object_type: &str,
    unique_id: &str,
) -> Result<(), Box<dyn Error>> {
    if object_type == "SplitKey" {
        let state = get_object_state(client, case_id, unique_id)?;
        // OASIS KMIP v2.1 §6.1.1 only activates Pre-Active objects;
        // §11.55, Table 486 assigns Pre-Active the raw value 1.
        if state != 1 {
            return Err(format!(
                "case {case_id} classification=client_blocked; Create Split Key returned server-managed State {state}, while KMIP Activate requires Pre-Active"
            )
            .into());
        }
    }
    execute_case(
        client,
        case_id,
        ClientRequest::activate(ActivateRequest::new(Some(identifier(unique_id)))),
        "Activate",
        |response| {
            require_identifier(
                response
                    .activate()
                    .and_then(|result| result.unique_identifier()),
                "Activate",
            )
        },
    )?;
    Ok(())
}

fn run_add_attribute(
    client: &mut Client,
    case_id: &str,
    identifier: &str,
) -> Result<(), Box<dyn Error>> {
    execute_case(
        client,
        case_id,
        ClientRequest::add_attribute(AddAttributeRequest::new(
            Some(identifier.to_owned()),
            NewAttribute::new(comment_item(case_id)?),
        )),
        "Add Attribute",
        |response| {
            require_text_identifier(
                response
                    .add_attribute()
                    .and_then(|result| result.unique_identifier()),
                "Add Attribute",
            )
        },
    )?;
    Ok(())
}

fn run_delete_attribute(
    client: &mut Client,
    case_id: &str,
    identifier: &str,
) -> Result<(), Box<dyn Error>> {
    add_case_comment(client, case_id, identifier)?;
    execute_case(
        client,
        case_id,
        ClientRequest::delete_attribute(DeleteAttributeRequest::new(
            Some(identifier.to_owned()),
            None,
            Some(AttributeReference::tag(COMMENT)),
        )),
        "Delete Attribute",
        |response| {
            require_text_identifier(
                response
                    .delete_attribute()
                    .and_then(|result| result.unique_identifier()),
                "Delete Attribute",
            )
        },
    )?;
    Ok(())
}

fn run_get_attribute_list(
    client: &mut Client,
    case_id: &str,
    identifier: &str,
) -> Result<(), Box<dyn Error>> {
    execute_case(
        client,
        case_id,
        ClientRequest::get_attribute_list(GetAttributeListRequest::new(Some(
            identifier.to_owned(),
        ))),
        "Get Attribute List",
        |response| {
            let references = response
                .get_attribute_list()
                .and_then(|result| result.attribute_references())
                .ok_or_else(|| {
                    "successful Get Attribute List has no Attribute References".to_owned()
                })?;
            if references.is_empty() {
                return Err(
                    "Get Attribute List returned an empty Attribute References list".to_owned(),
                );
            }
            Ok(())
        },
    )?;
    Ok(())
}

fn run_get_attributes(
    client: &mut Client,
    case_id: &str,
    identifier: &str,
) -> Result<(), Box<dyn Error>> {
    add_case_comment(client, case_id, identifier)?;
    execute_case(
        client,
        case_id,
        ClientRequest::get_attributes(GetAttributesRequest::try_new(
            Some(identifier.to_owned()),
            [AttributeReference::tag(COMMENT)],
        )?),
        "Get Attributes",
        |response| {
            let attributes = response
                .get_attributes()
                .and_then(|result| result.attributes())
                .ok_or_else(|| "successful Get Attributes has no Attributes".to_owned())?;
            if !attributes
                .as_items()
                .iter()
                .any(|item| item.tag().raw() == COMMENT)
            {
                return Err("Get Attributes omitted the requested Comment".to_owned());
            }
            Ok(())
        },
    )?;
    Ok(())
}

fn run_locate(
    client: &mut Client,
    case_id: &str,
    object_type: &str,
    identifier: &str,
) -> Result<(), Box<dyn Error>> {
    let filter = if object_type == "SplitKey" {
        AttributeSet::try_new([])?
    } else {
        AttributeSet::try_new([name_attribute(case_id)?])?
    };
    execute_case(
        client,
        case_id,
        ClientRequest::locate(LocateRequest::new(filter).with_maximum_items(100)),
        "Locate",
        |response| {
            let located = response
                .locate()
                .ok_or_else(|| "KMIPKit did not return a typed Locate result".to_owned())?;
            if located.located_items().unwrap_or_default() <= 0
                || !located.unique_identifiers().iter().any(
                    |found| matches!(found, UniqueIdentifier::TextString(value) if value == identifier),
                )
            {
                return Err(format!(
                    "Locate returned {} items but did not return the case-owned object identifier",
                    located.located_items().unwrap_or_default()
                ));
            }
            Ok(())
        },
    )?;
    Ok(())
}

fn run_check(client: &mut Client, case_id: &str, unique_id: &str) -> Result<(), Box<dyn Error>> {
    let request = CheckRequest::new().with_unique_identifier(identifier(unique_id));
    execute_case(
        client,
        case_id,
        ClientRequest::check(request),
        "Check",
        |response| {
            require_identifier(
                response
                    .check()
                    .and_then(|result| result.unique_identifier()),
                "Check",
            )
        },
    )?;
    Ok(())
}

fn run_destroy(
    client: &mut Client,
    case_id: &str,
    object_type: &str,
    unique_id: &str,
) -> Result<(), Box<dyn Error>> {
    if object_type == "SplitKey" {
        revoke_for_destroy(client, case_id, unique_id)?;
    }
    execute_case(
        client,
        case_id,
        ClientRequest::destroy(DestroyRequest::new(Some(identifier(unique_id)))),
        "Destroy",
        |response| {
            require_identifier(
                response
                    .destroy()
                    .and_then(|result| result.unique_identifier()),
                "Destroy",
            )
        },
    )?;
    Ok(())
}

fn run_revoke(
    client: &mut Client,
    case_id: &str,
    object_type: &str,
    unique_id: &str,
) -> Result<(), Box<dyn Error>> {
    if object_type != "OpaqueObject" && object_type != "SplitKey" {
        activate_for_revoke(client, case_id, unique_id)?;
    }
    execute_case(
        client,
        case_id,
        ClientRequest::revoke(
            RevokeRequest::new(RevocationReason::CESSATION_OF_OPERATION)
                .with_unique_identifier(identifier(unique_id)),
        ),
        "Revoke",
        |response| {
            require_identifier(
                response
                    .revoke()
                    .and_then(|result| result.unique_identifier()),
                "Revoke",
            )
        },
    )?;
    Ok(())
}

fn run_modify_attribute(
    client: &mut Client,
    case_id: &str,
    identifier: &str,
) -> Result<(), Box<dyn Error>> {
    if case_id.ends_with("activation-date") {
        modify_activation_date(client, case_id, identifier)?;
    } else {
        add_case_comment(client, case_id, identifier)?;
        execute_case(
            client,
            case_id,
            ClientRequest::modify_attribute(ModifyAttributeRequest::new(
                Some(identifier.to_owned()),
                Some(CurrentAttribute::new(comment_item(case_id)?)),
                NewAttribute::new(comment_item(&format!("{case_id}-updated"))?),
            )),
            "Modify Attribute",
            |response| {
                require_text_identifier(
                    response
                        .modify_attribute()
                        .and_then(|result| result.unique_identifier()),
                    "Modify Attribute",
                )
            },
        )?;
    }
    Ok(())
}

fn run_set_attribute(
    client: &mut Client,
    case_id: &str,
    identifier: &str,
) -> Result<(), Box<dyn Error>> {
    if case_id.ends_with("activation-date") {
        set_activation_date(client, case_id, identifier)?;
    } else {
        execute_case(
            client,
            case_id,
            ClientRequest::set_attribute(SetAttributeRequest::new(
                Some(identifier.to_owned()),
                NewAttribute::new(comment_item(case_id)?),
            )),
            "Set Attribute",
            |response| {
                require_text_identifier(
                    response
                        .set_attribute()
                        .and_then(|result| result.unique_identifier()),
                    "Set Attribute",
                )
            },
        )?;
    }
    Ok(())
}

fn execute_case<T>(
    client: &mut Client,
    case_id: &str,
    request: ClientRequest,
    operation: &str,
    validate: impl FnOnce(ClientResponseView<'_>) -> Result<T, String>,
) -> Result<T, Box<dyn Error>> {
    let response = client
        .execute(
            ClientBatch::new(ClientBatchItem::new(request)),
            &kmipkit_ttlv::codec::CodecLimits::defaults(),
        )
        .map_err(|error| {
            if let Some((wire_operation, status, reason)) = super::response_result_fields(&error)
            {
                format!(
                    "case {case_id} {operation}: typed response rejected by KMIPKit; wire operation {wire_operation:?}, Result Status {status:?}, Result Reason {reason:?}"
                )
            } else {
                format!(
                    "case {case_id} {operation}: client_blocked before a usable response (category={:?}, cause={:?}, delivery={:?})",
                    error.category(),
                    error.cause_category(),
                    error.delivery_state()
                )
            }
        })?;
    if response.len() != 1 {
        return Err(format!(
            "case {case_id} {operation}: expected one response item, received {}",
            response.len()
        )
        .into());
    }
    let item = response
        .get(0)
        .ok_or_else(|| format!("case {case_id} {operation}: response item missing"))?;
    let outcome = item.outcome();
    let status = outcome.result().status();
    let reason = outcome.result().reason().map(|reason| {
        (
            reason.raw(),
            reason.known_name().unwrap_or("unknown KMIP reason"),
        )
    });
    if status.raw() != SUCCESS {
        return Err(format!(
            "case {case_id} {operation}: KMIP Result Status {} ({}) Reason {:?}",
            status.raw(),
            status.known_name().unwrap_or("unknown"),
            reason
        )
        .into());
    }
    validate(outcome.response()).map_err(|message| -> Box<dyn Error> {
        format!("case {case_id} {operation}: {message}").into()
    })
}

fn provision_object(
    client: &mut Client,
    object_type: &str,
    case_id: &str,
) -> Result<OwnedObjects, Box<dyn Error>> {
    let (primary, linked, _raw_type) = match object_type {
        "SymmetricKey" => {
            let attrs = AttributeSet::try_new([
                kmip_item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(AES))?,
                kmip_item(CRYPTOGRAPHIC_LENGTH, Value::integer(256))?,
                kmip_item(CRYPTOGRAPHIC_USAGE_MASK, Value::integer(12))?,
                name_attribute(case_id)?,
            ])?;
            let id = create_result_id(
                client,
                case_id,
                ClientRequest::Create(CreateRequest::new(ObjectType::from_raw(2), attrs)),
                "Create",
            )?;
            (id, Vec::new(), 2)
        }
        "PrivateKey" | "PublicKey" => {
            let common = AttributeSet::try_new([
                kmip_item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(RSA))?,
                kmip_item(CRYPTOGRAPHIC_LENGTH, Value::integer(2048))?,
                name_attribute(case_id)?,
            ])?;
            let request = CreateKeyPairRequest::new()
                .with_common_attributes(common)
                .with_private_key_attributes(AttributeSet::try_new([kmip_item(
                    CRYPTOGRAPHIC_USAGE_MASK,
                    Value::integer(1),
                )?])?)
                .with_public_key_attributes(AttributeSet::try_new([kmip_item(
                    CRYPTOGRAPHIC_USAGE_MASK,
                    Value::integer(2),
                )?])?);
            let (private_id, public_id) = execute_created_key_pair(client, case_id, request)?;
            if object_type == "PrivateKey" {
                (private_id.clone(), vec![public_id], 4)
            } else {
                (public_id.clone(), vec![private_id], 3)
            }
        }
        "SplitKey" => {
            let source_id = create_result_id(
                client,
                case_id,
                ClientRequest::Create(CreateRequest::new(
                    ObjectType::from_raw(2),
                    AttributeSet::try_new([
                        kmip_item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(AES))?,
                        kmip_item(CRYPTOGRAPHIC_LENGTH, Value::integer(256))?,
                        kmip_item(CRYPTOGRAPHIC_USAGE_MASK, Value::integer(12))?,
                        name_attribute(&format!("{case_id}-source"))?,
                    ])?,
                )),
                "Create split source",
            )?;
            let split = CreateSplitKeyRequest::new(
                ObjectType::from_raw(2),
                2,
                2,
                SplitKeyMethod::XOR,
                AttributeSet::try_new([name_attribute(case_id)?])?,
            )
            .with_unique_identifier(identifier(&source_id));
            let ids = execute_created_split(client, case_id, split)?;
            let primary = ids.first().cloned().ok_or_else(|| {
                format!("case {case_id}: Create Split Key returned no identifiers")
            })?;
            let mut linked = ids.into_iter().skip(1).collect::<Vec<_>>();
            linked.push(source_id);
            (primary, linked, 5)
        }
        "Certificate" => (register_certificate(client, case_id)?, Vec::new(), 1),
        "SecretData" => (register_secret_data(client, case_id)?, Vec::new(), 7),
        "OpaqueObject" => (register_opaque_object(client, case_id)?, Vec::new(), 8),
        unknown => return Err(format!("case {case_id}: unsupported object type {unknown}").into()),
    };
    Ok(OwnedObjects {
        primary,
        _linked: linked,
    })
}

fn create_result_id(
    client: &mut Client,
    case_id: &str,
    request: ClientRequest,
    operation: &str,
) -> Result<String, Box<dyn Error>> {
    execute_case(client, case_id, request, operation, |response| {
        let id = response
            .create()
            .and_then(|result| result.unique_identifier())
            .ok_or_else(|| "successful Create response has no Unique Identifier".to_owned())?;
        text_identifier(id)
            .map_err(|_| "Create response identifier is not a text identifier".to_owned())
    })
}

fn execute_created_key_pair(
    client: &mut Client,
    case_id: &str,
    request: CreateKeyPairRequest,
) -> Result<(String, String), Box<dyn Error>> {
    execute_case(
        client,
        case_id,
        ClientRequest::CreateKeyPair(request),
        "Create Key Pair",
        |response| {
            let pair = response.create_key_pair().ok_or_else(|| {
                "successful Create Key Pair response has no typed result".to_owned()
            })?;
            let private_id = pair
                .private_key_unique_identifier()
                .ok_or_else(|| "Create Key Pair omitted the Private Key identifier".to_owned())?;
            let public_id = pair
                .public_key_unique_identifier()
                .ok_or_else(|| "Create Key Pair omitted the Public Key identifier".to_owned())?;
            Ok((
                text_identifier(private_id)
                    .map_err(|_| "Private Key identifier is not text".to_owned())?,
                text_identifier(public_id)
                    .map_err(|_| "Public Key identifier is not text".to_owned())?,
            ))
        },
    )
}

fn execute_created_split(
    client: &mut Client,
    case_id: &str,
    request: CreateSplitKeyRequest,
) -> Result<Vec<String>, Box<dyn Error>> {
    execute_case(
        client,
        case_id,
        ClientRequest::CreateSplitKey(request),
        "Create Split Key",
        |response| {
            let result = response.create_split_key().ok_or_else(|| {
                "successful Create Split Key response has no typed result".to_owned()
            })?;
            if result.unique_identifiers().is_empty() {
                return Err("Create Split Key returned no Unique Identifiers".to_owned());
            }
            result
                .unique_identifiers()
                .iter()
                .map(|id| {
                    text_identifier(id).map_err(|_| "Split Key identifier is not text".to_owned())
                })
                .collect()
        },
    )
}

fn register_certificate(client: &mut Client, case_id: &str) -> Result<String, Box<dyn Error>> {
    let der = read_pem_der(&certificate_directory().join("validation/leaf.crt"))?;
    let attributes = AttributeSet::try_new([
        kmip_item(
            CERTIFICATE_LENGTH,
            Value::integer(i32::try_from(der.len())?),
        )?,
        name_attribute(case_id)?,
    ])?;
    let object = kmip_item(
        CERTIFICATE,
        Value::structure(structure([
            kmip_item(CERTIFICATE_TYPE, Value::enumeration(1))?,
            kmip_item(CERTIFICATE_VALUE, Value::byte_string(der))?,
        ])?),
    )?;
    register_result_id(
        client,
        case_id,
        RegisterRequest::new(ObjectType::from_raw(1), attributes, object),
    )
}

fn register_secret_data(client: &mut Client, case_id: &str) -> Result<String, Box<dyn Error>> {
    let material = SecretBytes::new(case_id.as_bytes().to_vec());
    let attributes = AttributeSet::try_new([
        kmip_item(CRYPTOGRAPHIC_USAGE_MASK, Value::integer(12))?,
        name_attribute(case_id)?,
    ])?;
    let object = kmip_item(
        SECRET_DATA,
        Value::structure(structure([
            kmip_item(SECRET_DATA_TYPE, Value::enumeration(1))?,
            kmip_item(
                KEY_BLOCK,
                Value::structure(structure([
                    kmip_item(KEY_FORMAT_TYPE, Value::enumeration(1))?,
                    kmip_item(
                        KEY_VALUE,
                        Value::structure(structure([kmip_item(
                            KEY_MATERIAL,
                            Value::byte_string(material.with_bytes(<[u8]>::to_vec)),
                        )?])?),
                    )?,
                ])?),
            )?,
        ])?),
    )?;
    register_result_id(
        client,
        case_id,
        RegisterRequest::new(ObjectType::from_raw(7), attributes, object),
    )
}

fn register_opaque_object(client: &mut Client, case_id: &str) -> Result<String, Box<dyn Error>> {
    // OASIS KMIP v2.1 §2.3, Table 4 requires Opaque Data Type as an
    // Enumeration; §11.35, Table 469 assigns the 8XXXXXXX extension range.
    // Name remains valid here: §4.32, Table 94 applies it to All Objects.
    let object = kmip_item(
        OPAQUE_OBJECT,
        Value::structure(structure([
            kmip_item(OPAQUE_DATA_TYPE, Value::enumeration(0x8000_0001))?,
            kmip_item(
                OPAQUE_DATA_VALUE,
                Value::byte_string(case_id.as_bytes().to_vec()),
            )?,
        ])?),
    )?;
    register_result_id(
        client,
        case_id,
        RegisterRequest::new(
            ObjectType::from_raw(8),
            AttributeSet::try_new([name_attribute(case_id)?])?,
            object,
        )
        .with_opaque_object_context(OpaqueObjectContext::Generic),
    )
}

fn register_result_id(
    client: &mut Client,
    case_id: &str,
    request: RegisterRequest,
) -> Result<String, Box<dyn Error>> {
    let request = ClientRequest::register(request);
    execute_case(client, case_id, request, "Register setup", |response| {
        let id = response
            .register()
            .and_then(|result| result.unique_identifier())
            .ok_or_else(|| "successful Register response has no Unique Identifier".to_owned())?;
        text_identifier(id).map_err(|_| "Register response identifier is not text".to_owned())
    })
}

fn run_validate_case(
    client: &mut Client,
    case_id: &str,
    row: &serde_json::Value,
) -> Result<(), Box<dyn Error>> {
    let params = &row["parameters"];
    let mut request = ValidateRequest::new();
    let root_der = read_pem_der(&certificate_directory().join("ca/ca.crt"))?;
    let intermediate_der =
        read_pem_der(&certificate_directory().join("validation/intermediate.crt"))?;
    let leaf_der = read_pem_der(&certificate_directory().join("validation/leaf.crt"))?;

    let mut managed = Vec::new();
    if params["chain_input"] == "managed-identifiers" || params["chain_input"] == "mixed" {
        managed.push(register_certificate(client, &format!("{case_id}-leaf"))?);
        managed.push(register_certificate(
            client,
            &format!("{case_id}-intermediate"),
        )?);
        managed.push(register_certificate(client, &format!("{case_id}-root"))?);
    }
    match params["chain_input"].as_str() {
        Some("certificate-values") => {
            for der in [&leaf_der, &intermediate_der, &root_der] {
                request = request.with_certificate(certificate_structure(der)?);
            }
        }
        Some("managed-identifiers") => {
            for identifier in managed {
                request = request.with_unique_identifier(UniqueIdentifier::TextString(identifier));
            }
        }
        Some("mixed") => {
            request = request
                .with_certificate(certificate_structure(&leaf_der)?)
                .with_unique_identifier(UniqueIdentifier::TextString(managed[1].clone()))
                .with_unique_identifier(UniqueIdentifier::TextString(managed[2].clone()));
        }
        _ => return Err(format!("case {case_id}: unknown Validate chain_input").into()),
    }
    if params["validity_time"] == "explicit-time" {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        request = request.with_validity_date(i64::try_from(now)?);
    }
    execute_case(
        client,
        case_id,
        ClientRequest::validate(request),
        "Validate",
        |response| {
            let result = response
                .validate()
                .ok_or_else(|| "KMIPKit did not return a typed Validate result".to_owned())?;
            if result.validity_indicator().is_none() {
                return Err("successful Validate response omitted Validity Indicator".to_owned());
            }
            Ok(())
        },
    )?;
    println!("case_id={case_id} classification=typed_success");
    Ok(())
}

fn certificate_structure(der: &[u8]) -> Result<Structure, Box<dyn Error>> {
    structure([
        kmip_item(CERTIFICATE_TYPE, Value::enumeration(1))?,
        kmip_item(CERTIFICATE_VALUE, Value::byte_string(der.to_vec()))?,
    ])
}

fn read_pem_der(path: &std::path::Path) -> Result<Vec<u8>, Box<dyn Error>> {
    let pem = std::fs::read_to_string(path)?;
    let mut decoded = Vec::new();
    let mut accumulator = 0_u32;
    let mut bits = 0_u8;
    for line in pem.lines().filter(|line| !line.starts_with("-----")) {
        for byte in line.bytes() {
            if byte == b'=' {
                break;
            }
            let value = match byte {
                b'A'..=b'Z' => byte - b'A',
                b'a'..=b'z' => byte - b'a' + 26,
                b'0'..=b'9' => byte - b'0' + 52,
                b'+' => 62,
                b'/' => 63,
                _ => continue,
            };
            accumulator = (accumulator << 6) | u32::from(value);
            bits += 6;
            if bits >= 8 {
                bits -= 8;
                decoded.push(((accumulator >> bits) & 0xff) as u8);
            }
        }
    }
    if decoded.is_empty() {
        return Err("local validation certificate PEM contains no DER data".into());
    }
    Ok(decoded)
}

fn add_case_comment(
    client: &mut Client,
    case_id: &str,
    identifier: &str,
) -> Result<(), Box<dyn Error>> {
    let object_type = required_text(&manifest_row(case_id)?, "object_type")?.to_owned();
    if object_type == "SplitKey" {
        return execute_case(
            client,
            case_id,
            ClientRequest::set_attribute(SetAttributeRequest::new(
                Some(identifier.to_owned()),
                NewAttribute::new(comment_item(case_id)?),
            )),
            "Set Attribute setup for comment",
            |response| {
                require_text_identifier(
                    response
                        .set_attribute()
                        .and_then(|result| result.unique_identifier()),
                    "Set Attribute setup",
                )
            },
        );
    }
    execute_case(
        client,
        case_id,
        ClientRequest::add_attribute(AddAttributeRequest::new(
            Some(identifier.to_owned()),
            NewAttribute::new(comment_item(case_id)?),
        )),
        "Add Attribute setup",
        |response| {
            require_text_identifier(
                response.add_attribute().and_then(|r| r.unique_identifier()),
                "Add Attribute setup",
            )
        },
    )
}

fn get_object_state(
    client: &mut Client,
    case_id: &str,
    identifier: &str,
) -> Result<u32, Box<dyn Error>> {
    execute_case(
        client,
        case_id,
        ClientRequest::get_attributes(GetAttributesRequest::try_new(
            Some(identifier.to_owned()),
            [AttributeReference::tag(STATE)],
        )?),
        "Get Attributes state preflight",
        |response| {
            let attributes = response
                .get_attributes()
                .and_then(|result| result.attributes())
                .ok_or_else(|| "State preflight returned no Attributes".to_owned())?;
            let state = attributes
                .as_items()
                .iter()
                .find(|item| item.tag().raw() == STATE)
                .and_then(|item| {
                    item.with_value(|value| match value {
                        kmipkit_ttlv::ValueView::Enumeration(raw) => Some(*raw),
                        _ => None,
                    })
                })
                .ok_or_else(|| "State preflight omitted the State Enumeration".to_owned())?;
            Ok(state)
        },
    )
}

fn revoke_for_destroy(
    client: &mut Client,
    case_id: &str,
    unique_id: &str,
) -> Result<(), Box<dyn Error>> {
    // OASIS KMIP v2.1 §6.1.15 allows Destroy only in Pre-Active or
    // Deactivated; §6.1.44 moves a non-compromise Revoke to Deactivated.
    execute_case(
        client,
        case_id,
        ClientRequest::revoke(
            RevokeRequest::new(RevocationReason::CESSATION_OF_OPERATION)
                .with_unique_identifier(self_identifier(unique_id)),
        ),
        "Revoke setup to reach Deactivated state for Destroy",
        |response| {
            require_identifier(
                response
                    .revoke()
                    .and_then(|result| result.unique_identifier()),
                "Revoke setup",
            )
        },
    )
}

fn activate_for_revoke(
    client: &mut Client,
    case_id: &str,
    unique_id: &str,
) -> Result<(), Box<dyn Error>> {
    execute_case(
        client,
        case_id,
        ClientRequest::activate(ActivateRequest::new(Some(self_identifier(unique_id)))),
        "Activate setup for Revoke",
        |response| {
            require_identifier(
                response.activate().and_then(|r| r.unique_identifier()),
                "Activate setup",
            )
        },
    )
}

fn modify_activation_date(
    client: &mut Client,
    case_id: &str,
    identifier: &str,
) -> Result<(), Box<dyn Error>> {
    let date = future_date()?;
    let current = kmip_item(ACTIVATION_DATE, Value::date_time(date))?;
    let replacement = kmip_item(ACTIVATION_DATE, Value::date_time(date + 60))?;
    execute_case(
        client,
        case_id,
        ClientRequest::add_attribute(AddAttributeRequest::new(
            Some(identifier.to_owned()),
            NewAttribute::new(kmip_item(ACTIVATION_DATE, Value::date_time(date))?),
        )),
        "Add Activation Date setup",
        |response| {
            require_text_identifier(
                response.add_attribute().and_then(|r| r.unique_identifier()),
                "Add Attribute setup",
            )
        },
    )?;
    execute_case(
        client,
        case_id,
        ClientRequest::modify_attribute(ModifyAttributeRequest::new(
            Some(identifier.to_owned()),
            Some(CurrentAttribute::new(current)),
            NewAttribute::new(replacement),
        )),
        "Modify Attribute",
        |response| {
            require_text_identifier(
                response
                    .modify_attribute()
                    .and_then(|r| r.unique_identifier()),
                "Modify Attribute",
            )
        },
    )
}

fn set_activation_date(
    client: &mut Client,
    case_id: &str,
    identifier: &str,
) -> Result<(), Box<dyn Error>> {
    execute_case(
        client,
        case_id,
        ClientRequest::set_attribute(SetAttributeRequest::new(
            Some(identifier.to_owned()),
            NewAttribute::new(kmip_item(
                ACTIVATION_DATE,
                Value::date_time(future_date()?),
            )?),
        )),
        "Set Attribute",
        |response| {
            require_text_identifier(
                response.set_attribute().and_then(|r| r.unique_identifier()),
                "Set Attribute",
            )
        },
    )
}

fn future_date() -> Result<i64, Box<dyn Error>> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    Ok(i64::try_from(now)? + 3600)
}

fn name_attribute(value: &str) -> Result<Item, Box<dyn Error>> {
    let name = structure([
        kmip_item(NAME_VALUE, Value::text_string(value.to_owned()))?,
        kmip_item(NAME_TYPE, Value::enumeration(1))?,
    ])?;
    kmip_item(NAME, Value::structure(name))
}

fn identifier(value: &str) -> UniqueIdentifier {
    UniqueIdentifier::TextString(value.to_owned())
}

fn self_identifier(value: &str) -> UniqueIdentifier {
    identifier(value)
}

fn require_identifier(
    identifier: Option<&UniqueIdentifier>,
    operation: &str,
) -> Result<(), String> {
    identifier
        .ok_or_else(|| format!("successful {operation} response has no Unique Identifier"))
        .map(|_| ())
}

fn require_text_identifier(identifier: Option<&str>, operation: &str) -> Result<(), String> {
    identifier
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("successful {operation} response has no Unique Identifier"))
        .map(|_| ())
}

fn manifest_row(case_id: &str) -> Result<serde_json::Value, Box<dyn Error>> {
    let manifest: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../tests/integration/cosmian/capability-matrix.json"
    ))?;
    manifest["cases"]
        .as_array()
        .and_then(|rows| rows.iter().find(|row| row["case_id"] == case_id))
        .cloned()
        .ok_or_else(|| format!("case {case_id} was not found in the capability manifest").into())
}

fn required_text<'a>(row: &'a serde_json::Value, field: &str) -> Result<&'a str, Box<dyn Error>> {
    row[field]
        .as_str()
        .ok_or_else(|| format!("manifest case has no textual {field}").into())
}
