//! Opt-in smoke test against a locally deployed Cosmian KMS 5.28.0.
//!
//! This test covers only Discover Versions (OASIS KMIP Specification v2.1
//! §6.1.16, Tables 211–213) and Protocol Version encoding (§9.16, Table 421).
//! It is not a full KMIP conformance or profile test.

use std::error::Error;
use std::path::{Path, PathBuf};

use kmipkit_client::extension_registry::{ClientConfiguration, client_extension_registry};
use kmipkit_client::{Client, ClientBatch, ClientBatchItem, ClientRequest};
use kmipkit_protocol::{ProtocolVersion, extension};
use kmipkit_transport::{
    CertificateInput, ClientIdentity, Endpoint, PrivateKeyInput, TransportConfig, TrustSource,
};
use kmipkit_ttlv::codec::CodecLimits;

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
fn cosmian_kms_accepts_kmip_2_1_discover_versions_over_mutual_tls() -> Result<(), Box<dyn Error>> {
    let certificate_directory = certificate_directory();
    let transport_configuration = TransportConfig::builder(Endpoint::raw_tls("127.0.0.1", 5696))
        .tls_server_name("localhost")
        .client_identity(ClientIdentity::new(
            CertificateInput::from_pem_file(certificate_directory.join("client/client.crt"))?,
            PrivateKeyInput::from_pem_file(certificate_directory.join("client/client.key"))?,
        ))
        .trust_source(TrustSource::certificate_authorities(vec![
            CertificateInput::from_pem_file(certificate_directory.join("client/ca.crt"))?,
        ]))
        .build()?;

    let registry = client_extension_registry(Vec::new(), extension::defaults())?;
    let mut client = Client::new(ClientConfiguration::new(registry), transport_configuration)?;
    let response = client.execute(
        ClientBatch::new(ClientBatchItem::new(ClientRequest::discover_versions())),
        &CodecLimits::defaults(),
    )?;

    assert_eq!(
        response.len(),
        1,
        "Cosmian returns one Discover Versions result"
    );
    let item = response
        .get(0)
        .ok_or("Cosmian response has no Discover Versions result")?;
    let discover_versions = item.outcome().response();

    assert_eq!(
        discover_versions.result().status().raw(),
        0,
        "Cosmian reports KMIP Result Status Success"
    );
    let supported_versions = discover_versions
        .supported_versions()
        .ok_or("successful Discover Versions response has no version list")?;
    assert!(
        supported_versions.contains(&ProtocolVersion::from_raw(2, 1)),
        "Cosmian reports the KMIP 2.1 version offered by KMIPKit"
    );

    Ok(())
}

fn certificate_directory() -> PathBuf {
    std::env::var_os("KMIPKIT_COSMIAN_CERT_DIR").map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.local/cosmian-kms/certs"),
        PathBuf::from,
    )
}
