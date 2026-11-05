use crate::{
    ActivateRequest, AddAttributeRequest, AdjustAttributeRequest, ArchiveRequest, CancelRequest,
    CertifyRequest, CheckRequest, CreateKeyPairRequest, CreateRequest, CreateSplitKeyRequest,
    DecryptRequest, DelegatedLoginRequest, DeleteAttributeRequest, DeriveKeyRequest,
    DestroyRequest, DiscoverVersionsRequest, EncryptRequest, ExportRequest,
    GetAttributeListRequest, GetAttributesRequest, GetConstraintsRequest, GetRequest,
    GetUsageAllocationRequest, HashRequest, ImportRequest, InteropRequest, JoinSplitKeyRequest,
    LocateRequest, LogRequest, LoginRequest, LogoutRequest, MacRequest, MacVerifyRequest,
    ModifyAttributeRequest, ObtainLeaseRequest, PingRequest, Pkcs11Request, PollRequest,
    ProcessRequest, QueryAsyncRequestsRequest, QueryRequest, ReProvisionRequest, RecertifyRequest,
    RecoverRequest, RegisterRequest, RekeyKeyPairRequest, RekeyRequest, RevokeRequest,
    RngRetrieveRequest, RngSeedRequest, SetAttributeRequest, SetConstraintsRequest,
    SetDefaultsRequest, SignRequest, SignatureVerifyRequest, ValidateKmipRequest, ValidateRequest,
};

fn assert_validate_contract<T: ValidateKmipRequest>() {}

#[test]
fn every_supported_client_request_implements_borrowed_validation() {
    assert_validate_contract::<ActivateRequest>();
    assert_validate_contract::<AddAttributeRequest>();
    assert_validate_contract::<AdjustAttributeRequest>();
    assert_validate_contract::<ArchiveRequest>();
    assert_validate_contract::<CancelRequest>();
    assert_validate_contract::<CertifyRequest>();
    assert_validate_contract::<CheckRequest>();
    assert_validate_contract::<CreateRequest>();
    assert_validate_contract::<CreateKeyPairRequest>();
    assert_validate_contract::<CreateSplitKeyRequest>();
    assert_validate_contract::<DecryptRequest>();
    assert_validate_contract::<DelegatedLoginRequest>();
    assert_validate_contract::<DeleteAttributeRequest>();
    assert_validate_contract::<DeriveKeyRequest>();
    assert_validate_contract::<DestroyRequest>();
    assert_validate_contract::<DiscoverVersionsRequest>();
    assert_validate_contract::<EncryptRequest>();
    assert_validate_contract::<ExportRequest>();
    assert_validate_contract::<GetRequest>();
    assert_validate_contract::<GetAttributeListRequest>();
    assert_validate_contract::<GetAttributesRequest>();
    assert_validate_contract::<GetConstraintsRequest>();
    assert_validate_contract::<GetUsageAllocationRequest>();
    assert_validate_contract::<HashRequest>();
    assert_validate_contract::<ImportRequest>();
    assert_validate_contract::<InteropRequest>();
    assert_validate_contract::<JoinSplitKeyRequest>();
    assert_validate_contract::<LocateRequest>();
    assert_validate_contract::<LogRequest>();
    assert_validate_contract::<LoginRequest>();
    assert_validate_contract::<LogoutRequest>();
    assert_validate_contract::<MacRequest>();
    assert_validate_contract::<MacVerifyRequest>();
    assert_validate_contract::<ModifyAttributeRequest>();
    assert_validate_contract::<ObtainLeaseRequest>();
    assert_validate_contract::<PingRequest>();
    assert_validate_contract::<Pkcs11Request>();
    assert_validate_contract::<PollRequest>();
    assert_validate_contract::<ProcessRequest>();
    assert_validate_contract::<QueryRequest>();
    assert_validate_contract::<QueryAsyncRequestsRequest>();
    assert_validate_contract::<ReProvisionRequest>();
    assert_validate_contract::<RecertifyRequest>();
    assert_validate_contract::<RecoverRequest>();
    assert_validate_contract::<RegisterRequest>();
    assert_validate_contract::<RekeyRequest>();
    assert_validate_contract::<RekeyKeyPairRequest>();
    assert_validate_contract::<RevokeRequest>();
    assert_validate_contract::<RngRetrieveRequest>();
    assert_validate_contract::<RngSeedRequest>();
    assert_validate_contract::<SetAttributeRequest>();
    assert_validate_contract::<SetConstraintsRequest>();
    assert_validate_contract::<SetDefaultsRequest>();
    assert_validate_contract::<SignRequest>();
    assert_validate_contract::<SignatureVerifyRequest>();
    assert_validate_contract::<ValidateRequest>();
}

fn assert_schema<T: ValidateKmipRequest>(expected: crate::RequestPayloadSchema) {
    assert_eq!(T::SCHEMA, expected);
}

#[test]
fn every_typed_request_maps_to_its_exact_payload_schema() {
    use crate::RequestPayloadSchema as Schema;

    assert_schema::<ActivateRequest>(Schema::Activate);
    assert_schema::<AddAttributeRequest>(Schema::AddAttribute);
    assert_schema::<AdjustAttributeRequest>(Schema::AdjustAttribute);
    assert_schema::<ArchiveRequest>(Schema::Archive);
    assert_schema::<CancelRequest>(Schema::Cancel);
    assert_schema::<CertifyRequest>(Schema::Certify);
    assert_schema::<CheckRequest>(Schema::Check);
    assert_schema::<CreateRequest>(Schema::Create);
    assert_schema::<CreateKeyPairRequest>(Schema::CreateKeyPair);
    assert_schema::<CreateSplitKeyRequest>(Schema::CreateSplitKey);
    assert_schema::<DecryptRequest>(Schema::Decrypt);
    assert_schema::<DelegatedLoginRequest>(Schema::DelegatedLogin);
    assert_schema::<DeleteAttributeRequest>(Schema::DeleteAttribute);
    assert_schema::<DeriveKeyRequest>(Schema::DeriveKey);
    assert_schema::<DestroyRequest>(Schema::Destroy);
    assert_schema::<DiscoverVersionsRequest>(Schema::DiscoverVersions);
    assert_schema::<EncryptRequest>(Schema::Encrypt);
    assert_schema::<ExportRequest>(Schema::Export);
    assert_schema::<GetRequest>(Schema::Get);
    assert_schema::<GetAttributeListRequest>(Schema::GetAttributeList);
    assert_schema::<GetAttributesRequest>(Schema::GetAttributes);
    assert_schema::<GetConstraintsRequest>(Schema::GetConstraints);
    assert_schema::<GetUsageAllocationRequest>(Schema::GetUsageAllocation);
    assert_schema::<HashRequest>(Schema::Hash);
    assert_schema::<ImportRequest>(Schema::Import);
    assert_schema::<InteropRequest>(Schema::Interop);
    assert_schema::<JoinSplitKeyRequest>(Schema::JoinSplitKey);
    assert_schema::<LocateRequest>(Schema::Locate);
    assert_schema::<LogRequest>(Schema::Log);
    assert_schema::<LoginRequest>(Schema::Login);
    assert_schema::<LogoutRequest>(Schema::Logout);
    assert_schema::<MacRequest>(Schema::Mac);
    assert_schema::<MacVerifyRequest>(Schema::MacVerify);
    assert_schema::<ModifyAttributeRequest>(Schema::ModifyAttribute);
    assert_schema::<ObtainLeaseRequest>(Schema::ObtainLease);
    assert_schema::<PingRequest>(Schema::Ping);
    assert_schema::<Pkcs11Request>(Schema::Pkcs11);
    assert_schema::<PollRequest>(Schema::Poll);
    assert_schema::<ProcessRequest>(Schema::Process);
    assert_schema::<QueryRequest>(Schema::Query);
    assert_schema::<QueryAsyncRequestsRequest>(Schema::QueryAsyncRequests);
    assert_schema::<ReProvisionRequest>(Schema::ReProvision);
    assert_schema::<RecertifyRequest>(Schema::Recertify);
    assert_schema::<RecoverRequest>(Schema::Recover);
    assert_schema::<RegisterRequest>(Schema::Register);
    assert_schema::<RekeyRequest>(Schema::Rekey);
    assert_schema::<RekeyKeyPairRequest>(Schema::RekeyKeyPair);
    assert_schema::<RevokeRequest>(Schema::Revoke);
    assert_schema::<RngRetrieveRequest>(Schema::RngRetrieve);
    assert_schema::<RngSeedRequest>(Schema::RngSeed);
    assert_schema::<SetAttributeRequest>(Schema::SetAttribute);
    assert_schema::<SetConstraintsRequest>(Schema::SetConstraints);
    assert_schema::<SetDefaultsRequest>(Schema::SetDefaults);
    assert_schema::<SignRequest>(Schema::Sign);
    assert_schema::<SignatureVerifyRequest>(Schema::SignatureVerify);
    assert_schema::<ValidateRequest>(Schema::Validate);
}
