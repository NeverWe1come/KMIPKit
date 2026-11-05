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
