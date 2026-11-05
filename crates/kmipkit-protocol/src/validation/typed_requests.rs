//! Borrowed validation contract and operation identity mapping for typed requests.

use crate::{
    ActivateRequest, AddAttributeRequest, AdjustAttributeRequest, ArchiveRequest, CancelRequest,
    CertifyRequest, CheckRequest, CreateKeyPairRequest, CreateSplitKeyRequest, DecryptRequest,
    DelegatedLoginRequest, DeleteAttributeRequest, DeriveKeyRequest, DestroyRequest,
    DiscoverVersionsRequest, EncryptRequest, ExportRequest, GetAttributeListRequest,
    GetAttributesRequest, GetConstraintsRequest, GetRequest, GetUsageAllocationRequest,
    HashRequest, ImportRequest, InteropRequest, JoinSplitKeyRequest, LocateRequest, LogRequest,
    LoginRequest, LogoutRequest, MacRequest, MacVerifyRequest, ModifyAttributeRequest,
    ObtainLeaseRequest, PingRequest, Pkcs11Request, PollRequest, ProcessRequest,
    QueryAsyncRequestsRequest, QueryRequest, ReProvisionRequest, RecertifyRequest, RecoverRequest,
    RegisterRequest, RekeyKeyPairRequest, RekeyRequest, RevokeRequest, RngRetrieveRequest,
    RngSeedRequest, SetAttributeRequest, SetConstraintsRequest, SetDefaultsRequest, SignRequest,
    SignatureVerifyRequest, ValidateRequest, ValidationReport,
};

use super::schema::RequestPayloadSchema;

/// Borrowed validation contract for a typed client-initiated KMIP request.
///
/// Implementations identify the exact Request Payload schema and inspect the
/// request without consuming or cloning it. Operation checks are being added
/// with their source-backed tests; an empty report does not establish complete
/// KMIP conformance. No report establishes server capability, authorization,
/// state, or policy acceptance.
pub trait ValidateKmipRequest {
    /// The OASIS Request Payload schema for this operation.
    const SCHEMA: RequestPayloadSchema;

    /// Validates locally decidable constraints without consuming the request.
    #[must_use]
    fn validate(&self) -> ValidationReport;
}

macro_rules! impl_typed_request_validation {
    ($($request:ty => $schema:ident),+ $(,)?) => {
        $(
            impl ValidateKmipRequest for $request {
                const SCHEMA: RequestPayloadSchema = RequestPayloadSchema::$schema;

                fn validate(&self) -> ValidationReport {
                    // Operation-specific validation is added alongside its
                    // request model and source-backed tests. Keep the common
                    // contract usable while that exhaustive audit proceeds.
                    let _ = self;
                    ValidationReport::default()
                }
            }
        )+
    };
}

impl_typed_request_validation! {
    ActivateRequest => Activate,
    AddAttributeRequest => AddAttribute,
    AdjustAttributeRequest => AdjustAttribute,
    ArchiveRequest => Archive,
    CancelRequest => Cancel,
    CertifyRequest => Certify,
    CheckRequest => Check,
    CreateKeyPairRequest => CreateKeyPair,
    CreateSplitKeyRequest => CreateSplitKey,
    DecryptRequest => Decrypt,
    DelegatedLoginRequest => DelegatedLogin,
    DeleteAttributeRequest => DeleteAttribute,
    DeriveKeyRequest => DeriveKey,
    DestroyRequest => Destroy,
    DiscoverVersionsRequest => DiscoverVersions,
    EncryptRequest => Encrypt,
    ExportRequest => Export,
    GetRequest => Get,
    GetAttributeListRequest => GetAttributeList,
    GetAttributesRequest => GetAttributes,
    GetConstraintsRequest => GetConstraints,
    GetUsageAllocationRequest => GetUsageAllocation,
    HashRequest => Hash,
    ImportRequest => Import,
    InteropRequest => Interop,
    JoinSplitKeyRequest => JoinSplitKey,
    LocateRequest => Locate,
    LogRequest => Log,
    LoginRequest => Login,
    LogoutRequest => Logout,
    MacRequest => Mac,
    MacVerifyRequest => MacVerify,
    ModifyAttributeRequest => ModifyAttribute,
    ObtainLeaseRequest => ObtainLease,
    PingRequest => Ping,
    Pkcs11Request => Pkcs11,
    PollRequest => Poll,
    ProcessRequest => Process,
    QueryRequest => Query,
    QueryAsyncRequestsRequest => QueryAsyncRequests,
    ReProvisionRequest => ReProvision,
    RecertifyRequest => Recertify,
    RecoverRequest => Recover,
    RegisterRequest => Register,
    RekeyRequest => Rekey,
    RekeyKeyPairRequest => RekeyKeyPair,
    RevokeRequest => Revoke,
    RngRetrieveRequest => RngRetrieve,
    RngSeedRequest => RngSeed,
    SetAttributeRequest => SetAttribute,
    SetConstraintsRequest => SetConstraints,
    SetDefaultsRequest => SetDefaults,
    SignRequest => Sign,
    SignatureVerifyRequest => SignatureVerify,
    ValidateRequest => Validate,
}
