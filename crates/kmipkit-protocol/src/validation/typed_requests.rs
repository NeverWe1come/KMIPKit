//! Borrowed validation contract and operation identity mapping for typed requests.

use crate::{
    ActivateRequest, AddAttributeRequest, AdjustAttributeRequest, ArchiveRequest, CancelRequest,
    CheckRequest, DelegatedLoginRequest, DeleteAttributeRequest, DestroyRequest,
    DiscoverVersionsRequest, GetAttributeListRequest, GetAttributesRequest, GetConstraintsRequest,
    GetRequest, GetUsageAllocationRequest, ImportRequest, JoinSplitKeyRequest, LocateRequest,
    LoginRequest, LogoutRequest, ModifyAttributeRequest, ObtainLeaseRequest, PingRequest,
    Pkcs11Request, PollRequest, ProcessRequest, QueryAsyncRequestsRequest, QueryRequest,
    RecoverRequest, RngRetrieveRequest, RngSeedRequest, SetAttributeRequest, SetConstraintsRequest,
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

fn validate_nested_request_field(
    structure: &kmipkit_ttlv::Structure,
    schema: crate::NestedStructureSchema,
    field: &'static str,
) -> ValidationReport {
    crate::validation::validate_nested_structure_with_prefix(
        &structure.view(),
        schema,
        &crate::ValidationPath::root()
            .field("Request Payload")
            .field(field),
    )
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
    DeleteAttributeRequest => DeleteAttribute,
    DestroyRequest => Destroy,
    DiscoverVersionsRequest => DiscoverVersions,
    GetAttributeListRequest => GetAttributeList,
    GetAttributesRequest => GetAttributes,
    GetConstraintsRequest => GetConstraints,
    GetUsageAllocationRequest => GetUsageAllocation,
    LocateRequest => Locate,
    LogoutRequest => Logout,
    ModifyAttributeRequest => ModifyAttribute,
    ObtainLeaseRequest => ObtainLease,
    PingRequest => Ping,
    PollRequest => Poll,
    ProcessRequest => Process,
    QueryAsyncRequestsRequest => QueryAsyncRequests,
    RecoverRequest => Recover,
    RngRetrieveRequest => RngRetrieve,
    RngSeedRequest => RngSeed,
    SetAttributeRequest => SetAttribute,
}

impl ValidateKmipRequest for GetRequest {
    const SCHEMA: RequestPayloadSchema = RequestPayloadSchema::Get;

    fn validate(&self) -> ValidationReport {
        let Some(specification) = self.key_wrapping_specification() else {
            return ValidationReport::default();
        };
        validate_nested_request_field(
            specification,
            crate::NestedStructureSchema::KeyWrappingSpecification,
            "Key Wrapping Specification",
        )
    }
}

impl ValidateKmipRequest for LoginRequest {
    const SCHEMA: RequestPayloadSchema = RequestPayloadSchema::Login;

    fn validate(&self) -> ValidationReport {
        self.usage_limits()
            .map_or_else(ValidationReport::default, |usage_limits| {
                validate_nested_request_field(
                    usage_limits,
                    crate::NestedStructureSchema::UsageLimits,
                    "Usage Limits",
                )
            })
    }
}

impl ValidateKmipRequest for DelegatedLoginRequest {
    const SCHEMA: RequestPayloadSchema = RequestPayloadSchema::DelegatedLogin;

    fn validate(&self) -> ValidationReport {
        let mut report = validate_nested_request_field(
            self.rights(),
            crate::NestedStructureSchema::Rights,
            "Rights",
        );
        if let Some(usage_limits) = self.usage_limits() {
            report.append(validate_nested_request_field(
                usage_limits,
                crate::NestedStructureSchema::UsageLimits,
                "Usage Limits",
            ));
        }
        report
    }
}

impl ValidateKmipRequest for ImportRequest {
    const SCHEMA: RequestPayloadSchema = RequestPayloadSchema::Import;

    fn validate(&self) -> ValidationReport {
        let object = self.object();
        object.with_value(|value| match value {
            kmipkit_ttlv::ValueView::Structure(object_structure) => {
                super::nested::validate_managed_object(
                    &object_structure,
                    object.tag().raw(),
                    &crate::ValidationPath::root()
                        .field("Request Payload")
                        .field("Any Object"),
                )
            }
            _ => ValidationReport::default(),
        })
    }
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

macro_rules! impl_crypto_request_validation {
    ($request:ty => $schema:ident, $check:ident, $field:literal, $reference:literal) => {
        impl ValidateKmipRequest for $request {
            const SCHEMA: RequestPayloadSchema = RequestPayloadSchema::$schema;

            fn validate(&self) -> ValidationReport {
                let mut report = ValidationReport::default();
                if self.$check().is_err() {
                    report.append(crate::validation::request_issue_report(
                        "KMIPKIT-0051-P005",
                        $field,
                        crate::ValidationSourceRef::Oasis {
                            document: "KMIP v2.1",
                            reference: $reference,
                        },
                    ));
                }
                if let Some(parameters) = self.cryptographic_parameters() {
                    report.append(validate_nested_request_field(
                        parameters,
                        crate::NestedStructureSchema::CryptographicParameters,
                        "Cryptographic Parameters",
                    ));
                }
                report
            }
        }
    };
}

impl_crypto_request_validation!(
    crate::EncryptRequest => Encrypt,
    validate_multipart_shape,
    "Data",
    "§6.1.17, Table 214"
);
impl_crypto_request_validation!(
    crate::DecryptRequest => Decrypt,
    validate_multipart_shape,
    "Data",
    "§6.1.11, Table 196"
);
impl_crypto_request_validation!(
    crate::MacRequest => Mac,
    validate_multipart_shape,
    "Data",
    "§6.1.32, Table 259"
);
impl_crypto_request_validation!(
    crate::MacVerifyRequest => MacVerify,
    validate_multipart_shape,
    "MAC Data",
    "§6.1.33, Table 262"
);
impl_crypto_request_validation!(
    crate::SignRequest => Sign,
    validate_multipart_shape,
    "Data",
    "§6.1.55, Table 334"
);
impl_crypto_request_validation!(
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

impl ValidateKmipRequest for QueryRequest {
    const SCHEMA: RequestPayloadSchema = RequestPayloadSchema::Query;

    fn validate(&self) -> ValidationReport {
        if !self.has_query_functions() {
            crate::validation::request_issue_report(
                "KMIPKIT-0051-P005",
                "Query Function",
                crate::ValidationSourceRef::Oasis {
                    document: "KMIP v2.1",
                    reference: "§6.1.40, Table 282",
                },
            )
        } else if !self.has_valid_query_function_values() {
            crate::validation::request_issue_report(
                "KMIPKIT-0051-P005",
                "Query Function",
                crate::ValidationSourceRef::Oasis {
                    document: "KMIP v2.1",
                    reference: "§11.44, Table 476",
                },
            )
        } else {
            ValidationReport::default()
        }
    }
}

impl ValidateKmipRequest for JoinSplitKeyRequest {
    const SCHEMA: RequestPayloadSchema = RequestPayloadSchema::JoinSplitKey;

    fn validate(&self) -> ValidationReport {
        match self.validate_request_shape() {
            Ok(()) => ValidationReport::default(),
            Err(crate::join_split_key::JoinSplitKeyValidationFailure::Attributes) => {
                crate::validation::request_issue_report(
                    "KMIPKIT-0051-P005",
                    "Attributes",
                    crate::ValidationSourceRef::Oasis {
                        document: "KMIP v2.1",
                        reference: "§4, Table 46; §6.1.27, Table 244",
                    },
                )
            }
            Err(crate::join_split_key::JoinSplitKeyValidationFailure::References) => {
                crate::validation::request_issue_report(
                    "KMIPKIT-0051-P005",
                    "Unique Identifier",
                    crate::ValidationSourceRef::Oasis {
                        document: "KMIP v2.1",
                        reference: "§6.1.27, Table 244; §2.8, Table 9",
                    },
                )
            }
        }
    }
}

impl ValidateKmipRequest for Pkcs11Request {
    const SCHEMA: RequestPayloadSchema = RequestPayloadSchema::Pkcs11;

    fn validate(&self) -> ValidationReport {
        let (field, reference) = match self.validate_request_shape() {
            Ok(()) => return ValidationReport::default(),
            Err(crate::pkcs11::Pkcs11RequestValidationFailure::Interface) => {
                ("PKCS#11 Interface", "§6.1.37, Table 273; §7.28, Table 380")
            }
            Err(crate::pkcs11::Pkcs11RequestValidationFailure::Function) => {
                ("PKCS#11 Function", "§6.1.37, Table 273; §7.26, Table 378")
            }
            Err(crate::pkcs11::Pkcs11RequestValidationFailure::InputParameters) => (
                "PKCS#11 Input Parameters",
                "§6.1.37, Table 273; §7.27, Table 379",
            ),
            Err(crate::pkcs11::Pkcs11RequestValidationFailure::ExtensionCollision) => {
                ("Request Payload", "§6.1.37, Table 273")
            }
        };
        crate::validation::request_issue_report(
            "KMIPKIT-0051-P005",
            field,
            crate::ValidationSourceRef::Oasis {
                document: "KMIP v2.1",
                reference,
            },
        )
    }
}

impl ValidateKmipRequest for SetConstraintsRequest {
    const SCHEMA: RequestPayloadSchema = RequestPayloadSchema::SetConstraints;

    fn validate(&self) -> ValidationReport {
        crate::validation::validate_nested_structure_with_prefix(
            &self.constraints(),
            crate::NestedStructureSchema::Constraints,
            &crate::ValidationPath::root()
                .field("Request Payload")
                .field("Constraints"),
        )
    }
}

impl ValidateKmipRequest for SetDefaultsRequest {
    const SCHEMA: RequestPayloadSchema = RequestPayloadSchema::SetDefaults;

    fn validate(&self) -> ValidationReport {
        let mut report = ValidationReport::default();
        if let Some(defaults) = self.defaults_information() {
            report.append(crate::validation::validate_nested_structure_with_prefix(
                &defaults,
                crate::NestedStructureSchema::DefaultsInformation,
                &crate::ValidationPath::root()
                    .field("Request Payload")
                    .field("Defaults Information"),
            ));
        }
        report
    }
}
