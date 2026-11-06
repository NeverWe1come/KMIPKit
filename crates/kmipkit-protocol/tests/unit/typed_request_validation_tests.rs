use crate::{
    ActivateRequest, AddAttributeRequest, AdjustAttributeRequest, ArchiveRequest, AttributeSet,
    CancelRequest, CertifyRequest, CheckRequest, CreateKeyPairRequest, CreateRequest,
    CreateSplitKeyRequest, DecryptRequest, DelegatedLoginRequest, DeleteAttributeRequest,
    DerivationMethod, DerivationParameters, DeriveKeyRequest, DestroyRequest,
    DiscoverVersionsRequest, EncryptRequest, ExportRequest, GetAttributeListRequest,
    GetAttributesRequest, GetConstraintsRequest, GetRequest, GetUsageAllocationRequest,
    HashRequest, ImportRequest, InteropRequest, JoinSplitKeyRequest, LocateRequest, LogRequest,
    LoginRequest, LogoutRequest, MacRequest, MacVerifyRequest, ModifyAttributeRequest, ObjectType,
    ObtainLeaseRequest, OperationData, PingRequest, Pkcs11Request, PollRequest, ProcessRequest,
    QueryAsyncRequestsRequest, QueryFunction, QueryRequest, ReProvisionRequest, RecertifyRequest,
    RecoverRequest, RegisterRequest, RekeyKeyPairRequest, RekeyRequest, RevocationReason,
    RevokeRequest, RngRetrieveRequest, RngSeedRequest, SecretBytes, SetAttributeRequest,
    SetConstraintsRequest, SetDefaultsRequest, SignRequest, SignatureVerifyRequest, SplitKeyMethod,
    SplitKeyReference, UniqueIdentifier, ValidateKmipRequest, ValidateRequest,
};
use kmipkit_ttlv::{Item, RawTag, Structure, Value};

fn assert_validate_contract<T: ValidateKmipRequest>() {}

fn schema_item(tag: u32, value: Value) -> Item {
    Item::new(
        RawTag::new(tag)
            .expect("schema fixture tag is valid")
            .try_checked()
            .expect("schema fixture tag is allocated"),
        value,
    )
    .expect("schema fixture item is valid TTLV")
}

fn schema_structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut structure = Structure::new();
    for value in items {
        structure
            .try_push(value)
            .expect("schema fixture fits the generic Structure limits");
    }
    structure
}

fn gcm_parameters_without_required_lengths() -> Structure {
    schema_structure([schema_item(0x0042_0011, Value::enumeration(9))])
}

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

#[test]
fn attribute_mutation_validation_rejects_usage_limits_count_for_add_and_modify() {
    let usage_limits = schema_item(
        0x0042_0095,
        Value::structure(schema_structure([schema_item(
            0x0042_0096,
            Value::long_integer(7),
        )])),
    );
    let add = AddAttributeRequest::new(None, crate::NewAttribute::new(usage_limits));
    let modify = ModifyAttributeRequest::new(
        None,
        None,
        crate::NewAttribute::new(schema_item(
            0x0042_0095,
            Value::structure(schema_structure([schema_item(
                0x0042_0096,
                Value::long_integer(7),
            )])),
        )),
    );

    for report in [add.validate(), modify.validate()] {
        assert!(report.issues().iter().any(|issue| {
            issue.code().as_str() == "KMIPKIT-0051-R281"
                && issue.path().to_string()
                    == "Request Payload.New Attribute.Usage Limits.Usage Limits Count"
                && issue.source().to_string() == "KMIP v2.1 §4.59; §7.40, Table 392"
        }));
    }

    let total_only = crate::NewAttribute::new(schema_item(
        0x0042_0095,
        Value::structure(schema_structure([schema_item(
            0x0042_0097,
            Value::long_integer(12),
        )])),
    ));
    assert!(
        AddAttributeRequest::new(None, total_only)
            .validate()
            .is_valid()
    );
    assert!(
        ModifyAttributeRequest::new(
            None,
            None,
            crate::NewAttribute::new(schema_item(
                0x0042_0095,
                Value::structure(schema_structure([schema_item(
                    0x0042_0097,
                    Value::long_integer(12),
                )])),
            )),
        )
        .validate()
        .is_valid()
    );
}

#[test]
fn attribute_mutation_validation_rejects_source_prohibited_attributes() {
    let initial_date = || crate::NewAttribute::new(schema_item(0x0042_0039, Value::integer(1)));
    let add = AddAttributeRequest::new(None, initial_date());
    let modify = ModifyAttributeRequest::new(None, None, initial_date());
    let set = crate::SetAttributeRequest::new(None, initial_date());
    let adjust = AdjustAttributeRequest::new(
        None,
        crate::AttributeReference::tag(0x0042_0039),
        crate::AdjustmentType::INCREMENT,
        None,
    );
    let delete = DeleteAttributeRequest::new(
        None,
        Some(crate::CurrentAttribute::new(schema_item(
            0x0042_0094,
            Value::text_string("identifier".to_owned()),
        ))),
        None,
    );
    let vendor_attribute = crate::NewAttribute::new(schema_item(
        0x0042_0008,
        Value::structure(schema_structure([
            schema_item(0x0042_009D, Value::text_string("y".to_owned())),
            schema_item(0x0042_000A, Value::text_string("vendor-name".to_owned())),
            schema_item(0x0042_000B, Value::text_string("vendor-value".to_owned())),
        ])),
    ));
    let vendor_set = crate::SetAttributeRequest::new(None, vendor_attribute);
    let vendor_adjust = AdjustAttributeRequest::new(
        None,
        crate::AttributeReference::name("y", "attribute"),
        crate::AdjustmentType::INCREMENT,
        None,
    );
    let allowed_vendor = crate::SetAttributeRequest::new(
        None,
        crate::NewAttribute::new(schema_item(
            0x0042_0008,
            Value::structure(schema_structure([
                schema_item(0x0042_009D, Value::text_string("other_vendor".to_owned())),
                schema_item(0x0042_000A, Value::text_string("attribute".to_owned())),
                schema_item(0x0042_000B, Value::text_string("vendor-value".to_owned())),
            ])),
        )),
    );

    for report in [
        add.validate(),
        modify.validate(),
        set.validate(),
        adjust.validate(),
        delete.validate(),
        vendor_set.validate(),
        vendor_adjust.validate(),
    ] {
        assert!(
            report
                .issues()
                .iter()
                .any(|issue| { issue.code().as_str() == "KMIPKIT-0051-R282" })
        );
    }
    assert!(add.validate().issues().iter().any(|issue| {
        issue.source().to_string() == "KMIP v2.1 4.25"
            && issue.path().to_string() == "Request Payload.New Attribute.tag(0x420039)"
    }));
    assert!(vendor_set.validate().issues().iter().any(|issue| {
        issue.source().to_string() == "KMIP v2.1 4.60"
            && issue.path().to_string() == "Request Payload.New Attribute.tag(0x420008)"
    }));
    assert!(allowed_vendor.validate().is_valid());
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

    let report = request.validate();
    assert!(!report.is_valid());
    assert_eq!(
        report.issues()[0].source().to_string(),
        "KMIP v2.1 §6.1.6, Table 179"
    );
    assert!(!format!("{report:?}").contains("opaque-request"));
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

    let report = request.validate();
    assert!(!report.is_valid());
    assert_eq!(
        report.issues()[0].source().to_string(),
        "KMIP v2.1 §2.8, Table 9"
    );
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

    let report = request.validate();
    assert!(!report.is_valid());
    assert_eq!(
        report.issues()[0].source().to_string(),
        "KMIP v2.1 §6.1.14, Table 205"
    );
}

#[test]
fn revoke_validation_rejects_compromise_date_for_unrelated_reason() {
    let request = RevokeRequest::new(RevocationReason::UNSPECIFIED)
        .with_compromise_occurrence_date(1_700_000_000);

    let report = request.validate();
    assert!(!report.is_valid());
    assert_eq!(
        report.issues()[0].source().to_string(),
        "KMIP v2.1 §6.1.44, Table 295"
    );
}

#[test]
fn validate_request_validation_rejects_an_empty_chain() {
    let report = ValidateRequest::new().validate();
    assert!(!report.is_valid());
    assert_eq!(
        report.issues()[0].source().to_string(),
        "KMIP v2.1 §6.1.57, Table 340"
    );
}

#[test]
fn query_validation_rejects_an_empty_function_list() {
    let report = QueryRequest::new([]).validate();

    assert!(!report.is_valid());
    assert_eq!(report.issues().len(), 1);
    assert_eq!(
        report.issues()[0].path().to_string(),
        "Request Payload.Query Function"
    );
    assert_eq!(
        report.issues()[0].source().to_string(),
        "KMIP v2.1 §6.1.40, Table 282"
    );
}

#[test]
fn query_validation_rejects_reserved_function_but_preserves_extension_values() {
    let invalid = QueryRequest::new([QueryFunction::from_raw(15)]).validate();
    assert!(!invalid.is_valid());
    assert_eq!(
        invalid.issues()[0].source().to_string(),
        "KMIP v2.1 §11.44, Table 476"
    );

    let extension = QueryRequest::new([QueryFunction::from_raw(0x8000_0042)]).validate();
    assert!(extension.is_valid());
}

#[test]
fn join_split_key_validation_checks_references_and_attribute_multiplicity() {
    let missing_references = JoinSplitKeyRequest::new(ObjectType::from_raw(7), Vec::new());
    let report = missing_references.validate();
    assert!(!report.is_valid());
    assert_eq!(
        report.issues()[0].source().to_string(),
        "KMIP v2.1 §6.1.27, Table 244; §2.8, Table 9"
    );

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
    .expect("generic AttributeSet preserves operation-level multiplicity violations");
    let reference = || {
        SplitKeyReference::new(UniqueIdentifier::TextString("split-key".to_owned()))
            .with_split_key_threshold(1)
    };
    let request = JoinSplitKeyRequest::new(ObjectType::from_raw(7), vec![reference(), reference()])
        .with_attributes(attributes);
    let report = request.validate();
    assert!(!report.is_valid());
    assert_eq!(
        report.issues()[0].source().to_string(),
        "KMIP v2.1 §4, Table 46; §6.1.27, Table 244"
    );
}

#[test]
fn join_split_key_validation_requires_reference_threshold_context_and_count() {
    let identifier = UniqueIdentifier::TextString("split-key".to_owned());
    let missing_threshold = JoinSplitKeyRequest::new(
        ObjectType::from_raw(7),
        vec![SplitKeyReference::new(identifier.clone())],
    )
    .validate();
    assert!(!missing_threshold.is_valid());
    assert_eq!(
        missing_threshold.issues()[0].source().to_string(),
        "KMIP v2.1 §6.1.27, Table 244; §2.8, Table 9"
    );

    let insufficient_count = JoinSplitKeyRequest::new(
        ObjectType::from_raw(7),
        vec![SplitKeyReference::new(identifier).with_split_key_threshold(2)],
    )
    .validate();
    assert!(!insufficient_count.is_valid());
    assert_eq!(
        insufficient_count.issues()[0].source().to_string(),
        "KMIP v2.1 §6.1.27, Table 244; §2.8, Table 9"
    );
}

#[test]
fn constraints_validation_checks_known_nested_shapes_and_preserves_empty_values() {
    const CONSTRAINT: u32 = 0x0042_0169;
    const OBJECT_TYPES: u32 = 0x0042_0167;

    let mut malformed_constraint = kmipkit_ttlv::Structure::new();
    malformed_constraint
        .try_push(
            Item::new(
                RawTag::new(OBJECT_TYPES)
                    .expect("Object Types is a valid raw tag")
                    .try_checked()
                    .expect("Object Types is an allocated KMIP tag"),
                Value::integer(7),
            )
            .expect("an invalid known nested type remains valid generic TTLV"),
        )
        .expect("Constraint fixture is valid generic TTLV");
    let mut constraints = kmipkit_ttlv::Structure::new();
    constraints
        .try_push(
            Item::new(
                RawTag::new(CONSTRAINT)
                    .expect("Constraint is a valid raw tag")
                    .try_checked()
                    .expect("Constraint is an allocated KMIP tag"),
                Value::structure(malformed_constraint),
            )
            .expect("nested Constraint is valid generic TTLV"),
        )
        .expect("Constraints fixture is valid generic TTLV");
    let report = SetConstraintsRequest::new(constraints).validate();
    assert!(!report.is_valid());
    assert_eq!(
        report.issues()[0].source().to_string(),
        "KMIP v2.1 §7.6, Table 357"
    );

    assert!(
        SetConstraintsRequest::new(kmipkit_ttlv::Structure::new())
            .validate()
            .is_valid()
    );
}

#[test]
fn defaults_validation_checks_required_attribute_container_and_preserves_empty_defaults() {
    const OBJECT_DEFAULTS: u32 = 0x0042_0153;
    const OBJECT_TYPE: u32 = 0x0042_0057;

    let mut object_default = kmipkit_ttlv::Structure::new();
    object_default
        .try_push(
            Item::new(
                RawTag::new(OBJECT_TYPE)
                    .expect("Object Type is a valid raw tag")
                    .try_checked()
                    .expect("Object Type is an allocated KMIP tag"),
                Value::enumeration(2),
            )
            .expect("Object Type value is valid TTLV"),
        )
        .expect("Object Defaults fixture is valid generic TTLV");
    let mut defaults = kmipkit_ttlv::Structure::new();
    defaults
        .try_push(
            Item::new(
                RawTag::new(OBJECT_DEFAULTS)
                    .expect("Object Defaults is a valid raw tag")
                    .try_checked()
                    .expect("Object Defaults is an allocated KMIP tag"),
                Value::structure(object_default),
            )
            .expect("nested Object Defaults is valid generic TTLV"),
        )
        .expect("Defaults Information fixture is valid generic TTLV");
    let report = SetDefaultsRequest::new()
        .with_defaults_information(defaults)
        .validate();
    assert!(!report.is_valid());
    assert_eq!(
        report.issues()[0].source().to_string(),
        "KMIP v2.1 §7.22, Table 374"
    );

    assert!(
        SetDefaultsRequest::new()
            .with_defaults_information(kmipkit_ttlv::Structure::new())
            .validate()
            .is_valid()
    );
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

    let report = request.validate();
    assert!(!report.is_valid());
    assert_eq!(
        report.issues()[0].source().to_string(),
        "KMIP v2.1 §6.1.9, Table 189 and Table 191"
    );
}

#[test]
fn recertify_validation_rejects_missing_request_and_existing_identifier() {
    let report = RecertifyRequest::new().validate();
    assert!(!report.is_valid());
    assert_eq!(
        report.issues()[0].source().to_string(),
        "KMIP v2.1 §6.1.45, Table 300"
    );
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

    let report = RekeyRequest::new().with_attributes(attributes).validate();
    assert!(!report.is_valid());
    assert!(
        report.issues()[0]
            .source()
            .to_string()
            .contains("Table 305")
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

    let report = RekeyKeyPairRequest::new()
        .with_private_key_attributes(attributes)
        .validate();
    assert!(!report.is_valid());
    assert!(
        report.issues()[0]
            .source()
            .to_string()
            .contains("Table 310")
    );
}

#[test]
fn get_validation_checks_the_supplied_key_wrapping_specification() {
    const WRAPPING_METHOD: u32 = 0x0042_009E;

    let specification = schema_structure([schema_item(
        WRAPPING_METHOD,
        Value::text_string("secret-canary".to_owned()),
    )]);
    let report = GetRequest::new()
        .with_key_wrapping_specification(specification)
        .validate();

    assert!(report.issues().iter().any(|issue| {
        issue.code().as_str() == "KMIPKIT-0051-R229"
            && issue.path().to_string()
                == "Request Payload.Key Wrapping Specification.Wrapping Method"
            && issue.source().to_string() == "KMIP v2.1 §7.18, Table 370"
    }));
    assert!(!format!("{report:?}").contains("secret-canary"));
}

#[test]
fn export_validation_checks_the_supplied_key_wrapping_specification() {
    const WRAPPING_METHOD: u32 = 0x0042_009E;

    let specification = schema_structure([schema_item(
        WRAPPING_METHOD,
        Value::text_string("secret-canary".to_owned()),
    )]);
    let report = ExportRequest::new()
        .with_key_wrapping_specification(specification)
        .validate();

    assert!(report.issues().iter().any(|issue| {
        issue.code().as_str() == "KMIPKIT-0051-R229"
            && issue.path().to_string()
                == "Request Payload.Key Wrapping Specification.Wrapping Method"
            && issue.source().to_string() == "KMIP v2.1 §7.18, Table 370"
    }));
    assert!(!format!("{report:?}").contains("secret-canary"));
}

#[test]
fn login_validation_checks_optional_usage_limits_structure() {
    const USAGE_LIMITS_TOTAL: u32 = 0x0042_0097;
    const USAGE_LIMITS_COUNT: u32 = 0x0042_0096;
    const USAGE_LIMITS_UNIT: u32 = 0x0042_0098;

    let usage_limits = schema_structure([
        schema_item(USAGE_LIMITS_TOTAL, Value::long_integer(10)),
        schema_item(USAGE_LIMITS_COUNT, Value::long_integer(1)),
        schema_item(
            USAGE_LIMITS_UNIT,
            Value::text_string("secret-canary".to_owned()),
        ),
    ]);
    let report = LoginRequest::new()
        .with_usage_limits(usage_limits)
        .validate();

    assert!(report.issues().iter().any(|issue| {
        issue.code().as_str() == "KMIPKIT-0051-R244"
            && issue.path().to_string() == "Request Payload.Usage Limits.Usage Limits Unit"
            && issue.source().to_string() == "KMIP v2.1 §7.40, Table 392"
    }));
    assert!(!format!("{report:?}").contains("secret-canary"));
}

#[test]
fn import_validation_recurses_into_the_selected_managed_object() {
    const SYMMETRIC_KEY: u32 = 0x0042_008F;
    const KEY_BLOCK: u32 = 0x0042_0040;
    const KEY_FORMAT_TYPE: u32 = 0x0042_0042;

    let key_block = schema_structure([schema_item(
        KEY_FORMAT_TYPE,
        Value::text_string("secret-canary".to_owned()),
    )]);
    let symmetric_key = schema_item(
        SYMMETRIC_KEY,
        Value::structure(schema_structure([schema_item(
            KEY_BLOCK,
            Value::structure(key_block),
        )])),
    );
    let attributes = AttributeSet::try_new([]).expect("empty Attributes are valid");
    let report = ImportRequest::new(
        UniqueIdentifier::TextString("object-id".to_owned()),
        ObjectType::from_raw(2),
        attributes,
        symmetric_key,
    )
    .validate();

    assert!(report.issues().iter().any(|issue| {
        issue.code().as_str() == "KMIPKIT-0051-R262"
            && issue.path().to_string() == "Request Payload.Any Object.Key Block.Key Format Type"
            && issue.source().to_string() == "KMIP v2.1 §3.1, Table 12"
    }));
    assert!(!format!("{report:?}").contains("secret-canary"));
}

#[test]
fn import_and_register_validation_reject_known_object_type_mismatches() {
    const OPAQUE_OBJECT: u32 = 0x0042_005B;
    const OPAQUE_DATA_TYPE: u32 = 0x0042_0059;
    const OPAQUE_DATA_VALUE: u32 = 0x0042_005A;
    let opaque_object = || {
        schema_item(
            OPAQUE_OBJECT,
            Value::structure(schema_structure([
                schema_item(OPAQUE_DATA_TYPE, Value::enumeration(1)),
                schema_item(OPAQUE_DATA_VALUE, Value::byte_string(vec![0xCA, 0xFE])),
            ])),
        )
    };

    let import_report = ImportRequest::new(
        UniqueIdentifier::TextString("object-id".to_owned()),
        ObjectType::from_raw(2),
        AttributeSet::new(),
        opaque_object(),
    )
    .validate();
    assert!(import_report.issues().iter().any(|issue| {
        issue.code().as_str() == "KMIPKIT-0051-R092"
            && issue.path().to_string() == "Request Payload.Any Object"
            && issue.source().to_string() == "KMIP v2.1 §6.1.25, Table 238"
    }));

    let register_report = RegisterRequest::new(
        ObjectType::from_raw(2),
        AttributeSet::new(),
        opaque_object(),
    )
    .validate();
    assert!(register_report.issues().iter().any(|issue| {
        issue.code().as_str() == "KMIPKIT-0051-R140"
            && issue.path().to_string() == "Request Payload.Any Object"
            && issue.source().to_string() == "KMIP v2.1 §6.1.43, Table 291"
    }));
    assert!(!format!("{import_report:?}{register_report:?}").contains("CAFE"));
}

#[test]
fn cryptographic_requests_validate_supplied_parameter_structures() {
    let reports = [
        EncryptRequest::new(None, None)
            .with_cryptographic_parameters(gcm_parameters_without_required_lengths())
            .validate(),
        DecryptRequest::new(None, None)
            .with_cryptographic_parameters(gcm_parameters_without_required_lengths())
            .validate(),
        MacRequest::new()
            .with_cryptographic_parameters(gcm_parameters_without_required_lengths())
            .validate(),
        MacVerifyRequest::new()
            .with_cryptographic_parameters(gcm_parameters_without_required_lengths())
            .validate(),
        SignRequest::new()
            .with_cryptographic_parameters(gcm_parameters_without_required_lengths())
            .validate(),
        SignatureVerifyRequest::new()
            .with_cryptographic_parameters(gcm_parameters_without_required_lengths())
            .validate(),
    ];

    for report in reports {
        assert!(report.issues().iter().any(|issue| {
            issue.code().as_str() == "KMIPKIT-0051-R219"
                && issue
                    .path()
                    .to_string()
                    .ends_with("Cryptographic Parameters.IV Length")
                && issue.source().to_string() == "KMIP v2.1 §4.16, Table 59"
        }));
        assert!(report.issues().iter().any(|issue| {
            issue.code().as_str() == "KMIPKIT-0051-R220"
                && issue
                    .path()
                    .to_string()
                    .ends_with("Cryptographic Parameters.Tag Length")
        }));
    }
}

#[test]
fn hash_validation_checks_cryptographic_parameter_requirements() {
    const BLOCK_CIPHER_MODE: u32 = 0x0042_0011;
    const HASHING_ALGORITHM: u32 = 0x0042_0038;
    let parameters = schema_structure([
        schema_item(HASHING_ALGORITHM, Value::enumeration(6)),
        schema_item(BLOCK_CIPHER_MODE, Value::enumeration(9)),
    ]);
    let request = HashRequest::new(parameters).with_data(OperationData::ByteString(
        SecretBytes::new(b"synthetic-input".to_vec()),
    ));

    let report = request.validate();

    assert!(report.issues().iter().any(|issue| {
        issue.code().as_str() == "KMIPKIT-0051-R219"
            && issue
                .path()
                .to_string()
                .ends_with("Cryptographic Parameters.IV Length")
            && issue.source().to_string() == "KMIP v2.1 §4.16, Table 59"
    }));
    assert!(report.issues().iter().any(|issue| {
        issue.code().as_str() == "KMIPKIT-0051-R220"
            && issue
                .path()
                .to_string()
                .ends_with("Cryptographic Parameters.Tag Length")
    }));
    assert!(!format!("{report:?}").contains("synthetic-input"));

    let missing_algorithm = HashRequest::new(Structure::new()).validate();
    assert!(missing_algorithm.issues().iter().any(|issue| {
        issue.code().as_str() == "KMIPKIT-0051-P005"
            && issue.path().to_string() == "Request Payload.Cryptographic Parameters"
            && issue.source().to_string() == "KMIP v2.1 §6.1.24, Table 235"
    }));
}

#[test]
fn diagnostic_requests_preflight_reject_extensions_that_override_defined_fields() {
    let interop_report = InteropRequest::new(1, crate::SecretText::new("interop-id".to_owned()))
        .with_extension(schema_item(
            0x0042_0160,
            Value::text_string("secret-canary".to_owned()),
        ))
        .validate();
    assert!(interop_report.issues().iter().any(|issue| {
        issue.code().as_str() == "KMIPKIT-0051-P005"
            && issue.path().to_string() == "Request Payload.Interop Function"
            && issue.source().to_string() == "KMIP v2.1 §6.1.26, Table 241"
    }));

    let log_report = LogRequest::new(crate::SecretText::new("log-message".to_owned()))
        .with_extension(schema_item(
            0x0042_0141,
            Value::text_string("secret-canary".to_owned()),
        ))
        .validate();
    assert!(log_report.issues().iter().any(|issue| {
        issue.code().as_str() == "KMIPKIT-0051-P005"
            && issue.path().to_string() == "Request Payload.Log Message"
            && issue.source().to_string() == "KMIP v2.1 §6.1.29, Table 250"
    }));
    assert!(!format!("{interop_report:?}{log_report:?}").contains("secret-canary"));
}

#[test]
fn derive_key_validation_recurses_into_cryptographic_parameters() {
    let attributes = AttributeSet::try_new([
        schema_item(0x0042_002a, Value::integer(256)),
        schema_item(0x0042_0028, Value::enumeration(3)),
    ])
    .expect("the symmetric-key attributes use catalogued TTLV types");
    let request = DeriveKeyRequest::new(
        ObjectType::from_raw(2),
        vec![UniqueIdentifier::TextString("source".to_owned())],
        DerivationMethod::from_raw(4),
        DerivationParameters::new()
            .with_cryptographic_parameters(gcm_parameters_without_required_lengths()),
        attributes,
    );

    let report = request.validate();

    assert!(report.issues().iter().any(|issue| {
        issue.code().as_str() == "KMIPKIT-0051-R219"
            && issue.path().to_string()
                == "Request Payload.Derivation Parameters.Cryptographic Parameters.IV Length"
            && issue.source().to_string() == "KMIP v2.1 §4.16, Table 59"
    }));
    assert!(report.issues().iter().any(|issue| {
        issue.code().as_str() == "KMIPKIT-0051-R220"
            && issue.path().to_string()
                == "Request Payload.Derivation Parameters.Cryptographic Parameters.Tag Length"
    }));
}

#[test]
fn derive_key_validation_checks_known_derivation_parameter_extensions() {
    let attributes = AttributeSet::try_new([
        schema_item(0x0042_002a, Value::integer(256)),
        schema_item(0x0042_0028, Value::enumeration(3)),
    ])
    .expect("the symmetric-key attributes use catalogued TTLV types");
    let parameters = DerivationParameters::new()
        .with_extension(schema_item(
            0x0042_003c,
            Value::text_string("secret-canary".to_owned()),
        ))
        .expect("the generic extension is valid as TTLV");
    let request = DeriveKeyRequest::new(
        ObjectType::from_raw(2),
        vec![UniqueIdentifier::TextString("source".to_owned())],
        DerivationMethod::from_raw(4),
        parameters,
        attributes,
    );

    let report = request.validate();

    assert!(report.issues().iter().any(|issue| {
        issue.code().as_str() == "KMIPKIT-0051-R225"
            && issue.path().to_string() == "Request Payload.Derivation Parameters.Iteration Count"
            && issue.source().to_string() == "KMIP v2.1 §7.12, Table 364"
    }));
    assert!(!format!("{report:?}").contains("secret-canary"));
}

#[test]
fn reprovision_validation_checks_supplied_certificate_shapes() {
    let malformed_request = schema_structure([schema_item(0x0042_0019, Value::enumeration(1))]);
    let request = ReProvisionRequest::new().with_certificate_request(malformed_request);

    let report = request.validate();

    assert!(report.issues().iter().any(|issue| {
        issue.code().as_str() == "KMIPKIT-0051-R252"
            && issue.path().to_string()
                == "Request Payload.Certificate Request.Certificate Request Value"
            && issue.source().to_string() == "KMIP v2.1 §2.2, Table 3"
    }));

    let malformed_certificate = schema_structure([schema_item(0x0042_001d, Value::enumeration(1))]);
    let certificate_report = ReProvisionRequest::new()
        .with_certificate(malformed_certificate)
        .validate();
    assert!(certificate_report.issues().iter().any(|issue| {
        issue.code().as_str() == "KMIPKIT-0051-R250"
            && issue.path().to_string() == "Request Payload.Certificate.Certificate Value"
            && issue.source().to_string() == "KMIP v2.1 §2.1, Table 2"
    }));
}
