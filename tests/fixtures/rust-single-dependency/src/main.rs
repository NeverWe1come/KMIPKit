#![allow(dead_code)]

use kmipkit::protocol::{
    ActivateRequest, AddAttributeRequest, AdjustAttributeRequest, ArchiveRequest, CancelRequest,
    CertifyRequest, CheckRequest, CreateKeyPairRequest, CreateRequest, CreateSplitKeyRequest,
    DecryptRequest, DelegatedLoginRequest, DeleteAttributeRequest, DeriveKeyRequest,
    DestroyRequest, DiscoverVersionsRequest, EncryptRequest, ExportRequest,
    GetAttributeListRequest, GetAttributeListResponse, GetAttributesRequest, GetConstraintsRequest,
    GetRequest, GetResponse,
    GetUsageAllocationRequest, HashRequest, ImportRequest, InteropRequest, JoinSplitKeyRequest,
    LocateRequest, LogRequest, LoginRequest, LogoutRequest, MacRequest, MacVerifyRequest,
    ModifyAttributeRequest, ObtainLeaseRequest, PingRequest, Pkcs11Request, PollRequest,
    ProcessRequest, QueryAsyncRequestsRequest, QueryRequest, ReProvisionRequest,
    RecertifyRequest, RecoverRequest, RegisterRequest, RekeyKeyPairRequest, RekeyRequest,
    RevokeRequest, RngRetrieveRequest, RngSeedRequest, SetAttributeRequest,
    SetConstraintsRequest, SetDefaultsRequest, SignRequest, SignatureVerifyRequest,
    ValidateRequest,
};
use kmipkit::ttlv::codec::{decode_with_limits, CodecLimits};
use kmipkit::ttlv::{Item, RawTag, Value};
use kmipkit::{Client, ClientBatch, ClientBatchResponse, ClientError, PendingOutcome};
use kmipkit::client_config::{client_extension_registry, ClientConfiguration};
use kmipkit::transport_config::{
    CertificateInput, ClientIdentity, Endpoint, PrivateKeyInput, TransportConfig, TrustSource,
};

fn assert_public_request_type<T>() {}

fn all_client_operation_request_types_are_reachable() {
    assert_public_request_type::<ActivateRequest>();
    assert_public_request_type::<AddAttributeRequest>();
    assert_public_request_type::<AdjustAttributeRequest>();
    assert_public_request_type::<ArchiveRequest>();
    assert_public_request_type::<CancelRequest>();
    assert_public_request_type::<CertifyRequest>();
    assert_public_request_type::<CheckRequest>();
    assert_public_request_type::<CreateKeyPairRequest>();
    assert_public_request_type::<CreateRequest>();
    assert_public_request_type::<CreateSplitKeyRequest>();
    assert_public_request_type::<DecryptRequest>();
    assert_public_request_type::<DelegatedLoginRequest>();
    assert_public_request_type::<DeleteAttributeRequest>();
    assert_public_request_type::<DeriveKeyRequest>();
    assert_public_request_type::<DestroyRequest>();
    assert_public_request_type::<DiscoverVersionsRequest>();
    assert_public_request_type::<EncryptRequest>();
    assert_public_request_type::<ExportRequest>();
    assert_public_request_type::<GetAttributeListRequest>();
    assert_public_request_type::<GetAttributesRequest>();
    assert_public_request_type::<GetConstraintsRequest>();
    assert_public_request_type::<GetRequest>();
    assert_public_request_type::<GetUsageAllocationRequest>();
    assert_public_request_type::<HashRequest>();
    assert_public_request_type::<ImportRequest>();
    assert_public_request_type::<InteropRequest>();
    assert_public_request_type::<JoinSplitKeyRequest>();
    assert_public_request_type::<LocateRequest>();
    assert_public_request_type::<LogRequest>();
    assert_public_request_type::<LoginRequest>();
    assert_public_request_type::<LogoutRequest>();
    assert_public_request_type::<MacRequest>();
    assert_public_request_type::<MacVerifyRequest>();
    assert_public_request_type::<ModifyAttributeRequest>();
    assert_public_request_type::<ObtainLeaseRequest>();
    assert_public_request_type::<PingRequest>();
    assert_public_request_type::<Pkcs11Request>();
    assert_public_request_type::<PollRequest>();
    assert_public_request_type::<ProcessRequest>();
    assert_public_request_type::<QueryAsyncRequestsRequest>();
    assert_public_request_type::<QueryRequest>();
    assert_public_request_type::<ReProvisionRequest>();
    assert_public_request_type::<RecertifyRequest>();
    assert_public_request_type::<RecoverRequest>();
    assert_public_request_type::<RegisterRequest>();
    assert_public_request_type::<RekeyKeyPairRequest>();
    assert_public_request_type::<RekeyRequest>();
    assert_public_request_type::<RevokeRequest>();
    assert_public_request_type::<RngRetrieveRequest>();
    assert_public_request_type::<RngSeedRequest>();
    assert_public_request_type::<SetAttributeRequest>();
    assert_public_request_type::<SetConstraintsRequest>();
    assert_public_request_type::<SetDefaultsRequest>();
    assert_public_request_type::<SignRequest>();
    assert_public_request_type::<SignatureVerifyRequest>();
    assert_public_request_type::<ValidateRequest>();
}

fn one_dependency_supports_client_configuration(
    endpoint: impl Into<String>,
    certificate_pem: Vec<u8>,
    private_key_pem: Vec<u8>,
) -> Result<Client, Box<dyn std::error::Error>> {
    let client_configuration = ClientConfiguration::new(client_extension_registry(
        Vec::new(),
        kmipkit::protocol::extension::defaults(),
    )?);
    let transport = TransportConfig::builder(Endpoint::raw_tls(endpoint, 5696))
        .client_identity(ClientIdentity::new(
            CertificateInput::from_pem(certificate_pem),
            PrivateKeyInput::from_pem(private_key_pem),
        ))
        .trust_source(TrustSource::platform())
        .build()?;
    Ok(Client::new(client_configuration, transport)?)
}

fn one_dependency_supports_typed_client_and_generic_ttlv(
    bytes: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let limits = CodecLimits::defaults();
    let _ = decode_with_limits(bytes, &limits);
    let _item = Item::new(
        RawTag::new(0x0042_0077)?.try_checked()?,
        Value::integer(1),
    );
    Ok(())
}

fn typed_execution_returns_public_response_types(
    client: &mut Client,
    batch: ClientBatch,
) -> Result<ClientBatchResponse, ClientError> {
    client.execute(batch, &CodecLimits::defaults())
}

fn operation_response_types_are_reachable(_: GetResponse, _: GetAttributeListResponse) {}

fn asynchronous_result_type_is_reachable(_: PendingOutcome) {}

fn main() {}
