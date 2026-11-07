use kmipkit::{
    Client, ClientBatch, ClientBatchItem, ClientBatchResponse, ClientError, ClientRequest,
    client_config::ClientConfiguration,
    protocol::{
        Attribute, Authentication, Credential, CryptographicParametersBuilder, KeyMaterial,
        ManagedObject, OpaqueObject, ResponseMessage, attribute::typed,
    },
    transport_config::{
        CertificateInput, ClientIdentity, Endpoint, PrivateKeyInput, TransportConfig, TrustSource,
    },
    ttlv::{Item, RawTag, Structure, Value, codec::CodecLimits},
};

const RESPONSE_HEADER: u32 = 0x0042_007A;
const PROTOCOL_VERSION: u32 = 0x0042_0069;
const VERSION_MAJOR: u32 = 0x0042_006A;
const VERSION_MINOR: u32 = 0x0042_006B;
const TIME_STAMP: u32 = 0x0042_0092;
const BATCH_COUNT: u32 = 0x0042_000D;
const BATCH_ITEM: u32 = 0x0042_000F;
const OPERATION: u32 = 0x0042_005C;
const RESULT_STATUS: u32 = 0x0042_007F;
const RESPONSE_PAYLOAD: u32 = 0x0042_007C;

fn item(raw_tag: u32, value: Value) -> Result<Item, Box<dyn std::error::Error>> {
    Ok(Item::new(RawTag::new(raw_tag)?.try_checked()?, value)?)
}

fn structure(
    items: impl IntoIterator<Item = Item>,
) -> Result<Structure, Box<dyn std::error::Error>> {
    let mut result = Structure::new();
    for child in items {
        result.try_push(child)?;
    }
    Ok(result)
}

fn synthetic_ping_response() -> Result<Structure, Box<dyn std::error::Error>> {
    let version = structure([
        item(VERSION_MAJOR, Value::integer(2))?,
        item(VERSION_MINOR, Value::integer(1))?,
    ])?;
    let header = structure([
        item(PROTOCOL_VERSION, Value::structure(version))?,
        item(TIME_STAMP, Value::date_time(1_751_100_123))?,
        item(BATCH_COUNT, Value::integer(1))?,
    ])?;
    let response_item = structure([
        item(OPERATION, Value::enumeration(0x3B))?,
        item(RESULT_STATUS, Value::enumeration(0))?,
        item(RESPONSE_PAYLOAD, Value::structure(Structure::new()))?,
    ])?;
    structure([
        item(RESPONSE_HEADER, Value::structure(header))?,
        item(BATCH_ITEM, Value::structure(response_item))?,
    ])
}

fn demo_authentication() -> Result<Authentication, Box<dyn std::error::Error>> {
    let credential_value = structure([item(
        0x0042_00BB,
        Value::text_string("demo-user".to_owned()),
    )?])?;
    let credential = structure([
        item(0x0042_0024, Value::enumeration(1))?,
        item(0x0042_0025, Value::structure(credential_value))?,
    ])?;
    Ok(Authentication::new(vec![Credential::try_from_ttlv(
        credential,
    )?])?)
}

fn execute_ping(
    client: &mut Client,
    limits: &CodecLimits,
) -> Result<ClientBatchResponse, ClientError> {
    client.execute(ClientBatch::new(ClientBatchItem::ping()), limits)
}

fn demonstrate_client_request() -> Result<(), Box<dyn std::error::Error>> {
    let request = ClientRequest::ping();
    let validation_report = request.validation_report();
    assert!(validation_report.is_valid());
    let _authenticated_batch =
        ClientBatch::new(ClientBatchItem::ping()).with_authentication(demo_authentication()?);
    assert_eq!(ClientBatchItem::ping().operation(), 0x3B);
    Ok(())
}

fn inspect_synthetic_response() -> Result<(), Box<dyn std::error::Error>> {
    let response = ResponseMessage::try_from_ttlv(synthetic_ping_response()?)?;
    assert_eq!(response.batch_items().len(), 1);
    assert_eq!(
        response
            .batch_items()
            .next()
            .and_then(|item| item.result_status())
            .map(|status| status.raw()),
        Some(0)
    );
    assert_eq!(response.with_ttlv(|tree| tree.children().len()), 2);
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let attribute = Attribute::<typed::Comment>::new("local example".to_owned())?;
    assert_eq!(attribute.with_value(str::to_owned)?, "local example");

    let opaque = ManagedObject::OpaqueObject(OpaqueObject::try_new(1, b"demo bytes".to_vec())?);
    assert_eq!(opaque.object_type().raw(), 8);
    let key_material = KeyMaterial::byte_string(1, b"demo key bytes".to_vec())?;
    assert_eq!(key_material.format_raw(), 1);

    let parameters = CryptographicParametersBuilder::new()
        .with_cryptographic_algorithm(0x8000_0001)
        .build()?;
    assert_eq!(parameters.cryptographic_algorithm(), Some(0x8000_0001));

    demonstrate_client_request()?;
    inspect_synthetic_response()?;

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
    let limits = CodecLimits::defaults();
    let result = execute_ping(&mut client, &limits)?;
    let item = result.get(0).ok_or("Ping response item is missing")?;
    assert!(item.outcome().response().ping().is_some());
    let raw = item.raw().ok_or("Ping Response Payload is missing")?;
    assert_eq!(raw.children().len(), 0);
    println!("rust: Ping result and raw TTLV verified");
    Ok(())
}
