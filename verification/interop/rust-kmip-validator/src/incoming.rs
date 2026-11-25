//! Incoming KMIP operations through the installed public listener and dispatcher.
use crate::models::{Case, CaseResult, Connection, Result};
use kmipkit::protocol::{
    KmipOperationResult, ProtocolVersion, ServerCapabilities, ServerDiscoverVersionsRequest,
    ServerDiscoverVersionsResponse, ServerNotifyRequest, ServerNotifyResponse, ServerPutRequest,
    ServerPutResponse, ServerQueryRequest, ServerQueryResponse, SetEndpointRoleRequest,
    SetEndpointRoleResponse,
};
use kmipkit::{ClientListener, ServerRequestDispatcher, ServerRequestHandler};
use kmipkit_transport::{
    CertificateInput, ClientIdentity, ListenerTransportConfig, PrivateKeyInput, RawTlsListener,
    TrustSource,
};
use std::io::Write;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

mod generic;

const PEER_UNAVAILABLE: &str = "incoming peer was not available before the listener deadline";

pub(crate) enum Listener {
    Typed(ClientListener),
    Generic(RawTlsListener),
}

impl Listener {
    fn local_addr(&self) -> Result<SocketAddr> {
        match self {
            Self::Typed(listener) => listener.local_addr(),
            Self::Generic(listener) => listener.local_addr(),
        }
        .map_err(|_| "listener address unavailable")
    }
}

#[derive(Default, clap::Args)]
pub(crate) struct ListenerOptions {
    /// Local address for server-to-client cases. Configure the test server to call this address.
    #[arg(long)]
    pub listen: Option<SocketAddr>,
    /// Listener certificate with server authentication EKU.
    #[arg(long, requires = "listen")]
    pub listener_cert: Option<PathBuf>,
    /// Listener key; never included in a report.
    #[arg(long, requires = "listen")]
    pub listener_key: Option<PathBuf>,
    /// Write the selected local socket address so a controlled server can connect.
    #[arg(long, requires = "listen")]
    pub listener_ready_file: Option<PathBuf>,
}

pub(crate) fn listener(
    connection: &Connection,
    options: &ListenerOptions,
) -> Result<Option<Listener>> {
    let Some(address) = options.listen else {
        return Ok(None);
    };
    let tls = connection.tls.as_ref().ok_or("TLS identity required")?;
    let cert = options
        .listener_cert
        .as_deref()
        .unwrap_or_else(|| std::path::Path::new(&tls.client_cert_path));
    let key = options
        .listener_key
        .as_deref()
        .unwrap_or_else(|| std::path::Path::new(&tls.client_key_path));
    let config = ListenerTransportConfig::builder(address.ip().to_string(), address.port())
        .listener_identity(ClientIdentity::new(
            CertificateInput::from_pem_file(cert).map_err(|_| "invalid listener certificate")?,
            PrivateKeyInput::from_pem_file(key).map_err(|_| "invalid listener key")?,
        ))
        .trust_source(TrustSource::certificate_authorities(vec![
            CertificateInput::from_pem_file(&tls.ca_cert_path)
                .map_err(|_| "invalid listener CA")?,
        ]))
        .build()
        .map_err(|_| "invalid incoming-channel configuration")?;
    let listener = if connection.api_level == "generic-ttlv" {
        if connection.transport != "raw_tls"
            || connection.encoding != "TTLV"
            || !matches!(connection.protocol_version.as_str(), "2.0" | "2.1")
        {
            return Err("generic incoming listener requires KMIP 2.x TTLV over raw TLS");
        }
        RawTlsListener::bind(config)
            .map(Listener::Generic)
            .map_err(|_| "raw TLS listener bind failed")
    } else if connection.transport == "https" {
        ClientListener::bind_https(config)
            .map(Listener::Typed)
            .map_err(|_| "HTTPS listener bind failed")
    } else {
        ClientListener::bind_raw_tls(config)
            .map(Listener::Typed)
            .map_err(|_| "typed TLS listener bind failed")
    }?;
    if let Some(path) = &options.listener_ready_file {
        let address = listener.local_addr()?;
        std::fs::File::create_new(path)
            .and_then(|mut file| write!(file, "{address}"))
            .map_err(|_| "listener readiness output unavailable")?;
    }
    Ok(Some(listener))
}

struct Handler {
    version: ProtocolVersion,
    received: bool,
}
impl ServerRequestHandler for Handler {
    fn handle_discover_versions(
        &mut self,
        request: &ServerDiscoverVersionsRequest,
    ) -> std::result::Result<ServerDiscoverVersionsResponse, KmipOperationResult> {
        self.received = true;
        Ok(ServerDiscoverVersionsResponse::negotiate(
            request,
            &[self.version],
        ))
    }
    fn handle_notify(
        &mut self,
        _request: &ServerNotifyRequest,
    ) -> std::result::Result<ServerNotifyResponse, KmipOperationResult> {
        self.received = true;
        Ok(ServerNotifyResponse::new())
    }
    fn handle_put(
        &mut self,
        _request: &ServerPutRequest,
    ) -> std::result::Result<ServerPutResponse, KmipOperationResult> {
        self.received = true;
        // The validator accepts into the request's bounded zeroizing owner;
        // the material is discarded when the isolated attempt ends.
        Ok(ServerPutResponse::new())
    }
    fn handle_query(
        &mut self,
        request: &ServerQueryRequest,
    ) -> std::result::Result<ServerQueryResponse, KmipOperationResult> {
        self.received = true;
        let mut caps = ServerCapabilities::new();
        caps.set_operations(vec![0x1e, 0x1b, 0x1c, 0x18, 0x32]);
        Ok(ServerQueryResponse::respond(request, &caps))
    }
    fn handle_set_endpoint_role(
        &mut self,
        request: &SetEndpointRoleRequest,
    ) -> std::result::Result<SetEndpointRoleResponse, KmipOperationResult> {
        self.received = true;
        Ok(SetEndpointRoleResponse::success(
            request.endpoint_role(),
            None,
        ))
    }
}

pub(crate) fn route_available(connection: &Connection) -> bool {
    match connection.api_level.as_str() {
        "fluent" => false,
        "generic-ttlv" => {
            connection.encoding == "TTLV"
                && connection.transport == "raw_tls"
                && matches!(connection.protocol_version.as_str(), "2.0" | "2.1")
        }
        _ => connection.encoding == "TTLV" || connection.transport == "https",
    }
}

pub(crate) fn run(
    case: &Case,
    connection: &Connection,
    listener: Option<&mut Listener>,
) -> CaseResult {
    if !case
        .execution
        .protocol_versions
        .contains(&connection.protocol_version)
    {
        return CaseResult::non_run(
            case,
            "not_run",
            "incoming case is outside the configured protocol version".to_owned(),
        );
    }
    if case.execution.mutation != "read_only" && !connection.execution_policy.allow_mutations {
        return CaseResult::non_run(
            case,
            "policy_restricted",
            "incoming mutation requires explicit opt-in".to_owned(),
        );
    }
    if !route_available(connection) {
        return CaseResult::non_run(
            case,
            "not_run",
            "no Rust incoming recipe for the selected API, version or transport".to_owned(),
        );
    }
    let Some(listener) = listener else {
        return CaseResult::non_run(case, "not_run", "server-to-client requires --listen and an incoming peer in this environment; OASIS KMIP 2.1 §6.2".to_owned());
    };
    let version = match connection.protocol_version.as_str() {
        "2.0" => ProtocolVersion::V2_0,
        "3.0" => ProtocolVersion::V3_0,
        _ => ProtocolVersion::V2_1,
    };
    let timeout = Some(Duration::from_secs(
        connection
            .tls
            .as_ref()
            .and_then(|v| v.read_timeout_seconds)
            .unwrap_or(30),
    ));
    let expected_operation = match case.operation.as_str() {
        "Discover Versions" => 0x1e,
        "Notify" => 0x1b,
        "Put" => 0x1c,
        "Query" => 0x18,
        "Set Endpoint Role" => 0x32,
        _ => return CaseResult::non_run(case, "not_run", "unknown incoming operation".to_owned()),
    };
    let result = match listener {
        Listener::Generic(listener) => generic::run(listener, version, expected_operation, timeout),
        Listener::Typed(listener) => typed(listener, version, expected_operation, timeout),
    };
    match result {
        Ok(0) => {
            let mut evidence = CaseResult::non_run(
                case,
                "passed",
                "incoming request validated and response sent".to_owned(),
            );
            evidence.library_correctness = "passed";
            evidence.server_compatibility = "supported";
            evidence.result_status_code = Some(0);
            evidence
        }
        Ok(status) => {
            let mut evidence = CaseResult::non_run(
                case,
                "failed",
                "peer sent an invalid payload; Invalid Field sent".to_owned(),
            );
            evidence.result_status_code = Some(status);
            evidence.result_reason_code = Some(7);
            evidence
        }
        Err(reason) => CaseResult::non_run(
            case,
            if reason == PEER_UNAVAILABLE {
                "not_run"
            } else {
                "failed"
            },
            reason.to_owned(),
        ),
    }
}

fn typed(
    listener: &mut ClientListener,
    version: ProtocolVersion,
    expected_operation: u32,
    timeout: Option<Duration>,
) -> Result<u32> {
    let mut channel = listener.accept(timeout).map_err(|_| PEER_UNAVAILABLE)?;
    let batch = channel
        .recv_batch(&kmipkit::ttlv::codec::CodecLimits::defaults(), timeout)
        .map_err(|_| "incoming message rejected")?;
    if batch.protocol_version() != version
        || batch.batch_count() != 1
        || batch.items()[0].operation() != expected_operation
    {
        return Err("incoming operation/version mismatch");
    }
    let mut handler = Handler {
        version,
        received: false,
    };
    let response = ServerRequestDispatcher::new()
        .dispatch(&batch, &mut handler)
        .map_err(|_| "incoming dispatch failed")?;
    channel
        .send_response(&response, timeout)
        .map_err(|_| "incoming response delivery failed")?;
    channel
        .close()
        .map_err(|_| "incoming channel close failed")?;
    if !handler.received {
        return Err("incoming handler was not selected");
    }
    Ok(0)
}
