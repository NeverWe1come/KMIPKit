//! Construct the target of an existing recipe with the installed Fluent API.

use crate::models::Case;
use kmipkit::protocol::{AttributeSet, GetRequest};
use kmipkit::{Client, ClientRequest};

pub(crate) fn supports(case: &Case) -> bool {
    !matches!(
        case.requires["workflow_type"].as_str(),
        Some("server_to_client" | "async_poll" | "batch_id_placeholder")
    ) && (case.requires["workflow_type"] != "multipart_streaming" || case.operation == "Hash")
        && (crate::recipes::CRYPTO_OPERATIONS.contains(&case.operation.as_str())
            || matches!(
                case.operation.as_str(),
                "Create"
                    | "Create Group"
                    | "Create User"
                    | "Create Credential"
                    | "Deactivate"
                    | "Obliterate"
                    | "Encapsulate"
                    | "Decapsulate"
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
            ))
        && (case.operation != "Create"
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

#[cfg(test)]
mod tests {
    use super::*;
    use kmipkit::client_config::{ClientConfiguration, client_extension_registry};
    use kmipkit::protocol::{
        KeyCompressionType, KeyFormatType, KeyWrapType, LocateRequest, ObjectGroupMember,
        RevocationReason, RevokeRequest, StorageStatusMask, UniqueIdentifier,
    };
    use kmipkit::transport_config::{
        CertificateInput, ClientIdentity, Endpoint, PrivateKeyInput, TransportConfig, TrustSource,
    };

    fn fixture_client() -> Client {
        let registry =
            client_extension_registry(Vec::new(), kmipkit::protocol::extension::defaults())
                .expect("empty extension registry");
        let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../bindings/shared/fixtures/login-mtls");
        let transport = TransportConfig::builder(Endpoint::raw_tls("127.0.0.1", 1))
            .client_identity(ClientIdentity::new(
                CertificateInput::from_pem_file(fixture.join("client-cert.pem"))
                    .expect("test certificate"),
                PrivateKeyInput::from_pem_file(fixture.join("client-key.pem"))
                    .expect("test private key"),
            ))
            .trust_source(TrustSource::certificate_authorities(vec![
                CertificateInput::from_pem_file(fixture.join("server-cert.pem"))
                    .expect("test trust anchor"),
            ]))
            .build()
            .expect("valid endpoint configuration");
        Client::new(ClientConfiguration::new(registry), transport)
            .expect("client construction without connecting")
    }

    #[test]
    fn optional_get_selectors_survive_fluent_construction() {
        let mut client = fixture_client();
        let id = UniqueIdentifier::TextString("fixture-key".into());
        let format = KeyFormatType::from_raw(7);
        let wrapping = KeyWrapType::from_raw(2);
        let compression = KeyCompressionType::from_raw(1);
        let fixture = GetRequest::new()
            .with_unique_identifier(id.clone())
            .with_key_format_type(format)
            .with_key_wrap_type(wrapping)
            .with_key_compression_type(compression)
            .with_key_wrapping_specification(kmipkit::ttlv::Structure::new());

        let ClientRequest::Get(result) = build(&mut client, ClientRequest::Get(fixture))
            .expect("Fluent Get with optional selectors")
        else {
            panic!("expected Get");
        };
        assert_eq!(result.unique_identifier(), Some(&id));
        assert_eq!(result.key_format_type(), Some(format));
        assert_eq!(result.key_wrap_type(), Some(wrapping));
        assert_eq!(result.key_compression_type(), Some(compression));
        assert!(result.key_wrapping_specification().is_some());
    }

    #[test]
    fn locate_and_revoke_preserve_explicit_optional_values() {
        let mut client = fixture_client();
        let storage = StorageStatusMask::from_raw(3);
        let group = ObjectGroupMember::from_raw(1);
        let locate = LocateRequest::new(AttributeSet::new())
            .with_maximum_items(3)
            .with_offset_items(2)
            .with_storage_status_mask(storage)
            .with_object_group_member(group);
        let ClientRequest::Locate(result) = build(&mut client, ClientRequest::Locate(locate))
            .expect("Fluent Locate with explicit filters")
        else {
            panic!("expected Locate");
        };
        assert_eq!(result.maximum_items(), Some(3));
        assert_eq!(result.offset_items(), Some(2));
        assert_eq!(result.storage_status_mask(), Some(storage));
        assert_eq!(result.object_group_member(), Some(group));

        let revoke = RevokeRequest::new(RevocationReason::from_raw(2))
            .with_revocation_message("fixture compromise".into())
            .with_compromise_occurrence_date(1);
        let ClientRequest::Revoke(result) = build(&mut client, ClientRequest::Revoke(revoke))
            .expect("Fluent Revoke with explicit message and date")
        else {
            panic!("expected Revoke");
        };
        assert_eq!(result.revocation_message(), Some("fixture compromise"));
        assert_eq!(result.compromise_occurrence_date(), Some(1));
    }
}
