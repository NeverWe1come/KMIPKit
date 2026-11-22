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
    client: reqwest::Client,
}

impl HttpConnector for OpenSslConnector {
    fn call(&self, request: HttpRequest) -> HttpConnectorFuture {
        let request = match request.try_into_http1x() {
            Ok(request) => request,
            Err(error) => {
                return HttpConnectorFuture::ready(Err(ConnectorError::user(error.into())));
            }
        };
        let request = match reqwest::Request::try_from(request.map(reqwest::Body::wrap)) {
            Ok(request) => request,
            Err(error) => {
                return HttpConnectorFuture::ready(Err(ConnectorError::user(error.into())));
            }
        };
        let client = self.client.clone();
        HttpConnectorFuture::new(async move {
            let response = client.execute(request).await.map_err(|error| {
                let error = error.without_url();
                if error.is_timeout() {
                    ConnectorError::timeout(error.into())
                } else if error.is_request() || error.is_body() {
                    ConnectorError::io(error.into())
                } else {
                    ConnectorError::other(error.into(), None)
                }
            })?;
            let response: http::Response<reqwest::Body> = response.into();
            HttpResponse::try_from(response.map(SdkBody::from_body_1_x))
                .map_err(|error| ConnectorError::other(error.into(), None))
        })
    }
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
    let native_certs = rustls_native_certs::load_native_certs();
    roots.add_parsable_certificates(native_certs.certs);
    let mut tls_config = ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|error| format!("could not configure TLS versions for AWS HTTPS: {error}"))?
        .with_root_certificates(roots)
        .with_no_client_auth();
    tls_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    // Keep AWS signatures scoped to the original URL and let Smithy own retries.
    let mut builder = reqwest::Client::builder()
        .use_preconfigured_tls(tls_config)
        .redirect(reqwest::redirect::Policy::none())
        .retry(reqwest::retry::never());
    if let Some(timeout) = settings.connect_timeout {
        builder = builder.connect_timeout(timeout);
    }
    if let Some(timeout) = settings.read_timeout {
        builder = builder.read_timeout(timeout);
    }
    builder
        .build()
        .map(|client| OpenSslConnector { client })
        .map_err(|error| format!("could not configure AWS HTTP client: {error}"))
}
