use crate::{
    ActivateRequest, AddAttributeRequest, AdjustAttributeRequest, ArchiveRequest, AttributeSet,
    CancelRequest, CertifyRequest, CheckRequest, CreateKeyPairRequest, CreateRequest,
    CreateSplitKeyRequest, DecryptRequest, DelegatedLoginRequest, DeleteAttributeRequest,
    DerivationMethod, DerivationParameters, DeriveKeyRequest, DestroyRequest,
    DiscoverVersionsRequest, EncryptRequest, ExportRequest, GetAttributeListRequest,
    GetAttributesRequest, GetConstraintsRequest, GetRequest, GetUsageAllocationRequest,
    HashRequest, ImportRequest, InteropRequest, JoinSplitKeyRequest, LocateRequest, LogRequest,
    LoginRequest, LogoutRequest, MacRequest, MacVerifyRequest, ModifyAttributeRequest, ObjectType,
    ObtainLeaseRequest, PingRequest, Pkcs11Request, PollRequest, ProcessRequest,
    QueryAsyncRequestsRequest, QueryRequest, ReProvisionRequest, RecertifyRequest, RecoverRequest,
    RegisterRequest, RekeyKeyPairRequest, RekeyRequest, RevocationReason, RevokeRequest,
    RngRetrieveRequest, RngSeedRequest, SecretBytes, SetAttributeRequest, SetConstraintsRequest,
    SetDefaultsRequest, SignRequest, SignatureVerifyRequest, SplitKeyMethod, ValidateKmipRequest,
    ValidateRequest,
};
use kmipkit_ttlv::{Item, RawTag, Value};

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

#[test]
fn create_validation_borrows_and_reports_duplicate_single_instance_attributes() {
    let comment = RawTag::new(0x0042_00fd)
        .expect("Comment is a valid raw tag")
        .try_checked()
        .expect("Comment is an allocated KMIP tag");
    let attributes = AttributeSet::try_new([
        Item::new(comment, Value::text_string("first".to_owned()))
            .expect("first Comment is valid TTLV"),
        Item::new(comment, Value::text_string("second".to_owned()))
            .expect("second Comment is valid TTLV"),
    ])
    .expect("generic attribute sets preserve operation-level cardinality violations");
    let request = CreateRequest::new(ObjectType::from_raw(7), attributes);

    let report = request.validate();

    assert!(!report.is_valid());
    assert_eq!(report.issues().len(), 1);
    assert_eq!(
        report.issues()[0].path().to_string(),
        "Request Payload.Attributes"
    );
    assert_eq!(
        report.issues()[0].source().to_string(),
        "KMIP v2.1 §4, Table 46"
    );
    assert!(!format!("{report:?}").contains("first"));
    assert!(!format!("{report:?}").contains("second"));
}

#[test]
fn certify_validation_rejects_request_value_without_type() {
    let request = CertifyRequest::new()
        .with_certificate_request_value(SecretBytes::new(b"opaque-request".to_vec()));

    assert!(!request.validate().is_valid());
}

#[test]
fn create_split_key_validation_rejects_missing_prime_field_size() {
    let request = CreateSplitKeyRequest::new(
        ObjectType::from_raw(7),
        3,
        2,
        SplitKeyMethod::POLYNOMIAL_SHARING_PRIME_FIELD,
        AttributeSet::new(),
    );

    assert!(!request.validate().is_valid());
}

#[test]
fn derive_key_validation_rejects_missing_required_input_identifier() {
    let request = DeriveKeyRequest::new(
        ObjectType::from_raw(7),
        Vec::new(),
        DerivationMethod::from_raw(0),
        DerivationParameters::new(),
        AttributeSet::new(),
    );

    assert!(!request.validate().is_valid());
}

#[test]
fn revoke_validation_rejects_compromise_date_for_unrelated_reason() {
    let request = RevokeRequest::new(RevocationReason::UNSPECIFIED)
        .with_compromise_occurrence_date(1_700_000_000);

    assert!(!request.validate().is_valid());
}

#[test]
fn validate_request_validation_rejects_an_empty_chain() {
    assert!(!ValidateRequest::new().validate().is_valid());
}

#[test]
fn create_key_pair_validation_rejects_conflicting_table_191_attributes() {
    const CRYPTOGRAPHIC_LENGTH: u32 = 0x0042_002a;
    let attributes = |length| {
        AttributeSet::try_new([Item::new(
            RawTag::new(CRYPTOGRAPHIC_LENGTH)
                .expect("Cryptographic Length tag is valid")
                .try_checked()
                .expect("Cryptographic Length tag is allocated"),
            Value::integer(length),
        )
        .expect("Cryptographic Length is valid TTLV")])
        .expect("one Cryptographic Length attribute is valid")
    };
    let request = CreateKeyPairRequest::new()
        .with_private_key_attributes(attributes(256))
        .with_public_key_attributes(attributes(2048));

    assert!(!request.validate().is_valid());
}

#[test]
fn recertify_validation_rejects_missing_request_and_existing_identifier() {
    assert!(!RecertifyRequest::new().validate().is_valid());
}

#[test]
fn rekey_validation_rejects_repeated_single_instance_attribute() {
    let comment = RawTag::new(0x0042_00fd)
        .expect("Comment is a valid raw tag")
        .try_checked()
        .expect("Comment is an allocated KMIP tag");
    let mut attributes = kmipkit_ttlv::Structure::new();
    for value in ["one", "two"] {
        attributes
            .try_push(
                Item::new(comment, Value::text_string(value.to_owned()))
                    .expect("Comment is valid TTLV"),
            )
            .expect("both values fit the generic attribute Structure");
    }

    assert!(
        !RekeyRequest::new()
            .with_attributes(attributes)
            .validate()
            .is_valid()
    );
}

#[test]
fn rekey_key_pair_validation_rejects_repeated_single_instance_attribute() {
    let comment = RawTag::new(0x0042_00fd)
        .expect("Comment is a valid raw tag")
        .try_checked()
        .expect("Comment is an allocated KMIP tag");
    let attributes = AttributeSet::try_new([
        Item::new(comment, Value::text_string("one".to_owned()))
            .expect("first Comment is valid TTLV"),
        Item::new(comment, Value::text_string("two".to_owned()))
            .expect("second Comment is valid TTLV"),
    ])
    .expect("generic AttributeSet preserves operation-level repetitions");

    assert!(
        !RekeyKeyPairRequest::new()
            .with_private_key_attributes(attributes)
            .validate()
            .is_valid()
    );
}
