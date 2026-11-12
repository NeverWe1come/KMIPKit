use kmipkit::{
    Client, CryptographicUsageMask,
    client_config::ClientConfiguration,
    protocol::UniqueIdentifier,
    transport_config::{
        CertificateInput, ClientIdentity, Endpoint, PrivateKeyInput, TransportConfig, TrustSource,
    },
    ttlv::codec::CodecLimits,
};

fn execute_fluent_key_lifecycle(
    client: &mut Client,
    limits: &CodecLimits,
) -> Result<(), Box<dyn std::error::Error>> {
    let created = client
        .request()
        .create_symmetric_key()
        .aes_256()
        .usage_mask(CryptographicUsageMask::ENCRYPT | CryptographicUsageMask::DECRYPT)
        .name("fluent-key")
        .execute(limits)?;
    let identifier = created
        .outcome()
        .create_response()
        .and_then(|response| response.unique_identifier())
        .cloned()
        .ok_or("Create did not return its typed Unique Identifier")?;
    assert_eq!(identifier, UniqueIdentifier::from("fluent-key"));

    let retrieved = client.request().get(identifier.clone()).execute(limits)?;
    let get_response = retrieved
        .outcome()
        .get_response()
        .ok_or("Get did not return a typed successful response")?;
    assert_eq!(get_response.result().status().raw(), 0);
    assert_eq!(get_response.unique_identifier(), Some(&identifier));
    assert!(get_response.object().is_some());
    let retrieved_identifier = get_response
        .unique_identifier()
        .cloned()
        .ok_or("Get did not return its typed Unique Identifier")?;

    let destroyed = client
        .request()
        .destroy(retrieved_identifier.clone())
        .execute(limits)?;
    let destroy_response = destroyed
        .outcome()
        .destroy_response()
        .ok_or("Destroy did not return a typed successful response")?;
    assert_eq!(destroy_response.result().status().raw(), 0);
    assert_eq!(destroy_response.unique_identifier(), Some(&retrieved_identifier));
    println!("rust: fluent Create/Get/Destroy journey verified");
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port: u16 = std::env::var("KMIPKIT_CONSUMER_TEST_PORT")?.parse()?;
    let fixtures = std::path::PathBuf::from(std::env::var("KMIPKIT_CONSUMER_TEST_FIXTURES")?);
    let registry = kmipkit::client_config::client_extension_registry(
        Vec::new(),
        kmipkit::protocol::extension::defaults(),
    )?;
    let configuration = ClientConfiguration::new(registry);
    let transport = TransportConfig::builder(Endpoint::raw_tls("127.0.0.1", port))
        .client_identity(ClientIdentity::new(
            CertificateInput::from_pem_file(fixtures.join("client-cert.pem"))?,
            PrivateKeyInput::from_pem_file(fixtures.join("client-key.pem"))?,
        ))
        .trust_source(TrustSource::certificate_authorities(vec![
            CertificateInput::from_pem_file(fixtures.join("server-cert.pem"))?,
        ]))
        .tls_server_name("localhost")
        .build()?;
    let mut client = Client::new(configuration, transport)?;
    execute_fluent_key_lifecycle(&mut client, &CodecLimits::defaults())
}
