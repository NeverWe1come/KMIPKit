use kmipkit::{
    Client, ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientBatchResponse, ClientError,
    ClientRequest,
    client_config::ClientConfiguration,
    protocol::{
        Attribute, Authentication, Credential, CryptographicAlgorithm,
        CryptographicParametersBuilder, KeyMaterial, ManagedObject, OpaqueObject, OperationData,
        GetRequest, ResponseMessage, SecretBytes, UniqueIdentifier, attribute::typed,
    },
    transport_config::{
        CertificateInput, ClientIdentity, Endpoint, PrivateKeyInput, TransportConfig, TrustSource,
    },
    ttlv::{Item, RawTag, Structure, Value, codec::CodecLimits},
};
use kmipkit_client::fluent::CryptographicUsageMask;

const FLUENT_PLAINTEXT: &[u8] = b"fluent plaintext";
const FLUENT_CIPHERTEXT: &[u8] = b"fixture ciphertext";
const FLUENT_NONCE: &[u8] = b"nonce-12byte";
const FLUENT_AAD: &[u8] = b"fluent context";
const FLUENT_AUTH_TAG: &[u8] = b"fixture-auth-tag";

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

fn execute_fluent_lifecycle(client: &mut Client) -> Result<(), Box<dyn std::error::Error>> {
    let created = client
        .request()
        .create_symmetric_key()
        .aes_256()
        .usage_mask(CryptographicUsageMask::ENCRYPT | CryptographicUsageMask::DECRYPT)
        .name("fluent-key")
        .execute()?;
    let identifier = created
        .unique_identifier()
        .ok_or("Create response did not return its identifier")?
        .clone();

    let retrieved = client.request().get(identifier.clone()).execute()?;
    if retrieved.result().status().raw() != 0
        || retrieved.unique_identifier() != Some(&identifier)
        || retrieved.object().is_none()
    {
        return Err("unexpected Get outcome".into());
    }

    let aes_gcm_parameters = || {
        CryptographicParametersBuilder::new()
            .with_cryptographic_algorithm(
                kmipkit::protocol::named_values_generated::values::enumeration::cryptographic_algorithm::AES,
            )
            .with_block_cipher_mode(
                kmipkit::protocol::named_values_generated::values::enumeration::block_cipher_mode::GCM,
            )
            .with_iv_length(96)
            .with_tag_length(16)
            .build()
    };
    let encrypt_response = client
        .request()
        .encrypt()
        .unique_identifier(identifier.clone())
        .cryptographic_parameters(aes_gcm_parameters()?)
        .data(FLUENT_PLAINTEXT.to_vec())
        .iv_counter_nonce(FLUENT_NONCE.to_vec())
        .authenticated_encryption_additional_data(FLUENT_AAD.to_vec())
        .execute()?;
    if encrypt_response.result().status().raw() != 0
        || encrypt_response.unique_identifier() != Some(&identifier)
        || !encrypt_response
            .data()
            .is_some_and(|value| value.with_bytes(|actual| actual == FLUENT_CIPHERTEXT))
        || !encrypt_response
            .iv_counter_nonce()
            .is_some_and(|value| value.with_bytes(|actual| actual == FLUENT_NONCE))
        || !encrypt_response
            .authenticated_encryption_tag()
            .is_some_and(|value| value.with_bytes(|actual| actual == FLUENT_AUTH_TAG))
    {
        return Err("Encrypt response did not preserve the fixture's typed GCM fields".into());
    }

    let decrypt_response = client
        .request()
        .decrypt()
        .unique_identifier(identifier.clone())
        .cryptographic_parameters(aes_gcm_parameters()?)
        .data_value(OperationData::ByteString(
            encrypt_response
                .data()
                .ok_or("Encrypt response omitted ciphertext")?
                .with_bytes(|value| SecretBytes::new(value.to_vec())),
        ))
        .iv_counter_nonce(
            encrypt_response
                .iv_counter_nonce()
                .ok_or("Encrypt response omitted its nonce")?
                .with_bytes(<[u8]>::to_vec),
        )
        .authenticated_encryption_additional_data(FLUENT_AAD.to_vec())
        .authenticated_encryption_tag(
            encrypt_response
                .authenticated_encryption_tag()
                .ok_or("Encrypt response omitted its authentication tag")?
                .with_bytes(<[u8]>::to_vec),
        )
        .execute()?;
    if decrypt_response.result().status().raw() != 0
        || decrypt_response.unique_identifier() != Some(&identifier)
        || !decrypt_response
            .data()
            .is_some_and(|value| value.with_bytes(|actual| actual == FLUENT_PLAINTEXT))
    {
        return Err("Decrypt response did not preserve the fixture's typed plaintext".into());
    }

    let destroyed = client.request().destroy(identifier.clone()).execute()?;
    if destroyed.result().status().raw() != 0 || destroyed.unique_identifier() != Some(&identifier) {
        return Err("unexpected Destroy outcome".into());
    }
    println!("rust: fluent AES-GCM Create/Get/Encrypt/Decrypt/Destroy verified over TLS 1.3/mTLS");
    Ok(())
}

fn execute_fluent_key_pair_signing(client: &mut Client) -> Result<(), Box<dyn std::error::Error>> {
    let created = client
        .request()
        .create_key_pair()
        .algorithm(CryptographicAlgorithm::from_raw(6))
        .length(256)
        .private_usage_mask(CryptographicUsageMask::SIGN)
        .public_usage_mask(CryptographicUsageMask::VERIFY)
        .private_name("fluent-private-key")
        .public_name("fluent-public-key")
        .execute()?;
    if created.result().status().raw() != 0 {
        return Err("unexpected Create Key Pair status".into());
    }
    let private_identifier = created
        .private_key_unique_identifier()
        .ok_or("Create Key Pair omitted its private identifier")?
        .clone();
    let public_identifier = created
        .public_key_unique_identifier()
        .ok_or("Create Key Pair omitted its public identifier")?
        .clone();

    let parameters = CryptographicParametersBuilder::new()
        .with_hashing_algorithm(6)
        .with_digital_signature_algorithm(0x0E)
        .build()?;
    let signature = client
        .request()
        .sign()
        .unique_identifier(private_identifier.clone())
        .cryptographic_parameters(parameters)
        .data(b"fluent sign input".to_vec())
        .execute()?;
    if signature.result().status().raw() != 0
        || signature.unique_identifier() != Some(&private_identifier)
        || !signature
            .signature_data()
            .is_some_and(|value| value.with_bytes(|actual| actual == b"fixture signature"))
    {
        return Err("Sign did not preserve the fixture's typed Signature Data".into());
    }

    for identifier in [private_identifier, public_identifier] {
        let destroyed = client.request().destroy(identifier.clone()).execute()?;
        if destroyed.result().status().raw() != 0
            || destroyed.unique_identifier() != Some(&identifier)
        {
            return Err("unexpected key-pair Destroy outcome".into());
        }
    }
    println!("rust: fluent ECDSA key pair, Sign, and Destroy verified over TLS 1.3/mTLS");
    Ok(())
}

fn execute_fluent_expert_attributes(client: &mut Client) -> Result<(), Box<dyn std::error::Error>> {
    let attribute_value = item(
        0x0042_000B,
        Value::text_string("fluent expert journey".to_owned()),
    )?;
    let created = client
        .request()
        .create_symmetric_key()
        .aes_256()
        .usage_mask(CryptographicUsageMask::ENCRYPT | CryptographicUsageMask::DECRYPT)
        .name("fluent-expert-key")
        .vendor_attribute("KMIPKit.Example", "purpose", &attribute_value)
        .execute()?;
    if created.result().status().raw() != 0 || created.object_type().is_none() {
        return Err("unexpected expert Create outcome".into());
    }
    let identifier = created
        .unique_identifier()
        .ok_or("expert Create response omitted its identifier")?
        .clone();
    if identifier != kmipkit::protocol::UniqueIdentifier::from("fluent-expert-key") {
        return Err("expert Create did not return its requested key identifier".into());
    }

    let retrieved = client.request().get(identifier.clone()).execute()?;
    if retrieved.result().status().raw() != 0
        || retrieved.unique_identifier() != Some(&identifier)
        || retrieved.object().is_none()
    {
        return Err("unexpected expert Get outcome".into());
    }

    let destroyed = client.request().destroy(identifier.clone()).execute()?;
    if destroyed.result().status().raw() != 0 || destroyed.unique_identifier() != Some(&identifier) {
        return Err("unexpected expert Destroy outcome".into());
    }
    println!("rust: fluent Vendor Attribute Create/Get/Destroy verified over TLS 1.3/mTLS");
    Ok(())
}

fn execute_fluent_batch_placeholder(
    client: &mut Client,
    limits: &CodecLimits,
) -> Result<(), Box<dyn std::error::Error>> {
    let create = client
        .request()
        .create_symmetric_key()
        .aes_256()
        .usage_mask(CryptographicUsageMask::ENCRYPT | CryptographicUsageMask::DECRYPT)
        .name("fluent-batch-key")
        .to_batch_item()?
        .with_unique_batch_item_id(b"fluent-batch-create".to_vec());
    let get = ClientBatchItem::new(ClientRequest::Get(GetRequest::new()))
        .with_unique_batch_item_id(b"fluent-batch-get".to_vec());
    let batch = ClientBatch::from_items([create, get]).with_batch_order_option(true);
    let responses = client.execute(batch, limits)?;
    if responses.len() != 2 {
        return Err(format!(
            "batch returned {} response items, expected two",
            responses.len()
        )
        .into());
    }

    let created = responses
        .get(0)
        .ok_or("batch Create response item is missing")?;
    match created.outcome() {
        ClientBatchOutcome::CreateCompleted(response)
            if response.result().status().raw() == 0
                && response.object_type().is_some()
                && response.unique_identifier()
                    == Some(&UniqueIdentifier::TextString("fluent-batch-key".to_owned())) => {}
        outcome => return Err(format!("unexpected batch Create outcome: {outcome:?}").into()),
    }

    let retrieved = responses
        .get(1)
        .ok_or("batch Get response item is missing")?;
    match retrieved.outcome() {
        ClientBatchOutcome::GetCompleted(response)
            if response.result().status().raw() == 0
                && response.unique_identifier()
                    == Some(&UniqueIdentifier::TextString("fluent-batch-key".to_owned()))
                && response.object().is_some() => {}
        outcome => return Err(format!("unexpected batch Get outcome: {outcome:?}").into()),
    }

    println!(
        "rust: fluent ordered batch Create/Get with ID Placeholder verified over TLS 1.3/mTLS"
    );
    Ok(())
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
    assert_eq!(item.outcome().result().status().raw(), 0);
    assert!(item.outcome().response().ping().is_some());
    let raw = item.raw().ok_or("Ping Response Payload is missing")?;
    assert_eq!(raw.children().len(), 0);
    println!("rust: Ping result and raw TTLV verified");
    execute_fluent_lifecycle(&mut client)?;
    execute_fluent_key_pair_signing(&mut client)?;
    execute_fluent_expert_attributes(&mut client)?;
    execute_fluent_batch_placeholder(&mut client, &limits)?;
    Ok(())
}
