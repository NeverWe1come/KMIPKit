//! Shared, sanitized helpers for case-isolated Cosmian lifecycle tests.

use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

use kmipkit_client::{
    Client, ClientBatch, ClientBatchItem, ClientBatchResponse, ClientRequest, ClientResponseView,
};
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
const OBJECT_TYPE: u32 = 0x0042_0057;
const OBJECT_NOT_FOUND: u32 = 0x0000_0037;
const STATUS_SUCCESS: u32 = 0;

struct OwnedObjects {
    primary: String,
}

/// Executes one manifest case using a fresh mTLS client and fresh server object.
pub(super) fn run_lifecycle_case(case_id: &str) -> Result<(), Box<dyn Error>> {
    let row = manifest_row(case_id)?;
    let operation = required_text(&row, "operation")?;
    let object_type = required_text(&row, "object_type")?;
    let mut client = client()?;

    let mut ownership = CaseOwnership::default();
    let operation_result = if operation == "Validate" {
        run_validate_case(&mut client, case_id, &row, &mut ownership)
    } else {
        provision_object(&mut client, object_type, case_id, &mut ownership).and_then(|objects| {
            run_lifecycle_operation(
                &mut client,
                case_id,
                operation,
                object_type,
                &row,
                &objects,
                &mut ownership,
            )
        })
    }
    .map_err(|error| error.to_string());
    let result = finish_operation_with_cleanup(operation_result, || {
        cleanup_owned_objects(&mut client, case_id, &ownership)
    });
    result.map_err(|message| -> Box<dyn Error> { message.into() })?;

    println!("case_id={case_id} classification=typed_success");
    Ok(())
}

fn run_lifecycle_operation(
    client: &mut Client,
    case_id: &str,
    operation: &str,
    object_type: &str,
    row: &serde_json::Value,
    objects: &OwnedObjects,
    ownership: &mut CaseOwnership,
) -> Result<(), Box<dyn Error>> {
    match operation {
        "Activate" => run_activate(client, case_id, object_type, &objects.primary),
        "Add Attribute" => run_add_attribute(client, case_id, &objects.primary),
        "Delete Attribute" => run_delete_attribute(client, case_id, &objects.primary),
        "Get Attribute List" => run_get_attribute_list(client, case_id, &objects.primary),
        "Get Attributes" => run_get_attributes(client, case_id, &objects.primary),
        "Locate" => run_locate(client, case_id, row, &objects.primary),
        "Check" => run_check(client, case_id, row, &objects.primary),
        "Destroy" => run_destroy(client, case_id, object_type, &objects.primary, ownership),
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
                unique_id,
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
                identifier,
                "Add Attribute",
            )
        },
    )?;
    verify_comment(client, case_id, identifier, case_id)?;
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
                identifier,
                "Delete Attribute",
            )
        },
    )?;
    verify_comment_absent(client, case_id, identifier)?;
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
            let comments = attributes
                .as_items()
                .iter()
                .filter(|item| item.tag().raw() == COMMENT)
                .collect::<Vec<_>>();
            if comments.len() != 1 || !comment_value_matches(comments[0], case_id) {
                return Err(
                    "Get Attributes did not return exactly the case-owned Comment value".to_owned(),
                );
            }
            Ok(())
        },
    )?;
    Ok(())
}

fn run_locate(
    client: &mut Client,
    case_id: &str,
    row: &serde_json::Value,
    identifier: &str,
) -> Result<(), Box<dyn Error>> {
    let filter = locate_filter_from_row(row, case_id)?;
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

fn run_check(
    client: &mut Client,
    case_id: &str,
    row: &serde_json::Value,
    unique_id: &str,
) -> Result<(), Box<dyn Error>> {
    let mut request = CheckRequest::new().with_unique_identifier(identifier(unique_id));
    if let Some(mask) = check_usage_mask_from_row(row)? {
        request = request.with_cryptographic_usage_mask(mask);
    }
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
                unique_id,
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
    ownership: &mut CaseOwnership,
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
                unique_id,
                "Destroy",
            )
        },
    )?;
    ownership.release(unique_id);
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
                unique_id,
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
                    identifier,
                    "Modify Attribute",
                )
            },
        )?;
        verify_comment(client, case_id, identifier, &format!("{case_id}-updated"))?;
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
                    identifier,
                    "Set Attribute",
                )
            },
        )?;
        verify_comment(client, case_id, identifier, case_id)?;
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CleanupResult {
    Destroyed,
    AlreadyAbsent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CleanupAction {
    Destroy,
    Revoke,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CleanupActionResult {
    Destroy(CleanupResult),
    Revoke,
}

fn cleanup_owned_objects(
    client: &mut Client,
    case_id: &str,
    ownership: &CaseOwnership,
) -> Result<(), String> {
    let mut failures = Vec::new();
    for identifier in ownership.cleanup_order() {
        if let Err(error) = cleanup_owned_object_attempts(|action| match action {
            CleanupAction::Destroy => {
                cleanup_destroy(client, case_id, identifier).map(CleanupActionResult::Destroy)
            }
            CleanupAction::Revoke => {
                cleanup_revoke(client, case_id, identifier).map(|()| CleanupActionResult::Revoke)
            }
        }) {
            failures.push(format!("case-owned object cleanup failed: {error}"));
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!("case {case_id}: {}", failures.join("; ")))
    }
}

fn cleanup_owned_object_attempts(
    mut perform: impl FnMut(CleanupAction) -> Result<CleanupActionResult, String>,
) -> Result<CleanupResult, String> {
    match perform(CleanupAction::Destroy) {
        Ok(CleanupActionResult::Destroy(result)) => Ok(result),
        Ok(CleanupActionResult::Revoke) => Err("cleanup expected a Destroy result".to_owned()),
        Err(initial_destroy) => {
            let revoke_result = perform(CleanupAction::Revoke);
            match perform(CleanupAction::Destroy) {
                Ok(CleanupActionResult::Destroy(result)) => Ok(result),
                Ok(CleanupActionResult::Revoke) => {
                    Err("cleanup expected a Destroy result".to_owned())
                }
                Err(retry_destroy) => Err(format!(
                    "Destroy {}; Revoke {}; retry Destroy {}",
                    initial_destroy,
                    revoke_result
                        .err()
                        .unwrap_or_else(|| "succeeded".to_owned()),
                    retry_destroy
                )),
            }
        }
    }
}

fn cleanup_destroy(
    client: &mut Client,
    case_id: &str,
    identifier: &str,
) -> Result<CleanupResult, String> {
    let response = execute_cleanup_request(
        client,
        case_id,
        ClientRequest::destroy(DestroyRequest::new(Some(UniqueIdentifier::TextString(
            identifier.to_owned(),
        )))),
        "Destroy cleanup",
    )?;
    let item = response
        .get(0)
        .ok_or_else(|| format!("case {case_id} Destroy cleanup returned no result"))?;
    let outcome = item.outcome();
    let status = outcome.result().status();
    if status.raw() == STATUS_SUCCESS {
        let returned = outcome
            .response()
            .destroy()
            .and_then(|result| result.unique_identifier());
        require_identifier(returned, identifier, "Destroy cleanup")?;
        return Ok(CleanupResult::Destroyed);
    }
    let reason = outcome
        .result()
        .reason()
        .map(kmipkit_protocol::ResultReason::raw);
    if reason == Some(OBJECT_NOT_FOUND) {
        return Ok(CleanupResult::AlreadyAbsent);
    }
    Err(format!(
        "case {case_id} Destroy cleanup: Result Status {} ({}) Reason {:?}",
        status.raw(),
        status.known_name().unwrap_or("unknown"),
        reason
    ))
}

fn cleanup_revoke(client: &mut Client, case_id: &str, identifier: &str) -> Result<(), String> {
    let response = execute_cleanup_request(
        client,
        case_id,
        ClientRequest::revoke(
            RevokeRequest::new(RevocationReason::CESSATION_OF_OPERATION)
                .with_unique_identifier(UniqueIdentifier::TextString(identifier.to_owned())),
        ),
        "Revoke cleanup",
    )?;
    let item = response
        .get(0)
        .ok_or_else(|| format!("case {case_id} Revoke cleanup returned no result"))?;
    let outcome = item.outcome();
    let status = outcome.result().status();
    if status.raw() == STATUS_SUCCESS {
        let returned = outcome
            .response()
            .revoke()
            .and_then(|result| result.unique_identifier());
        return require_identifier(returned, identifier, "Revoke cleanup");
    }
    let reason = outcome
        .result()
        .reason()
        .map(kmipkit_protocol::ResultReason::raw);
    Err(format!(
        "case {case_id} Revoke cleanup: Result Status {} ({}) Reason {:?}",
        status.raw(),
        status.known_name().unwrap_or("unknown"),
        reason
    ))
}

fn execute_cleanup_request(
    client: &mut Client,
    case_id: &str,
    request: ClientRequest,
    operation: &str,
) -> Result<ClientBatchResponse, String> {
    client
        .execute(
            ClientBatch::new(ClientBatchItem::new(request)),
            &kmipkit_ttlv::codec::CodecLimits::defaults(),
        )
        .map_err(|error| {
            format!(
                "case {case_id} {operation}: client error category={:?}, cause={:?}, delivery={:?}",
                error.category(),
                error.cause_category(),
                error.delivery_state()
            )
        })
}

fn provision_object(
    client: &mut Client,
    object_type: &str,
    case_id: &str,
    ownership: &mut CaseOwnership,
) -> Result<OwnedObjects, Box<dyn Error>> {
    let primary = match object_type {
        "SymmetricKey" => {
            let attrs = AttributeSet::try_new([
                kmip_item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(AES))?,
                kmip_item(CRYPTOGRAPHIC_LENGTH, Value::integer(256))?,
                kmip_item(CRYPTOGRAPHIC_USAGE_MASK, Value::integer(12))?,
                name_attribute(case_id)?,
            ])?;
            create_result_id(
                client,
                case_id,
                ClientRequest::Create(CreateRequest::new(ObjectType::from_raw(2), attrs)),
                "Create",
                ownership,
            )?
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
            let (private_id, public_id) =
                execute_created_key_pair(client, case_id, request, ownership)?;
            if object_type == "PrivateKey" {
                private_id
            } else {
                public_id
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
                ownership,
            )?;
            let split = CreateSplitKeyRequest::new(
                ObjectType::from_raw(2),
                2,
                2,
                SplitKeyMethod::XOR,
                AttributeSet::try_new([
                    name_attribute(case_id)?,
                    kmip_item(CRYPTOGRAPHIC_USAGE_MASK, Value::integer(12))?,
                ])?,
            )
            .with_unique_identifier(identifier(&source_id));
            let ids = execute_created_split(client, case_id, split, ownership)?;
            let primary = ids.first().cloned().ok_or_else(|| {
                format!("case {case_id}: Create Split Key returned no identifiers")
            })?;
            ownership.track(source_id);
            primary
        }
        "Certificate" => {
            register_certificate(client, case_id, ValidationCertificate::Leaf, ownership)?
        }
        "SecretData" => register_secret_data(client, case_id, ownership)?,
        "OpaqueObject" => register_opaque_object(client, case_id, ownership)?,
        unknown => return Err(format!("case {case_id}: unsupported object type {unknown}").into()),
    };
    Ok(OwnedObjects { primary })
}

fn create_result_id(
    client: &mut Client,
    case_id: &str,
    request: ClientRequest,
    operation: &str,
    ownership: &mut CaseOwnership,
) -> Result<String, Box<dyn Error>> {
    execute_case(client, case_id, request, operation, |response| {
        let id = response
            .create()
            .and_then(|result| result.unique_identifier())
            .ok_or_else(|| "successful Create response has no Unique Identifier".to_owned())?;
        let id = text_identifier(id)
            .map_err(|_| "Create response identifier is not a text identifier".to_owned())?;
        ownership.track(id.clone());
        Ok(id)
    })
}

fn execute_created_key_pair(
    client: &mut Client,
    case_id: &str,
    request: CreateKeyPairRequest,
    ownership: &mut CaseOwnership,
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
            let private_id = text_identifier(private_id)
                .map_err(|_| "Private Key identifier is not text".to_owned())?;
            ownership.track(private_id.clone());
            let public_id = text_identifier(public_id)
                .map_err(|_| "Public Key identifier is not text".to_owned())?;
            ownership.track(public_id.clone());
            Ok((private_id, public_id))
        },
    )
}

fn execute_created_split(
    client: &mut Client,
    case_id: &str,
    request: CreateSplitKeyRequest,
    ownership: &mut CaseOwnership,
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
                .try_fold(Vec::new(), |mut ids, id| {
                    let id = text_identifier(id)
                        .map_err(|_| "Split Key identifier is not text".to_owned())?;
                    ownership.track(id.clone());
                    ids.push(id);
                    Ok(ids)
                })
        },
    )
}

fn register_certificate(
    client: &mut Client,
    case_id: &str,
    certificate: ValidationCertificate,
    ownership: &mut CaseOwnership,
) -> Result<String, Box<dyn Error>> {
    let der = read_pem_der(&validation_certificate_path(certificate))?;
    let attributes = AttributeSet::try_new([
        kmip_item(
            CERTIFICATE_LENGTH,
            Value::integer(i32::try_from(der.len())?),
        )?,
        kmip_item(CRYPTOGRAPHIC_USAGE_MASK, Value::integer(2))?,
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
        ownership,
    )
}

fn register_secret_data(
    client: &mut Client,
    case_id: &str,
    ownership: &mut CaseOwnership,
) -> Result<String, Box<dyn Error>> {
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
        ownership,
    )
}

fn register_opaque_object(
    client: &mut Client,
    case_id: &str,
    ownership: &mut CaseOwnership,
) -> Result<String, Box<dyn Error>> {
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
        ownership,
    )
}

fn register_result_id(
    client: &mut Client,
    case_id: &str,
    request: RegisterRequest,
    ownership: &mut CaseOwnership,
) -> Result<String, Box<dyn Error>> {
    let request = ClientRequest::register(request);
    execute_case(client, case_id, request, "Register setup", |response| {
        let id = response
            .register()
            .and_then(|result| result.unique_identifier())
            .ok_or_else(|| "successful Register response has no Unique Identifier".to_owned())?;
        let id = text_identifier(id)
            .map_err(|_| "Register response identifier is not text".to_owned())?;
        ownership.track(id.clone());
        Ok(id)
    })
}

fn run_validate_case(
    client: &mut Client,
    case_id: &str,
    row: &serde_json::Value,
    ownership: &mut CaseOwnership,
) -> Result<(), Box<dyn Error>> {
    let params = &row["parameters"];
    let mut request = ValidateRequest::new();
    let plan = validation_chain_plan(row, case_id);
    let mut managed = Vec::new();
    for (name, certificate) in &plan.managed_certificates {
        managed.push(register_certificate(client, name, *certificate, ownership)?);
    }
    match params["chain_input"].as_str() {
        Some("certificate-values") => {
            for certificate in &plan.certificate_values {
                let der = read_pem_der(&validation_certificate_path(*certificate))?;
                request = request.with_certificate(certificate_structure(&der)?);
            }
        }
        Some("managed-identifiers") => {
            for identifier in managed {
                request = request.with_unique_identifier(UniqueIdentifier::TextString(identifier));
            }
        }
        Some("mixed") => {
            let leaf_der = read_pem_der(&validation_certificate_path(ValidationCertificate::Leaf))?;
            request = request
                .with_certificate(certificate_structure(&leaf_der)?)
                .with_unique_identifier(UniqueIdentifier::TextString(managed[0].clone()))
                .with_unique_identifier(UniqueIdentifier::TextString(managed[1].clone()));
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
                    identifier,
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
                identifier,
                "Add Attribute setup",
            )
        },
    )
}

fn verify_comment(
    client: &mut Client,
    case_id: &str,
    identifier: &str,
    expected: &str,
) -> Result<(), Box<dyn Error>> {
    execute_case(
        client,
        case_id,
        ClientRequest::get_attributes(GetAttributesRequest::try_new(
            Some(identifier.to_owned()),
            [AttributeReference::tag(COMMENT)],
        )?),
        "Get Attributes Comment verification",
        |response| {
            let attributes = response
                .get_attributes()
                .and_then(|result| result.attributes())
                .ok_or_else(|| "Comment verification returned no Attributes".to_owned())?;
            let comments = attributes
                .as_items()
                .iter()
                .filter(|item| item.tag().raw() == COMMENT)
                .collect::<Vec<_>>();
            if comments.len() != 1 || !comment_value_matches(comments[0], expected) {
                return Err(
                    "server Comment does not equal the exact value written by this case".to_owned(),
                );
            }
            Ok(())
        },
    )
}

fn verify_comment_absent(
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
        "Get Attribute List Comment deletion verification",
        |response| {
            let references = response
                .get_attribute_list()
                .and_then(|result| result.attribute_references())
                .ok_or_else(|| "Get Attribute List returned no Attribute References".to_owned())?;
            if references
                .iter()
                .any(|reference| reference.tag_value() == Some(COMMENT))
            {
                return Err("Delete Attribute left the case-owned Comment present".to_owned());
            }
            Ok(())
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
                unique_id,
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
                unique_id,
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
                identifier,
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
                identifier,
                "Modify Attribute",
            )
        },
    )?;
    verify_activation_date(client, case_id, identifier, date + 60)
}

fn set_activation_date(
    client: &mut Client,
    case_id: &str,
    identifier: &str,
) -> Result<(), Box<dyn Error>> {
    let date = future_date()?;
    execute_case(
        client,
        case_id,
        ClientRequest::set_attribute(SetAttributeRequest::new(
            Some(identifier.to_owned()),
            NewAttribute::new(kmip_item(ACTIVATION_DATE, Value::date_time(date))?),
        )),
        "Set Attribute",
        |response| {
            require_text_identifier(
                response.set_attribute().and_then(|r| r.unique_identifier()),
                identifier,
                "Set Attribute",
            )
        },
    )?;
    verify_activation_date(client, case_id, identifier, date)
}

fn verify_activation_date(
    client: &mut Client,
    case_id: &str,
    identifier: &str,
    expected: i64,
) -> Result<(), Box<dyn Error>> {
    execute_case(
        client,
        case_id,
        ClientRequest::get_attributes(GetAttributesRequest::try_new(
            Some(identifier.to_owned()),
            [AttributeReference::tag(ACTIVATION_DATE)],
        )?),
        "Get Attributes Activation Date verification",
        |response| {
            let attributes = response
                .get_attributes()
                .and_then(|result| result.attributes())
                .ok_or_else(|| "Activation Date verification returned no Attributes".to_owned())?;
            let matches = attributes.as_items().iter().any(|item| {
                item.tag().raw() == ACTIVATION_DATE
                    && item.with_value(|value| {
                        matches!(value, kmipkit_ttlv::ValueView::DateTime(actual) if *actual == expected)
                    })
            });
            if matches {
                Ok(())
            } else {
                Err(
                    "server Activation Date does not equal the exact value written by this case"
                        .to_owned(),
                )
            }
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
    actual: Option<&UniqueIdentifier>,
    expected: &str,
    operation: &str,
) -> Result<(), String> {
    actual
        .filter(|value| identifier_matches(Some(value), expected))
        .ok_or_else(|| {
            format!("successful {operation} response omitted the case-owned Unique Identifier")
        })
        .map(|_| ())
}

fn require_text_identifier(
    actual: Option<&str>,
    expected: &str,
    operation: &str,
) -> Result<(), String> {
    actual
        .filter(|value| !value.is_empty() && *value == expected)
        .ok_or_else(|| {
            format!("successful {operation} response omitted the case-owned Unique Identifier")
        })
        .map(|_| ())
}

fn locate_filter_from_row(
    row: &serde_json::Value,
    case_id: &str,
) -> Result<AttributeSet, Box<dyn Error>> {
    let object_type = object_type_value(required_text(row, "object_type")?)?;
    if row["parameters"]["filter"] != "unique case-owned name and Object Type" {
        return Err(format!("case {case_id}: unsupported Locate filter contract").into());
    }
    AttributeSet::try_new([
        name_attribute(case_id)?,
        kmip_item(OBJECT_TYPE, Value::enumeration(object_type))?,
    ])
    .map_err(Into::into)
}

fn object_type_value(name: &str) -> Result<u32, Box<dyn Error>> {
    match name {
        "Certificate" => Ok(1),
        "SymmetricKey" => Ok(2),
        "PublicKey" => Ok(3),
        "PrivateKey" => Ok(4),
        "SplitKey" => Ok(5),
        "SecretData" => Ok(7),
        "OpaqueObject" => Ok(8),
        _ => Err(format!("unsupported manifest Object Type {name}").into()),
    }
}

fn check_usage_mask_from_row(row: &serde_json::Value) -> Result<Option<i32>, Box<dyn Error>> {
    let value = &row["parameters"]["usage_mask"];
    if value.is_null() {
        Ok(None)
    } else {
        value
            .as_i64()
            .and_then(|raw| i32::try_from(raw).ok())
            .map(Some)
            .ok_or_else(|| "manifest Check usage_mask must be a KMIP Integer".into())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ValidationCertificate {
    Leaf,
    Intermediate,
    Root,
}

fn validation_certificate_path(certificate: ValidationCertificate) -> std::path::PathBuf {
    let relative = match certificate {
        ValidationCertificate::Leaf => "validation/leaf.crt",
        ValidationCertificate::Intermediate => "validation/intermediate.crt",
        ValidationCertificate::Root => "ca/ca.crt",
    };
    certificate_directory().join(relative)
}

#[derive(Debug, Eq, PartialEq)]
struct ValidationChainPlan {
    certificate_values: Vec<ValidationCertificate>,
    managed_certificates: Vec<(String, ValidationCertificate)>,
}

fn validation_chain_plan(row: &serde_json::Value, case_id: &str) -> ValidationChainPlan {
    match row["parameters"]["chain_input"].as_str() {
        Some("certificate-values") => ValidationChainPlan {
            certificate_values: vec![
                ValidationCertificate::Leaf,
                ValidationCertificate::Intermediate,
                ValidationCertificate::Root,
            ],
            managed_certificates: Vec::new(),
        },
        Some("managed-identifiers") => ValidationChainPlan {
            certificate_values: Vec::new(),
            managed_certificates: [
                ("leaf", ValidationCertificate::Leaf),
                ("intermediate", ValidationCertificate::Intermediate),
                ("root", ValidationCertificate::Root),
            ]
            .map(|(name, certificate)| (format!("{case_id}-{name}"), certificate))
            .into(),
        },
        Some("mixed") => ValidationChainPlan {
            certificate_values: vec![ValidationCertificate::Leaf],
            managed_certificates: [
                ("intermediate", ValidationCertificate::Intermediate),
                ("root", ValidationCertificate::Root),
            ]
            .map(|(name, certificate)| (format!("{case_id}-{name}"), certificate))
            .into(),
        },
        _ => ValidationChainPlan {
            certificate_values: Vec::new(),
            managed_certificates: Vec::new(),
        },
    }
}

#[derive(Default)]
struct CaseOwnership {
    identifiers: Vec<String>,
}

impl CaseOwnership {
    fn track(&mut self, identifier: String) {
        self.identifiers.push(identifier);
    }

    fn cleanup_order(&self) -> impl Iterator<Item = &str> {
        self.identifiers.iter().rev().map(String::as_str)
    }

    fn release(&mut self, identifier: &str) {
        self.identifiers
            .retain(|owned_identifier| owned_identifier != identifier);
    }
}

fn finish_operation_with_cleanup<T>(
    operation: Result<T, String>,
    cleanup: impl FnOnce() -> Result<(), String>,
) -> Result<T, String> {
    let cleanup_result = cleanup();
    match (operation, cleanup_result) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(primary), Ok(())) => Err(primary),
        (Ok(_), Err(cleanup)) => Err(format!("case cleanup failed: {cleanup}")),
        (Err(primary), Err(cleanup)) => {
            Err(format!("{primary}; case cleanup also failed: {cleanup}"))
        }
    }
}

fn identifier_matches(actual: Option<&UniqueIdentifier>, expected: &str) -> bool {
    matches!(actual, Some(UniqueIdentifier::TextString(value)) if value == expected)
}

fn comment_value_matches(item: &Item, expected: &str) -> bool {
    item.tag().raw() == COMMENT
        && item.with_value(|value| {
            matches!(value, kmipkit_ttlv::ValueView::TextString(actual) if actual == expected)
        })
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

#[cfg(test)]
mod review_regression_tests {
    use std::cell::Cell;

    use super::*;

    const LOCATE_CASES: &[(&str, u32)] = &[
        ("cosmian-528-locate-certificate", 1),
        ("cosmian-528-locate-opaqueobject", 8),
        ("cosmian-528-locate-privatekey", 4),
        ("cosmian-528-locate-publickey", 3),
        ("cosmian-528-locate-secretdata", 7),
        ("cosmian-528-locate-splitkey", 5),
        ("cosmian-528-locate-symmetrickey", 2),
    ];

    #[test]
    fn locate_filter_contains_case_name_and_manifest_object_type() {
        for (case_id, object_type) in LOCATE_CASES {
            let row = manifest_row(case_id).expect("Locate manifest row");
            let filter = locate_filter_from_row(&row, case_id).expect("Locate filter");
            let object_type_item = filter
                .as_items()
                .iter()
                .find(|item| item.tag().raw() == 0x0042_0057);
            assert!(
                object_type_item.is_some_and(|item| {
                    item.with_value(|value| matches!(value, kmipkit_ttlv::ValueView::Enumeration(raw) if *raw == *object_type))
                }),
                "{case_id} Locate filter must contain its Object Type"
            );
            assert!(
                filter
                    .as_items()
                    .iter()
                    .any(|item| item.tag().raw() == NAME),
                "{case_id} Locate filter must contain its case-owned Name"
            );
        }
    }

    #[test]
    fn check_usage_masks_are_explicit_and_match_each_case_fixture() {
        let cases = [
            ("cosmian-528-check-certificate", 2),
            ("cosmian-528-check-privatekey", 1),
            ("cosmian-528-check-publickey", 2),
            ("cosmian-528-check-secretdata", 12),
            ("cosmian-528-check-splitkey", 12),
            ("cosmian-528-check-symmetrickey", 12),
        ];
        for (case_id, expected) in cases {
            let row = manifest_row(case_id).expect("Check manifest row");
            assert_eq!(
                check_usage_mask_from_row(&row).expect("usage mask"),
                Some(expected),
                "{case_id} must send the manifest's exact usage mask"
            );
        }
        let opaque = manifest_row("cosmian-528-check-opaqueobject")
            .expect("OpaqueObject Check manifest row");
        assert_eq!(
            check_usage_mask_from_row(&opaque).expect("optional mask"),
            None
        );
    }

    #[test]
    fn validation_managed_identifiers_use_the_matching_chain_certificates() {
        let cases = [
            (
                "cosmian-528-validate-certificate-managed-identifiers-current",
                vec![
                    ValidationCertificate::Leaf,
                    ValidationCertificate::Intermediate,
                    ValidationCertificate::Root,
                ],
            ),
            (
                "cosmian-528-validate-certificate-managed-identifiers-explicit-time",
                vec![
                    ValidationCertificate::Leaf,
                    ValidationCertificate::Intermediate,
                    ValidationCertificate::Root,
                ],
            ),
            (
                "cosmian-528-validate-certificate-mixed-current",
                vec![
                    ValidationCertificate::Intermediate,
                    ValidationCertificate::Root,
                ],
            ),
            (
                "cosmian-528-validate-certificate-mixed-explicit-time",
                vec![
                    ValidationCertificate::Intermediate,
                    ValidationCertificate::Root,
                ],
            ),
        ];
        for (case_id, expected) in cases {
            let row = manifest_row(case_id).expect("Validate manifest row");
            let plan = validation_chain_plan(&row, case_id);
            assert_eq!(
                plan.managed_certificates
                    .iter()
                    .map(|(_, certificate)| *certificate)
                    .collect::<Vec<_>>(),
                expected,
                "{case_id} managed IDs must refer to their matching chain certificate"
            );
        }
        let values_row =
            manifest_row("cosmian-528-validate-certificate-certificate-values-current")
                .expect("certificate-values Validate row");
        assert_eq!(
            validation_chain_plan(&values_row, "values").certificate_values,
            [
                ValidationCertificate::Leaf,
                ValidationCertificate::Intermediate,
                ValidationCertificate::Root
            ]
        );
    }

    #[test]
    fn operation_failure_still_attempts_cleanup_and_remains_primary() {
        let attempted = Cell::new(false);
        let result =
            finish_operation_with_cleanup(Err::<(), _>("primary failure".to_owned()), || {
                attempted.set(true);
                Ok(())
            });
        assert!(
            attempted.get(),
            "cleanup must run after an operation failure"
        );
        assert_eq!(result, Err("primary failure".to_owned()));

        let combined =
            finish_operation_with_cleanup(Err::<(), _>("primary failure".to_owned()), || {
                Err("secondary cleanup failure".to_owned())
            });
        let message = combined.expect_err("both errors should be observable");
        assert!(message.contains("primary failure"));
        assert!(message.contains("secondary cleanup failure"));
    }

    #[test]
    fn owned_objects_are_cleaned_in_reverse_creation_order() {
        let mut ownership = CaseOwnership::default();
        ownership.track("first-created".to_owned());
        ownership.track("linked-second".to_owned());
        assert_eq!(
            ownership.cleanup_order().collect::<Vec<_>>(),
            ["linked-second", "first-created"]
        );
    }

    #[test]
    fn successfully_destroyed_case_object_is_released_from_cleanup() {
        let mut ownership = CaseOwnership::default();
        ownership.track("first-created".to_owned());
        ownership.track("destroyed-by-case-operation".to_owned());
        ownership.track("linked-second".to_owned());
        ownership.release("destroyed-by-case-operation");
        assert_eq!(
            ownership.cleanup_order().collect::<Vec<_>>(),
            ["linked-second", "first-created"]
        );
    }

    #[test]
    fn split_key_source_and_parts_keep_one_reverse_creation_order_ledger() {
        let mut ownership = CaseOwnership::default();
        ownership.track("split-source".to_owned());
        ownership.track("split-part-one".to_owned());
        ownership.track("split-part-two".to_owned());
        ownership.track("split-source".to_owned());
        assert_eq!(
            ownership.cleanup_order().collect::<Vec<_>>(),
            ["split-part-two", "split-part-one", "split-source"]
        );
    }

    #[test]
    fn ambiguous_destroy_failure_is_not_retried_or_followed_by_revoke() {
        for diagnostic in [
            "cleanup transport error delivery=NotSent",
            "cleanup transport error delivery=PossiblySent",
            "cleanup response validation failed: Unique Identifier mismatch",
        ] {
            let destroy_calls = Cell::new(0);
            let revoke_calls = Cell::new(0);
            let result = cleanup_owned_object_attempts(|action| match action {
                CleanupAction::Destroy => {
                    destroy_calls.set(destroy_calls.get() + 1);
                    if destroy_calls.get() == 1 {
                        Err(diagnostic.to_owned())
                    } else {
                        Ok(CleanupActionResult::Destroy(CleanupResult::Destroyed))
                    }
                }
                CleanupAction::Revoke => {
                    revoke_calls.set(revoke_calls.get() + 1);
                    Ok(CleanupActionResult::Revoke)
                }
            });
            assert_eq!(destroy_calls.get(), 1, "{diagnostic}");
            assert_eq!(revoke_calls.get(), 0, "{diagnostic}");
            assert_eq!(result, Err(diagnostic.to_owned()));
        }
    }

    #[test]
    fn lifecycle_response_identifier_must_equal_case_owned_identifier() {
        assert!(identifier_matches(
            Some(&UniqueIdentifier::TextString("case-1".to_owned())),
            "case-1"
        ));
        assert!(!identifier_matches(
            Some(&UniqueIdentifier::TextString("other-case".to_owned())),
            "case-1"
        ));
    }

    #[test]
    fn each_attribute_readback_rejects_a_different_case_identifier() {
        for operation in [
            "Get Attribute List",
            "Get Attributes",
            "Get Attributes Comment verification",
            "Get Attribute List Comment deletion verification",
            "Get Attributes Activation Date verification",
        ] {
            assert!(
                require_text_identifier(Some("another-case"), "case-1", operation).is_err(),
                "{operation} must reject a response for a different object"
            );
        }
    }

    #[test]
    fn comment_response_must_equal_the_written_case_value() {
        let wrong = kmip_item(COMMENT, Value::text_string("other-value".to_owned()))
            .expect("well-formed Comment item");
        assert!(!comment_value_matches(&wrong, "case-1"));
        let expected = kmip_item(COMMENT, Value::text_string("case-1".to_owned()))
            .expect("well-formed Comment item");
        assert!(comment_value_matches(&expected, "case-1"));
    }
}
