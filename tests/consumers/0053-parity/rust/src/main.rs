use kmipkit::{
    Client, ClientBatch, ClientBatchItem, ClientBatchResponse, ClientError, ClientRequest,
    client_config::ClientConfiguration,
    protocol::{
        Attribute, Authentication, Credential, CryptographicParametersBuilder, KeyMaterial,
        ManagedObject, OpaqueObject, ResponseMessage, attribute::typed,
    },
    transport_config::TransportConfig,
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

fn structure(items: impl IntoIterator<Item = Item>) -> Result<Structure, Box<dyn std::error::Error>> {
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
    Ok(Authentication::new(vec![Credential::try_from_ttlv(credential)?])?)
}

fn execute_ping(
    client: &mut Client,
    limits: &CodecLimits,
) -> Result<ClientBatchResponse, ClientError> {
    client.execute(ClientBatch::new(ClientBatchItem::ping()), limits)
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

    let request = ClientRequest::ping();
    let validation_report = request.validation_report();
    assert!(validation_report.is_valid());
    let _authenticated_batch =
        ClientBatch::new(ClientBatchItem::ping()).with_authentication(demo_authentication()?);
    assert_eq!(ClientBatchItem::ping().operation(), 0x3B);

    let response = ResponseMessage::try_from_ttlv(synthetic_ping_response()?)?;
    assert_eq!(response.batch_items().len(), 1);
    assert_eq!(response.batch_items().next().and_then(|item| item.result_status()).map(|s| s.raw()), Some(0));
    assert_eq!(response.with_ttlv(|tree| tree.children().len()), 2);

    // The consumer prepares an operation and validates it locally. No server is contacted.
    let _client_builder: fn(ClientConfiguration, TransportConfig) -> Result<Client, ClientError> =
        Client::new;
    let _exchange: fn(&mut Client, &CodecLimits) -> Result<ClientBatchResponse, ClientError> =
        execute_ping;
    Ok(())
}
