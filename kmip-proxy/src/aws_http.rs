//! AWS SDK HTTP transport configured with KMIPKit's OpenSSL rustls provider.
#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::io;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use aws_smithy_runtime_api::client::http::{
    HttpClient, HttpConnector, HttpConnectorFuture, HttpConnectorSettings, SharedHttpClient,
    SharedHttpConnector,
};
use aws_smithy_runtime_api::client::orchestrator::{HttpRequest, HttpResponse};
use aws_smithy_runtime_api::client::result::ConnectorError;
use aws_smithy_runtime_api::client::runtime_components::RuntimeComponents;
use aws_smithy_types::body::SdkBody;
use hyper_rustls::HttpsConnector;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector as TcpConnector;
use hyper_util::rt::TokioExecutor;
use rustls::crypto::CryptoProvider;
use rustls::{ClientConfig, RootCertStore};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct ConnectorSettingsKey {
    connect_timeout: Option<Duration>,
    read_timeout: Option<Duration>,
}

impl From<&HttpConnectorSettings> for ConnectorSettingsKey {
    fn from(settings: &HttpConnectorSettings) -> Self {
        Self {
            connect_timeout: settings.connect_timeout(),
            read_timeout: settings.read_timeout(),
        }
    }
}

#[derive(Debug, Clone)]
struct OpenSslConnector {
    client: Client<HttpsConnector<TcpConnector>, SdkBody>,
    read_timeout: Option<Duration>,
}

impl HttpConnector for OpenSslConnector {
    fn call(&self, request: HttpRequest) -> HttpConnectorFuture {
        let request = match request.try_into_http1x() {
            Ok(request) => request,
            Err(error) => {
                return HttpConnectorFuture::ready(Err(ConnectorError::user(error.into())));
            }
        };
        HttpConnectorFuture::new(send_request(
            self.client.clone(),
            request,
            self.read_timeout,
        ))
    }
}

async fn send_request(
    client: Client<HttpsConnector<TcpConnector>, SdkBody>,
    request: hyper::Request<SdkBody>,
    read_timeout: Option<Duration>,
) -> Result<HttpResponse, ConnectorError> {
    let request_future = client.request(request);
    let response = if let Some(timeout) = read_timeout {
        tokio::time::timeout(timeout, request_future)
            .await
            .map_err(|error| ConnectorError::timeout(Box::new(error)))?
    } else {
        request_future.await
    }
    .map_err(|error| ConnectorError::other(Box::new(error), None))?;

    HttpResponse::try_from(response.map(SdkBody::from_body_1_x))
        .map_err(|error| ConnectorError::other(Box::new(error), None))
}

#[derive(Debug)]
struct OpenSslHttpClient {
    provider: Arc<CryptoProvider>,
    connectors: Mutex<HashMap<ConnectorSettingsKey, SharedHttpConnector>>,
}

#[derive(Debug)]
struct FailedConnector(String);

impl HttpConnector for FailedConnector {
    fn call(&self, _request: HttpRequest) -> HttpConnectorFuture {
        HttpConnectorFuture::ready(Err(ConnectorError::other(
            Box::new(io::Error::other(self.0.clone())),
            None,
        )))
    }
}

impl HttpClient for OpenSslHttpClient {
    fn http_connector(
        &self,
        settings: &HttpConnectorSettings,
        _components: &RuntimeComponents,
    ) -> SharedHttpConnector {
        let key = ConnectorSettingsKey::from(settings);
        let mut connectors = self
            .connectors
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        connectors
            .entry(key)
            .or_insert_with(|| match build_connector(self.provider.clone(), key) {
                Ok(connector) => SharedHttpConnector::new(connector),
                Err(error) => SharedHttpConnector::new(FailedConnector(error)),
            })
            .clone()
    }
}

/// Builds a reusable AWS SDK HTTP client using OpenSSL through rustls-openssl.
pub(crate) fn http_client() -> Result<SharedHttpClient, String> {
    Ok(SharedHttpClient::new(OpenSslHttpClient {
        provider: Arc::new(rustls_openssl::default_provider()),
        connectors: Mutex::new(HashMap::new()),
    }))
}

fn build_connector(
    provider: Arc<CryptoProvider>,
    settings: ConnectorSettingsKey,
) -> Result<OpenSslConnector, String> {
    let mut roots = RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let mut tls_config = ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|error| format!("could not configure TLS versions for AWS HTTPS: {error}"))?
        .with_root_certificates(roots)
        .with_no_client_auth();
    tls_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

    let mut tcp = TcpConnector::new();
    tcp.enforce_http(false);
    tcp.set_connect_timeout(settings.connect_timeout);
    let https = HttpsConnector::from((tcp, tls_config));
    let client = Client::builder(TokioExecutor::new()).build::<_, SdkBody>(https);

    Ok(OpenSslConnector {
        client,
        read_timeout: settings.read_timeout,
    })
}
