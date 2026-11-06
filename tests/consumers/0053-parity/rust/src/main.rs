use kmipkit::{
    Client, ClientBatch, ClientBatchItem, ClientBatchResponse, ClientError,
    client_config::ClientConfiguration,
    protocol::{QueryFunction, QueryRequest},
    transport_config::TransportConfig,
    ttlv::codec::CodecLimits,
};

fn execute_ping(
    client: &mut Client,
    limits: &CodecLimits,
) -> Result<ClientBatchResponse, ClientError> {
    client.execute(ClientBatch::new(ClientBatchItem::ping()), limits)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ping = ClientBatchItem::ping();
    assert_eq!(ping.operation(), 0x3b);

    let query = QueryRequest::new([QueryFunction::OPERATIONS]);
    let query_payload = query.to_ttlv_payload()?;
    assert_eq!(query_payload.view().children().len(), 1);

    // Type-check the complete single-dependency client configuration boundary.
    let _client_builder: fn(ClientConfiguration, TransportConfig) -> Result<Client, ClientError> =
        Client::new;
    let _exchange: fn(&mut Client, &CodecLimits) -> Result<ClientBatchResponse, ClientError> =
        execute_ping;
    Ok(())
}
