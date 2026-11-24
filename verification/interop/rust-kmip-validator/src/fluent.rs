//! Construct the target of an existing recipe with the installed Fluent API.

use crate::models::Case;
use kmipkit::protocol::{AttributeSet, GetRequest};
use kmipkit::{Client, ClientRequest};

pub(crate) fn supports(case: &Case) -> bool {
    !matches!(
        case.requires["workflow_type"].as_str(),
        Some("server_to_client" | "async_poll" | "batch_id_placeholder" | "multipart_streaming")
    ) && matches!(
        case.operation.as_str(),
        "Create"
            | "Query"
            | "Discover Versions"
            | "Add Attribute"
            | "Set Attribute"
            | "Modify Attribute"
            | "Delete Attribute"
            | "Get Usage Allocation"
            | "RNG Retrieve"
            | "Revoke"
            | "Get"
            | "Get Attributes"
            | "Get Attribute List"
            | "Locate"
            | "Ping"
            | "Get Constraints"
            | "Activate"
            | "Archive"
            | "Recover"
            | "Destroy"
    ) && (case.operation != "Create"
        || matches!(
            case.requires["object_type"].as_str(),
            None | Some("Symmetric Key")
        ))
}

fn attributes(source: &AttributeSet) -> Result<AttributeSet, &'static str> {
    let mut result = AttributeSet::new();
    for item in source.as_items() {
        result
            .try_push(item.try_clone().map_err(|_| "attribute copy failed")?)
            .map_err(|_| "attribute set rejected a fixture value")?;
    }
    Ok(result)
}

fn build_get(client: &mut Client, source: &GetRequest) -> Result<ClientRequest, &'static str> {
    let mut builder = client.request().get_request();
    if let Some(id) = source.unique_identifier() {
        builder = builder.unique_identifier(id.clone());
    }
    if let Some(value) = source.key_format_type() {
        builder = builder.key_format_type(value);
    }
    if let Some(value) = source.key_wrap_type() {
        builder = builder.key_wrap_type(value);
    }
    if let Some(value) = source.key_compression_type() {
        builder = builder.key_compression_type(value);
    }
    if let Some(value) = source.key_wrapping_specification() {
        builder = builder.key_wrapping_specification(
            value
                .view()
                .try_clone()
                .map_err(|_| "wrapping specification copy failed")?,
        );
    }
    Ok(ClientRequest::Get(builder.build()))
}

pub(crate) fn build(
    client: &mut Client,
    source: ClientRequest,
) -> Result<ClientRequest, &'static str> {
    match source {
        ClientRequest::Get(source) => build_get(client, &source),
        ClientRequest::AddAttribute(_)
        | ClientRequest::SetAttribute(_)
        | ClientRequest::ModifyAttribute(_)
        | ClientRequest::DeleteAttribute(_) => build_mutation(client, &source),
        ClientRequest::Query(_)
        | ClientRequest::DiscoverVersions(_)
        | ClientRequest::GetUsageAllocation(_)
        | ClientRequest::RngRetrieve(_)
        | ClientRequest::Revoke(_) => build_simple(client, &source),
        other => build_object(client, other),
    }
}

fn build_object(client: &mut Client, source: ClientRequest) -> Result<ClientRequest, &'static str> {
    let request = client.request();
    let invalid = |_| "Fluent builder rejected the typed recipe inputs";
    Ok(match source {
        ClientRequest::Create(source) => {
            let mut builder = request
                .create()
                .object_type(source.object_type())
                .attributes(attributes(source.attributes())?);
            if let Some(masks) = source.protection_storage_masks() {
                builder = builder.protection_storage_masks(
                    masks
                        .view()
                        .try_clone()
                        .map_err(|_| "protection masks copy failed")?,
                );
            }
            ClientRequest::Create(builder.build().map_err(invalid)?)
        }
        ClientRequest::GetAttributes(source) => {
            let mut builder = request.get_attributes();
            if let Some(id) = source.unique_identifier() {
                builder = builder.unique_identifier(id);
            }
            for reference in source.attribute_references() {
                builder = builder.attribute_reference(reference.clone());
            }
            ClientRequest::GetAttributes(builder.build().map_err(invalid)?)
        }
        ClientRequest::GetAttributeList(source) => {
            let mut builder = request.get_attribute_list();
            if let Some(id) = source.unique_identifier() {
                builder = builder.unique_identifier(id);
            }
            ClientRequest::GetAttributeList(builder.build().map_err(invalid)?)
        }
        ClientRequest::Locate(source) => {
            let mut builder = request.locate(attributes(source.attributes())?);
            if let Some(value) = source.maximum_items() {
                builder = builder.maximum_items(value);
            }
            if let Some(value) = source.offset_items() {
                builder = builder.offset_items(value);
            }
            if let Some(value) = source.storage_status_mask() {
                builder = builder.storage_status_mask(value);
            }
            if let Some(value) = source.object_group_member() {
                builder = builder.object_group_member(value);
            }
            ClientRequest::Locate(builder.build_request().map_err(invalid)?)
        }
        ClientRequest::Ping(_) => ClientRequest::Ping(request.ping().build()),
        ClientRequest::GetConstraints(_) => {
            ClientRequest::GetConstraints(request.get_constraints().build())
        }
        ClientRequest::Activate(source) => {
            let mut builder = request.activate();
            if let Some(id) = source.unique_identifier() {
                builder = builder.unique_identifier(id.clone());
            }
            ClientRequest::Activate(builder.build_request().map_err(invalid)?)
        }
        ClientRequest::Archive(source) => {
            let mut builder = request.archive();
            if let Some(id) = source.unique_identifier() {
                builder = builder.unique_identifier(id.clone());
            }
            ClientRequest::Archive(builder.build_request().map_err(invalid)?)
        }
        ClientRequest::Recover(source) => {
            let mut builder = request.recover();
            if let Some(id) = source.unique_identifier() {
                builder = builder.unique_identifier(id.clone());
            }
            ClientRequest::Recover(builder.build_request().map_err(invalid)?)
        }
        ClientRequest::Destroy(source) => ClientRequest::Destroy(
            request
                .destroy(
                    source
                        .unique_identifier()
                        .ok_or("Fluent Destroy requires a fixture identifier")?
                        .clone(),
                )
                .build(),
        ),
        _ => return Err("no equivalent Rust Fluent request recipe"),
    })
}

fn build_simple(
    client: &mut Client,
    source: &ClientRequest,
) -> Result<ClientRequest, &'static str> {
    let request = client.request();
    let invalid = |_| "Fluent builder rejected the typed recipe inputs";
    Ok(match source {
        ClientRequest::Query(source) => {
            let mut builder = request.query().functions(source.functions().to_vec());
            if let Some(groups) = source.object_groups() {
                builder = builder.object_groups(groups.to_vec());
            }
            ClientRequest::Query(builder.build().map_err(invalid)?)
        }
        ClientRequest::DiscoverVersions(source) => {
            let mut builder = request.discover_versions();
            if !source.protocol_versions().is_empty() {
                builder = builder.protocol_versions(source.protocol_versions().to_vec());
            }
            ClientRequest::DiscoverVersions(builder.build().map_err(invalid)?)
        }
        ClientRequest::GetUsageAllocation(source) => {
            let mut builder = request
                .get_usage_allocation()
                .usage_limits_count(source.usage_limits_count());
            if let Some(id) = source.unique_identifier() {
                builder = builder.unique_identifier(id.clone());
            }
            ClientRequest::GetUsageAllocation(builder.build().map_err(invalid)?)
        }
        ClientRequest::RngRetrieve(source) => ClientRequest::RngRetrieve(
            request
                .rng_retrieve()
                .data_length(source.data_length())
                .build()
                .map_err(invalid)?,
        ),
        ClientRequest::Revoke(source) => {
            let mut builder = request.revoke(source.revocation_reason());
            if let Some(id) = source.unique_identifier() {
                builder = builder.unique_identifier(id.clone());
            }
            if let Some(message) = source.revocation_message() {
                builder = builder.revocation_message(message);
            }
            if let Some(date) = source.compromise_occurrence_date() {
                builder = builder.compromise_occurrence_date(date);
            }
            ClientRequest::Revoke(builder.build_request().map_err(invalid)?)
        }
        _ => return Err("no equivalent Rust Fluent request recipe"),
    })
}

fn build_mutation(
    client: &mut Client,
    source: &ClientRequest,
) -> Result<ClientRequest, &'static str> {
    let request = client.request();
    let invalid = |_| "Fluent builder rejected the typed recipe inputs";
    let copy_item =
        |value: &kmipkit::ttlv::Item| value.try_clone().map_err(|_| "attribute copy failed");
    Ok(match source {
        ClientRequest::AddAttribute(source) => {
            let mut builder = request
                .add_attribute()
                .attribute_item(copy_item(source.new_attribute().item())?);
            if let Some(id) = source.unique_identifier() {
                builder = builder.unique_identifier(id);
            }
            ClientRequest::AddAttribute(builder.build().map_err(invalid)?)
        }
        ClientRequest::SetAttribute(source) => {
            let mut builder = request
                .set_attribute()
                .attribute_item(copy_item(source.new_attribute().item())?);
            if let Some(id) = source.unique_identifier() {
                builder = builder.unique_identifier(id);
            }
            ClientRequest::SetAttribute(builder.build().map_err(invalid)?)
        }
        ClientRequest::ModifyAttribute(source) => {
            let mut builder =
                request
                    .modify_attribute()
                    .new_attribute(kmipkit::protocol::NewAttribute::new(copy_item(
                        source.new_attribute().item(),
                    )?));
            if let Some(id) = source.unique_identifier() {
                builder = builder.unique_identifier(id);
            }
            if let Some(attribute) = source.current_attribute() {
                builder = builder.current_attribute(kmipkit::protocol::CurrentAttribute::new(
                    copy_item(attribute.item())?,
                ));
            }
            ClientRequest::ModifyAttribute(builder.build().map_err(invalid)?)
        }
        ClientRequest::DeleteAttribute(source) => {
            let mut builder = request.delete_attribute();
            if let Some(id) = source.unique_identifier() {
                builder = builder.unique_identifier(id);
            }
            if let Some(attribute) = source.current_attribute() {
                builder = builder.current_attribute_item(copy_item(attribute.item())?);
            }
            if let Some(reference) = source.attribute_reference() {
                builder = builder.attribute_reference(reference.clone());
            }
            ClientRequest::DeleteAttribute(builder.build().map_err(invalid)?)
        }
        _ => return Err("no equivalent Rust Fluent attribute recipe"),
    })
}
