//! Opt-in probes for credential-session operations against Cosmian KMS 5.28.0.
//!
//! These probes exercise OASIS KMIP 2.1 §6.1.12 Tables 199–201, §6.1.30
//! Tables 253–255, and §6.1.31 Tables 256–258 over the real binary TLS
//! transport. §8.6, Table 399 requires Operation in a response batch item when
//! the request batch item includes it. Cosmian 5.28.0 omits that field in its
//! failure response; these tests record the interoperability failure.

use std::error::Error;
use std::path::{Path, PathBuf};

use kmipkit_client::extension_registry::{ClientConfiguration, client_extension_registry};
use kmipkit_client::{Client, ClientBatch, ClientBatchItem, ClientErrorCategory, ClientRequest};
use kmipkit_protocol::extension;
use kmipkit_protocol::{
    DelegatedLoginRequest, LoginRequest, LogoutRequest, SecretBytes, Ticket, TicketType,
};
use kmipkit_transport::{
    CertificateInput, ClientIdentity, Endpoint, PrivateKeyInput, TransportConfig, TrustSource,
};
use kmipkit_ttlv::Structure;
use kmipkit_ttlv::codec::CodecLimits;

#[test]
#[ignore = "requires local Cosmian KMS 5.28.0; see docs/development/kmip-interoperability.md"]
fn cosmian_kms_delegated_login_error_omits_required_operation_over_mutual_tls()
-> Result<(), Box<dyn Error>> {
    let mut client = client()?;
    assert_rejected_without_operation(
        &mut client,
        ClientRequest::delegated_login(DelegatedLoginRequest::new(Structure::new())),
        "Delegated Login",
    )
}

#[test]
#[ignore = "requires local Cosmian KMS 5.28.0; see docs/development/kmip-interoperability.md"]
fn cosmian_kms_login_error_omits_required_operation_over_mutual_tls() -> Result<(), Box<dyn Error>>
{
    let mut client = client()?;
    assert_rejected_without_operation(
        &mut client,
        ClientRequest::login(LoginRequest::new()),
        "Login",
    )
}

#[test]
#[ignore = "requires local Cosmian KMS 5.28.0; see docs/development/kmip-interoperability.md"]
fn cosmian_kms_logout_error_omits_required_operation_over_mutual_tls() -> Result<(), Box<dyn Error>>
{
    let mut client = client()?;
    let ticket = Ticket::new(
        TicketType::LOGIN,
        SecretBytes::new(b"synthetic-credential-session-ticket".to_vec()),
    )?;
    assert_rejected_without_operation(
        &mut client,
        ClientRequest::logout(LogoutRequest::new(ticket)),
        "Logout",
    )
}

fn assert_rejected_without_operation(
    client: &mut Client,
    request: ClientRequest,
    operation_name: &str,
) -> Result<(), Box<dyn Error>> {
    let Err(error) = client.execute(
        ClientBatch::new(ClientBatchItem::new(request)),
        &CodecLimits::defaults(),
    ) else {
        return Err("Cosmian unexpectedly returned a parsed operation result".into());
    };
    assert_eq!(
        error.delivery_state(),
        Some(kmipkit_transport::RequestDeliveryState::ResponseStarted)
    );
    assert_eq!(error.category(), ClientErrorCategory::Protocol);
    assert!(error.response_ttlv().is_none());
    println!(
        "Cosmian KMS 5.28.0 {operation_name}: malformed Failure response rejected without retaining its body"
    );
    Ok(())
}

fn client() -> Result<Client, Box<dyn Error>> {
    let certificate_directory = certificate_directory();
    let transport = TransportConfig::builder(Endpoint::raw_tls("127.0.0.1", 5696))
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
    Ok(Client::new(ClientConfiguration::new(registry), transport)?)
}

fn certificate_directory() -> PathBuf {
    std::env::var_os("KMIPKIT_COSMIAN_CERT_DIR").map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.local/cosmian-kms/certs"),
        PathBuf::from,
    )
}
