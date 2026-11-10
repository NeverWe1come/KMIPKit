//! Installed public-library configuration and a single request exchange.
use crate::models::{Connection, Result};
use kmipkit::client_config::{ClientConfiguration, client_extension_registry};
use kmipkit::protocol::{Authentication, Credential, SecretText, UsernameAndPasswordCredential};
use kmipkit::transport_config::{CertificateInput, ClientIdentity, Endpoint, PrivateKeyInput, TimeoutLimit, TimeoutPolicy, TransportConfig, TrustSource};
use kmipkit::ttlv::{Item, RawTag, Structure, Value};
use kmipkit::{Client, ClientBatch, ClientBatchItem, ClientBatchItemResponse, ClientRequest, RequestOptions};
use std::time::Duration;

pub(crate) fn item(tag: u32, value: Value) -> Result<Item> {
    Item::new(RawTag::new(tag).map_err(|_| "invalid tag")?.try_checked().map_err(|_| "unallocated tag")?, value).map_err(|_| "invalid TTLV item")
}
pub(crate) fn structure(_tag: u32, children: Vec<Item>) -> Result<Structure> {
    let mut result = Structure::new();
    for child in children { result.try_push(child).map_err(|_| "invalid TTLV structure")?; }
    Ok(result)
}
pub(crate) fn client(connection: &Connection) -> Result<Client> {
    connection.validate()?;
    if env!("KMIPKIT_LIBRARY_COMMIT") != crate::models::PIN { return Err("library release provenance mismatch"); }
    let tls = connection.tls.as_ref().ok_or("TLS identity required")?;
    let endpoint = match connection.transport.as_str() {
        "raw_tls" => { let value = connection.endpoints.raw_tls.as_ref().ok_or("raw TLS endpoint required")?; Endpoint::raw_tls(&value.host, value.port) }
        "https" => { let value = connection.endpoints.https.as_ref().ok_or("HTTPS endpoint required")?; Endpoint::https(&value.url) }
        _ => return Err("unsupported transport"),
    };
    let mut builder = TransportConfig::builder(endpoint)
        .client_identity(ClientIdentity::new(CertificateInput::from_pem_file(&tls.client_cert_path).map_err(|_| "client certificate unavailable or invalid")?, PrivateKeyInput::from_pem_file(&tls.client_key_path).map_err(|_| "client key unavailable or invalid")?))
        .trust_source(TrustSource::certificate_authorities(vec![CertificateInput::from_pem_file(&tls.ca_cert_path).map_err(|_| "CA certificate unavailable or invalid")?]))
        .timeouts(TimeoutPolicy::default().with_connect(TimeoutLimit::Bounded(Duration::from_secs(tls.connect_timeout_seconds.unwrap_or(10))))
            .with_read(TimeoutLimit::Bounded(Duration::from_secs(tls.read_timeout_seconds.unwrap_or(30)))));
    if let Some(name) = &tls.server_name_override { builder = builder.tls_server_name(name); }
    let transport = builder.build().map_err(|_| "invalid TLS or endpoint configuration")?;
    let registry = client_extension_registry(Vec::new(), kmipkit::protocol::extension::defaults()).map_err(|_| "library registry failure")?;
    let version = if connection.protocol_version == "2.0" { kmipkit::ProtocolVersion::V2_0 } else { kmipkit::ProtocolVersion::V2_1 };
    let configuration = ClientConfiguration::new(registry).with_protocol_version(version);
    Client::new(configuration, transport).map_err(|_| "client configuration rejected")
}

pub(crate) fn authenticated_batch(connection: &Connection, request: ClientRequest) -> Result<ClientBatch> {
    let mut batch = ClientBatch::new(ClientBatchItem::new(request));
    if let Some(credentials) = connection.authentication.as_ref().and_then(|a| a.username_password.as_ref()) {
        let user = SecretText::new(std::env::var(&credentials.username_env).map_err(|_| "credential environment unavailable")?);
        let password = SecretText::new(std::env::var(&credentials.password_env).map_err(|_| "credential environment unavailable")?);
        let value = structure(0x420025, vec![item(0x420099, user.into_ttlv_value())?, item(0x4200a1, password.into_ttlv_value())?])?;
        let value = UsernameAndPasswordCredential::try_from_ttlv(value).map_err(|_| "invalid credential")?;
        let credential = Credential::try_from_ttlv(structure(0x420023, vec![
            item(0x420024, Value::enumeration(1))?, item(0x420025, Value::structure(value.into_ttlv()))?
        ])?).map_err(|_| "invalid credential")?;
        batch = batch.with_authentication(Authentication::new(vec![credential]).map_err(|_| "invalid authentication")?);
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
            if [0x420094, 0x420066, 0x42006f].contains(&value.tag().raw()) {
                identifier = value.with_value(|v| match v {
                    ValueView::TextString(v) => Some(kmipkit::protocol::UniqueIdentifier::TextString(v.to_string())),
                    _ => None,
                });
                if let Some(id) = &identifier { identifiers.push(id.clone()); }
            }
            if [0x4200c2, 0x4200c3, 0x42004f].contains(&value.tag().raw()) {
                bytes = value.with_value(|v| match v { ValueView::ByteString(v) => Some(kmipkit::protocol::SecretBytes::new(v.to_vec())), _ => None });
            }
            if value.tag().raw() == 0x420149 {
                ticket = value.as_structure().and_then(|v| v.try_clone().ok()).and_then(|v| kmipkit::protocol::Ticket::try_from_ttlv(v).ok());
            }
            if value.tag().raw() == 0x4200d6 {
                correlation = value.with_value(|v| match v { ValueView::ByteString(v) => Some(kmipkit::protocol::SecretBytes::new(v.to_vec())), _ => None });
            }
        }
    }
    if let kmipkit::ClientBatchOutcome::Pending(pending) = response.outcome() {
        correlation = Some(kmipkit::protocol::SecretBytes::new(pending.asynchronous_correlation_value().to_vec()));
    }
    Observed { status: result.status().raw(), reason: result.reason().map(kmipkit::ResultReason::raw), payload_tags, identifier, identifiers, correlation, bytes, ticket }
}

pub(crate) fn execute(client: &mut Client, connection: &Connection, request: ClientRequest) -> std::result::Result<Observed, kmipkit::ClientError> {
    let encoding = match connection.encoding.as_str() { "JSON" => kmipkit_transport::WireEncoding::Json, "XML" => kmipkit_transport::WireEncoding::Xml, _ => kmipkit_transport::WireEncoding::Ttlv };
    let options = RequestOptions::default().with_wire_encoding(encoding);
    let limits = kmipkit::ttlv::codec::CodecLimits::defaults();
    // Direct operation methods are used where the public high-level API admits
    // the same immutable request. Authentication requires the explicit batch.
    if connection.api_level == "high-level" && connection.authentication.is_none() {
        match request {
            ClientRequest::DiscoverVersions(request) => return client.discover_versions_with_options(request, &limits, &options).map(|response| observed(&response)),
            ClientRequest::Ping(_) => return client.ping_with_options(&limits, &options).map(|response| observed(&response)),
            ClientRequest::Query(request) => return client.query_with_options(request, &limits, &options).map(|response| observed(&response)),
            request => return execute_batch(client, connection, request, &options, &limits),
        }
    }
    execute_batch(client, connection, request, &options, &limits)
}

fn execute_batch(client: &mut Client, connection: &Connection, request: ClientRequest, options: &RequestOptions, limits: &kmipkit::ttlv::codec::CodecLimits) -> std::result::Result<Observed, kmipkit::ClientError> {
    let batch = authenticated_batch(connection, request).map_err(|_| kmipkit::ClientError::validation(kmipkit::ClientCauseCategory::InvalidInput, kmipkit::RequestDeliveryState::NotSent, std::io::Error::other("credential unavailable")))?;
    let response = client.execute_with_options(batch, limits, options)?;
    response.get(0).map(observed).ok_or_else(|| kmipkit::ClientError::validation(kmipkit::ClientCauseCategory::InvalidInput, kmipkit::RequestDeliveryState::ResponseStarted, std::io::Error::other("missing result")))
}

pub(crate) fn options(connection: &Connection) -> RequestOptions {
    RequestOptions::default().with_wire_encoding(match connection.encoding.as_str() {
        "JSON" => kmipkit_transport::WireEncoding::Json,
        "XML" => kmipkit_transport::WireEncoding::Xml,
        _ => kmipkit_transport::WireEncoding::Ttlv,
    })
}

pub(crate) fn batch(client: &mut Client, connection: &Connection, requests: Vec<ClientRequest>, asynchronous: bool) -> std::result::Result<Vec<Observed>, kmipkit::ClientError> {
    let mut batch = ClientBatch::from_items(requests.into_iter().map(ClientBatchItem::new)).with_batch_order_option(true);
    if asynchronous { batch = batch.with_asynchronous_indicator(1); }
    let response = client.execute_with_options(batch, &kmipkit::ttlv::codec::CodecLimits::defaults(), &options(connection))?;
    Ok(response.iter().map(observed).collect())
}

pub(crate) fn asynchronous(client: &mut Client, connection: &Connection, operation: &str, correlation: &[u8]) -> std::result::Result<Observed, kmipkit::ClientError> {
    use kmipkit::protocol::{CancelRequest, PollRequest, ProcessRequest, QueryAsyncRequestsRequest};
    let limits = kmipkit::ttlv::codec::CodecLimits::defaults();
    let options = options(connection);
    let response = match operation {
        "Cancel" => client.execute_cancel_with_options(CancelRequest::new(correlation), &limits, &options),
        "Process" => client.execute_process_with_options(ProcessRequest::new(correlation), None, &limits, &options),
        "Query Asynchronous Requests" => client.execute_query_async_requests_with_options(QueryAsyncRequestsRequest::new(), None, &limits, &options),
        _ => client.execute_poll_with_options(PollRequest::new(correlation), &limits, &options),
    }?;
    Ok(Observed { status: response.result().status().raw(), reason: response.result().reason().map(kmipkit::ResultReason::raw),
        payload_tags: Vec::new(), identifier: None, identifiers: Vec::new(), bytes: None, ticket: None, correlation: None })
}
