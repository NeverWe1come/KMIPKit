//! Borrowed validation contract and operation identity mapping for typed requests.

use crate::{
    ActivateRequest, AddAttributeRequest, AdjustAttributeRequest, ArchiveRequest, CancelRequest,
    CheckRequest, DelegatedLoginRequest, DeleteAttributeRequest, DestroyRequest,
    DiscoverVersionsRequest, ExportRequest, GetAttributeListRequest, GetAttributesRequest,
    GetConstraintsRequest, GetRequest, GetUsageAllocationRequest, ImportRequest, InteropRequest,
    JoinSplitKeyRequest, LocateRequest, LogRequest, LoginRequest, LogoutRequest,
    ModifyAttributeRequest, ObtainLeaseRequest, PingRequest, Pkcs11Request, PollRequest,
    ProcessRequest, QueryAsyncRequestsRequest, QueryRequest, ReProvisionRequest, RecoverRequest,
    RngRetrieveRequest, RngSeedRequest, SetAttributeRequest, SetConstraintsRequest,
    SetDefaultsRequest, ValidationReport,
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
    CheckRequest => Check,
    DelegatedLoginRequest => DelegatedLogin,
    DeleteAttributeRequest => DeleteAttribute,
    DestroyRequest => Destroy,
    DiscoverVersionsRequest => DiscoverVersions,
    ExportRequest => Export,
    GetRequest => Get,
    GetAttributeListRequest => GetAttributeList,
    GetAttributesRequest => GetAttributes,
    GetConstraintsRequest => GetConstraints,
    GetUsageAllocationRequest => GetUsageAllocation,
    ImportRequest => Import,
    InteropRequest => Interop,
    JoinSplitKeyRequest => JoinSplitKey,
    LocateRequest => Locate,
    LogRequest => Log,
    LoginRequest => Login,
    LogoutRequest => Logout,
    ModifyAttributeRequest => ModifyAttribute,
    ObtainLeaseRequest => ObtainLease,
    PingRequest => Ping,
    Pkcs11Request => Pkcs11,
    PollRequest => Poll,
    ProcessRequest => Process,
    QueryRequest => Query,
    QueryAsyncRequestsRequest => QueryAsyncRequests,
    ReProvisionRequest => ReProvision,
    RecoverRequest => Recover,
    RngRetrieveRequest => RngRetrieve,
    RngSeedRequest => RngSeed,
    SetAttributeRequest => SetAttribute,
    SetConstraintsRequest => SetConstraints,
    SetDefaultsRequest => SetDefaults,
}

macro_rules! impl_validation_with_existing_check {
    ($request:ty => $schema:ident, $check:ident, $field:literal, $reference:literal) => {
        impl ValidateKmipRequest for $request {
            const SCHEMA: RequestPayloadSchema = RequestPayloadSchema::$schema;

            fn validate(&self) -> ValidationReport {
                if self.$check().is_err() {
                    crate::validation::request_issue_report(
                        "KMIPKIT-0051-P005",
                        $field,
                        crate::ValidationSourceRef::Oasis {
                            document: "KMIP v2.1",
                            reference: $reference,
                        },
                    )
                } else {
                    ValidationReport::default()
                }
            }
        }
    };
}

impl_validation_with_existing_check!(
    crate::EncryptRequest => Encrypt,
    validate_multipart_shape,
    "Data",
    "§6.1.17, Table 214"
);
impl_validation_with_existing_check!(
    crate::DecryptRequest => Decrypt,
    validate_multipart_shape,
    "Data",
    "§6.1.11, Table 196"
);
impl_validation_with_existing_check!(
    crate::HashRequest => Hash,
    validate_multipart_shape,
    "Data",
    "§6.1.24, Table 235"
);
impl_validation_with_existing_check!(
    crate::MacRequest => Mac,
    validate_multipart_shape,
    "Data",
    "§6.1.32, Table 259"
);
impl_validation_with_existing_check!(
    crate::MacVerifyRequest => MacVerify,
    validate_multipart_shape,
    "MAC Data",
    "§6.1.33, Table 262"
);
impl_validation_with_existing_check!(
    crate::SignRequest => Sign,
    validate_multipart_shape,
    "Data",
    "§6.1.55, Table 334"
);
impl_validation_with_existing_check!(
    crate::SignatureVerifyRequest => SignatureVerify,
    validate_multipart_shape,
    "Signature Data",
    "§6.1.56, Table 337"
);
impl_validation_with_existing_check!(
    crate::RegisterRequest => Register,
    validate_local_shape,
    "Object",
    "§6.1.43, Tables 291 and 293"
);
