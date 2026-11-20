//! Installed public-library configuration and a single request exchange.
use crate::models::{Connection, Result};
use kmipkit::client_config::{ClientConfiguration, client_extension_registry};
use kmipkit::protocol::{Authentication, Credential, SecretText, UsernameAndPasswordCredential};
use kmipkit::transport_config::{
    CertificateInput, ClientIdentity, Endpoint, PrivateKeyInput, TimeoutLimit, TimeoutPolicy,
    TransportConfig, TrustSource,
};
use kmipkit::ttlv::{Item, RawTag, Structure, Value};
use kmipkit::{
    Client, ClientBatch, ClientBatchItem, ClientBatchItemResponse, ClientRequest, RequestOptions,
};
use std::time::Duration;

pub(crate) fn item(tag: u32, value: Value) -> Result<Item> {
    Item::new(
        RawTag::new(tag)
            .map_err(|_| "invalid tag")?
            .try_checked()
            .map_err(|_| "unallocated tag")?,
        value,
    )
    .map_err(|_| "invalid TTLV item")
}
pub(crate) fn structure(_tag: u32, children: Vec<Item>) -> Result<Structure> {
    let mut result = Structure::new();
    for child in children {
        result
            .try_push(child)
            .map_err(|_| "invalid TTLV structure")?;
    }
    Ok(result)
}
pub(crate) fn client(connection: &Connection) -> Result<Client> {
    connection.validate()?;
    if env!("KMIPKIT_LIBRARY_COMMIT") != crate::models::PIN {
        return Err("library release provenance mismatch");
    }
    if let Some(credentials) = connection
        .authentication
        .as_ref()
        .and_then(|a| a.username_password.as_ref())
    {
        for name in [&credentials.username_env, &credentials.password_env] {
            let _value = SecretText::new(
                std::env::var(name).map_err(|_| "credential environment unavailable")?,
            );
        }
    }
    let tls = connection.tls.as_ref().ok_or("TLS identity required")?;
    let endpoint = match connection.transport.as_str() {
        "raw_tls" => {
            let value = connection
                .endpoints
                .raw_tls
                .as_ref()
                .ok_or("raw TLS endpoint required")?;
            Endpoint::raw_tls(&value.host, value.port)
        }
        "https" => {
            let value = connection
                .endpoints
                .https
                .as_ref()
                .ok_or("HTTPS endpoint required")?;
            Endpoint::https(&value.url)
        }
        _ => return Err("unsupported transport"),
    };
    let mut builder = TransportConfig::builder(endpoint)
        .client_identity(ClientIdentity::new(
            CertificateInput::from_pem_file(&tls.client_cert_path)
                .map_err(|_| "client certificate unavailable or invalid")?,
            PrivateKeyInput::from_pem_file(&tls.client_key_path)
                .map_err(|_| "client key unavailable or invalid")?,
        ))
        .trust_source(TrustSource::certificate_authorities(vec![
            CertificateInput::from_pem_file(&tls.ca_cert_path)
                .map_err(|_| "CA certificate unavailable or invalid")?,
        ]))
        .timeouts(
            TimeoutPolicy::default()
                .with_connect(TimeoutLimit::Bounded(Duration::from_secs(
                    tls.connect_timeout_seconds.unwrap_or(10),
                )))
                .with_read(TimeoutLimit::Bounded(Duration::from_secs(
                    tls.read_timeout_seconds.unwrap_or(30),
                ))),
        );
    if let Some(name) = &tls.server_name_override {
        builder = builder.tls_server_name(name);
    }
    let transport = builder
        .build()
        .map_err(|_| "invalid TLS or endpoint configuration")?;
    let registry = client_extension_registry(Vec::new(), kmipkit::protocol::extension::defaults())
        .map_err(|_| "library registry failure")?;
    let version = if connection.protocol_version == "2.0" {
        kmipkit::ProtocolVersion::V2_0
    } else {
        kmipkit::ProtocolVersion::V2_1
    };
    let configuration = ClientConfiguration::new(registry).with_protocol_version(version);
    Client::new(configuration, transport).map_err(|_| "client configuration rejected")
}

pub(crate) fn authenticated_batch(
    connection: &Connection,
    request: ClientRequest,
) -> Result<ClientBatch> {
    authenticate(connection, ClientBatch::new(ClientBatchItem::new(request)))
}
fn authenticate(connection: &Connection, mut batch: ClientBatch) -> Result<ClientBatch> {
    if let Some(credentials) = connection
        .authentication
        .as_ref()
        .and_then(|a| a.username_password.as_ref())
    {
        let user = SecretText::new(
            std::env::var(&credentials.username_env)
                .map_err(|_| "credential environment unavailable")?,
        );
        let password = SecretText::new(
            std::env::var(&credentials.password_env)
                .map_err(|_| "credential environment unavailable")?,
        );
        let value = structure(
            0x0042_0025,
            vec![
                item(0x0042_0099, user.into_ttlv_value())?,
                item(0x0042_00a1, password.into_ttlv_value())?,
            ],
        )?;
        let value = UsernameAndPasswordCredential::try_from_ttlv(value)
            .map_err(|_| "invalid credential")?;
        let credential = Credential::try_from_ttlv(structure(
            0x0042_0023,
            vec![
                item(0x0042_0024, Value::enumeration(1))?,
                item(0x0042_0025, Value::structure(value.into_ttlv()))?,
            ],
        )?)
        .map_err(|_| "invalid credential")?;
        batch = batch.with_authentication(
            Authentication::new(vec![credential]).map_err(|_| "invalid authentication")?,
        );
    }
    Ok(batch)
}

pub(crate) struct Observed {
    pub status: u32,
    pub reason: Option<u32>,
    pub payload_tags: Vec<u32>,
    pub identifier: Option<kmipkit::protocol::UniqueIdentifier>,
    pub identifiers: Vec<kmipkit::protocol::UniqueIdentifier>,
    pub correlation: Option<kmipkit::protocol::SecretBytes>,
    pub bytes: Option<kmipkit::protocol::SecretBytes>,
    pub ticket: Option<kmipkit::protocol::Ticket>,
}

pub(crate) fn observed(response: &ClientBatchItemResponse) -> Observed {
    use kmipkit::ttlv::ValueView;
    let view = response.outcome().response();
    let result = view.result();
    let mut identifier = None;
    let mut identifiers = Vec::new();
    let mut bytes = None;
    let mut ticket = None;
    let mut correlation = None;
    let mut payload_tags = Vec::new();
    if let Some(raw) = response.raw() {
        for value in raw.children() {
            payload_tags.push(value.tag().raw());
            if [0x0042_0094, 0x0042_0066, 0x0042_006f].contains(&value.tag().raw()) {
                identifier = value.with_value(|v| match v {
                    ValueView::TextString(v) => Some(
                        kmipkit::protocol::UniqueIdentifier::TextString(v.to_string()),
                    ),
                    _ => None,
                });
                if let Some(id) = &identifier {
                    identifiers.push(id.clone());
                }
            }
            if [0x0042_00c2, 0x0042_00c3, 0x0042_00c6].contains(&value.tag().raw()) {
                bytes = value.with_value(|v| match v {
                    ValueView::ByteString(v) => {
                        Some(kmipkit::protocol::SecretBytes::new(v.to_vec()))
                    }
                    _ => None,
                });
            }
            if value.tag().raw() == 0x0042_0149 {
                ticket = value
                    .as_structure()
                    .and_then(|v| v.try_clone().ok())
                    .and_then(|v| kmipkit::protocol::Ticket::try_from_ttlv(v).ok());
            }
            if value.tag().raw() == 0x0042_00d6 {
                correlation = value.with_value(|v| match v {
                    ValueView::ByteString(v) => {
                        Some(kmipkit::protocol::SecretBytes::new(v.to_vec()))
                    }
                    _ => None,
                });
            }
        }
    }
    if let kmipkit::ClientBatchOutcome::Pending(pending) = response.outcome() {
        correlation = Some(kmipkit::protocol::SecretBytes::new(
            pending.asynchronous_correlation_value().to_vec(),
        ));
    }
    Observed {
        status: result.status().raw(),
        reason: result.reason().map(kmipkit::ResultReason::raw),
        payload_tags,
        identifier,
        identifiers,
        correlation,
        bytes,
        ticket,
    }
}

#[allow(clippy::too_many_lines)] // Exhaustive direct-method dispatch mirrors the public operation API.
pub(crate) fn execute(
    client: &mut Client,
    connection: &Connection,
    request: ClientRequest,
) -> std::result::Result<Observed, kmipkit::ClientError> {
    let encoding = match connection.encoding.as_str() {
        "JSON" => kmipkit_transport::WireEncoding::Json,
        "XML" => kmipkit_transport::WireEncoding::Xml,
        _ => kmipkit_transport::WireEncoding::Ttlv,
    };
    let options = RequestOptions::default().with_wire_encoding(encoding);
    let limits = kmipkit::ttlv::codec::CodecLimits::defaults();
    // Direct operation methods are used where the public high-level API admits
    // the same immutable request. Authentication requires the explicit batch.
    if connection.api_level == "high-level" && connection.authentication.is_none() {
        match request {
            ClientRequest::DiscoverVersions(request) => {
                return client
                    .discover_versions_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::Ping(_) => {
                return client
                    .ping_with_options(&limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::Query(request) => {
                return client
                    .query_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::DelegatedLogin(request) => {
                return client
                    .delegated_login_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::Login(request) => {
                return client
                    .login_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::Logout(request) => {
                return client
                    .logout_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::Get(request) => {
                return client
                    .get_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::Locate(request) => {
                return client
                    .locate_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::AddAttribute(request) => {
                return client
                    .add_attribute_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::AdjustAttribute(request) => {
                return client
                    .adjust_attribute_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::DeleteAttribute(request) => {
                return client
                    .delete_attribute_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::ModifyAttribute(request) => {
                return client
                    .modify_attribute_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::SetAttribute(request) => {
                return client
                    .set_attribute_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::GetAttributes(request) => {
                return client
                    .get_attributes_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::Encrypt(request) => {
                return client
                    .encrypt_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::Decrypt(request) => {
                return client
                    .decrypt_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::Certify(request) => {
                return client
                    .certify_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::Check(request) => {
                return client
                    .check_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::ObtainLease(request) => {
                return client
                    .obtain_lease_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::Recertify(request) => {
                return client
                    .recertify_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::Rekey(request) => {
                return client
                    .rekey_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::RekeyKeyPair(request) => {
                return client
                    .rekey_key_pair_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::ReProvision(request) => {
                return client
                    .reprovision_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::Register(request) => {
                return client
                    .register_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::Revoke(request) => {
                return client
                    .revoke_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::Activate(request) => {
                return client
                    .activate_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::Archive(request) => {
                return client
                    .archive_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::Destroy(request) => {
                return client
                    .destroy_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::Recover(request) => {
                return client
                    .recover_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::Pkcs11(request) => {
                return client
                    .pkcs11_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::DeriveKey(request) => {
                return client
                    .derive_key_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::Export(request) => {
                return client
                    .export_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::Import(request) => {
                return client
                    .import_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::JoinSplitKey(request) => {
                return client
                    .join_split_key_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::Hash(request) => {
                return client
                    .hash_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::Mac(request) => {
                return client
                    .mac_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::MacVerify(request) => {
                return client
                    .mac_verify_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::Sign(request) => {
                return client
                    .sign_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::SignatureVerify(request) => {
                return client
                    .signature_verify_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::GetAttributeList(request) => {
                return client
                    .get_attribute_list_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::SetConstraints(request) => {
                return client
                    .set_constraints_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::SetDefaults(request) => {
                return client
                    .set_defaults_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::GetUsageAllocation(request) => {
                return client
                    .get_usage_allocation_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::Validate(request) => {
                return client
                    .validate_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::RngRetrieve(request) => {
                return client
                    .rng_retrieve_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::Log(request) => {
                return client
                    .log_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::Interop(request) => {
                return client
                    .interop_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::RngSeed(request) => {
                return client
                    .rng_seed_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::Create(request) => {
                return client
                    .create_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::CreateKeyPair(request) => {
                return client
                    .create_key_pair_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::CreateSplitKey(request) => {
                return client
                    .create_split_key_with_options(request, &limits, &options)
                    .map(|response| observed(&response));
            }
            ClientRequest::GetConstraints(_) => {
                return client
                    .get_constraints_with_options(&limits, &options)
                    .map(|response| observed(&response));
            }
            request => return execute_batch(client, connection, request, &options, &limits),
        }
    }
    execute_batch(client, connection, request, &options, &limits)
}

fn execute_batch(
    client: &mut Client,
    connection: &Connection,
    request: ClientRequest,
    options: &RequestOptions,
    limits: &kmipkit::ttlv::codec::CodecLimits,
) -> std::result::Result<Observed, kmipkit::ClientError> {
    let batch = authenticated_batch(connection, request).map_err(|_| {
        kmipkit::ClientError::validation(
            kmipkit::ClientCauseCategory::InvalidInput,
            kmipkit::RequestDeliveryState::NotSent,
            std::io::Error::other("credential unavailable"),
        )
    })?;
    let response = client.execute_with_options(batch, limits, options)?;
    response.get(0).map(observed).ok_or_else(|| {
        kmipkit::ClientError::validation(
            kmipkit::ClientCauseCategory::InvalidInput,
            kmipkit::RequestDeliveryState::ResponseStarted,
            std::io::Error::other("missing result"),
        )
    })
}

pub(crate) fn options(connection: &Connection) -> RequestOptions {
    RequestOptions::default().with_wire_encoding(match connection.encoding.as_str() {
        "JSON" => kmipkit_transport::WireEncoding::Json,
        "XML" => kmipkit_transport::WireEncoding::Xml,
        _ => kmipkit_transport::WireEncoding::Ttlv,
    })
}

pub(crate) fn batch(
    client: &mut Client,
    connection: &Connection,
    requests: Vec<ClientRequest>,
    asynchronous: bool,
) -> std::result::Result<Vec<Observed>, kmipkit::ClientError> {
    let count = requests.len();
    let mut batch =
        ClientBatch::from_items(requests.into_iter().enumerate().map(|(index, request)| {
            let item = ClientBatchItem::new(request);
            if count > 1 {
                item.with_unique_batch_item_id(index.to_be_bytes().to_vec())
            } else {
                item
            }
        }));
    if count > 1 {
        batch = batch.with_batch_order_option(true);
    }
    if asynchronous {
        batch = batch.with_asynchronous_indicator(1);
    }
    let batch = authenticate(connection, batch).map_err(|_| {
        kmipkit::ClientError::validation(
            kmipkit::ClientCauseCategory::InvalidInput,
            kmipkit::RequestDeliveryState::NotSent,
            std::io::Error::other("credential unavailable"),
        )
    })?;
    let response = client.execute_with_options(
        batch,
        &kmipkit::ttlv::codec::CodecLimits::defaults(),
        &options(connection),
    )?;
    Ok(response.iter().map(observed).collect())
}

pub(crate) fn asynchronous(
    client: &mut Client,
    connection: &Connection,
    operation: &str,
    correlation: &[u8],
) -> std::result::Result<Observed, kmipkit::ClientError> {
    use kmipkit::protocol::{
        CancelRequest, PollRequest, ProcessRequest, QueryAsyncRequestsRequest,
    };
    let limits = kmipkit::ttlv::codec::CodecLimits::defaults();
    let options = options(connection);
    let response = match operation {
        "Cancel" => {
            client.execute_cancel_with_options(CancelRequest::new(correlation), &limits, &options)
        }
        "Process" => client.execute_process_with_options(
            ProcessRequest::new(correlation),
            None,
            &limits,
            &options,
        ),
        "Query Asynchronous Requests" => client.execute_query_async_requests_with_options(
            QueryAsyncRequestsRequest::new(),
            None,
            &limits,
            &options,
        ),
        _ => client.execute_poll_with_options(PollRequest::new(correlation), &limits, &options),
    }?;
    Ok(Observed {
        status: response.result().status().raw(),
        reason: response.result().reason().map(kmipkit::ResultReason::raw),
        payload_tags: Vec::new(),
        identifier: None,
        identifiers: Vec::new(),
        bytes: None,
        ticket: None,
        correlation: None,
    })
}
