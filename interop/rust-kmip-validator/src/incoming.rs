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
    CertificateInput, ClientIdentity, ListenerTransportConfig, PrivateKeyInput, TrustSource,
};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

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
) -> Result<Option<ClientListener>> {
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
    let listener = if connection.transport == "https" {
        ClientListener::bind_https(config)
    } else {
        ClientListener::bind_raw_tls(config)
    }
    .map_err(|_| "listener bind failed")?;
    if let Some(path) = &options.listener_ready_file {
        std::fs::write(
            path,
            listener
                .local_addr()
                .map_err(|_| "listener address unavailable")?
                .to_string(),
        )
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

pub(crate) fn run(
    case: &Case,
    connection: &Connection,
    listener: Option<&mut ClientListener>,
) -> CaseResult {
    let Some(listener) = listener else {
        return CaseResult::non_run(case, "not_run", "server-to-client requires --listen and an incoming peer in this environment; OASIS KMIP 2.1 §6.2".to_owned());
    };
    if connection.transport == "raw_tls" && connection.encoding != "TTLV" {
        return CaseResult::non_run(case, "not_run", "library_api_unavailable: raw TLS listener hardcodes TTLV decoding; OASIS KMIP 2.1 §6.2".to_owned());
    }
    let version = if connection.protocol_version == "2.0" {
        ProtocolVersion::V2_0
    } else {
        ProtocolVersion::V2_1
    };
    let timeout = Some(Duration::from_secs(
        connection
            .tls
            .as_ref()
            .and_then(|v| v.read_timeout_seconds)
            .unwrap_or(30),
    ));
    let result = (|| {
        let mut channel = listener
            .accept(timeout)
            .map_err(|_| "incoming connection unavailable")?;
        let batch = channel
            .recv_batch(&kmipkit::ttlv::codec::CodecLimits::defaults(), timeout)
            .map_err(|_| "incoming message rejected")?;
        let expected_operation = match case.operation.as_str() {
            "Discover Versions" => 0x1e,
            "Notify" => 0x1b,
            "Put" => 0x1c,
            "Query" => 0x18,
            "Set Endpoint Role" => 0x32,
            _ => return Err("unknown incoming operation"),
        };
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
        Ok(())
    })();
    let mut output = CaseResult::non_run(
        case,
        if result.is_ok() { "passed" } else { "failed" },
        result
            .err()
            .unwrap_or("incoming typed handler and response completed")
            .to_owned(),
    );
    output.library_correctness = if result.is_ok() {
        "passed"
    } else {
        "not_checked"
    };
    output.server_compatibility = if result.is_ok() {
        "supported"
    } else {
        "unknown"
    };
    output
}
