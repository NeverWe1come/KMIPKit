use super::{NestedStructureSchema, RequestPayloadSchema, ValidationSchema, parse_schema, span};

#[test]
#[allow(clippy::too_many_lines)] // Keep the public request and nested-schema catalog explicit in one table.
fn schema_parser_accepts_every_public_request_and_nested_schema_name() {
    let requests = [
        "Activate",
        "AddAttribute",
        "AdjustAttribute",
        "Archive",
        "Cancel",
        "Certify",
        "Check",
        "Create",
        "CreateKeyPair",
        "CreateSplitKey",
        "Decrypt",
        "DelegatedLogin",
        "DeleteAttribute",
        "DeriveKey",
        "Destroy",
        "DiscoverVersions",
        "Encrypt",
        "Export",
        "Get",
        "GetAttributeList",
        "GetAttributes",
        "GetConstraints",
        "GetUsageAllocation",
        "Hash",
        "Import",
        "Interop",
        "JoinSplitKey",
        "Locate",
        "Log",
        "Login",
        "Logout",
        "Mac",
        "MacVerify",
        "ModifyAttribute",
        "ObtainLease",
        "Ping",
        "Pkcs11",
        "Poll",
        "Process",
        "Query",
        "QueryAsyncRequests",
        "ReProvision",
        "Recertify",
        "Recover",
        "Register",
        "Rekey",
        "RekeyKeyPair",
        "Revoke",
        "RngRetrieve",
        "RngSeed",
        "SetAttribute",
        "SetConstraints",
        "SetDefaults",
        "SetEndpointRole",
        "Sign",
        "SignatureVerify",
        "Validate",
    ];
    for name in requests {
        let schema = format!("RequestPayload/{name}");
        assert!(
            matches!(
                parse_schema(schema.as_bytes()),
                Ok(Some(ValidationSchema::RequestPayload(_)))
            ),
            "request schema {name} must be recognized"
        );
    }

    let nested = [
        "Constraints",
        "Constraint",
        "ObjectTypes",
        "ObjectGroups",
        "Attributes",
        "DefaultsInformation",
        "ObjectDefaults",
        "CryptographicParameters",
        "DerivationParameters",
        "KeyWrappingSpecification",
        "EncryptionKeyInformation",
        "MacSignatureKeyInformation",
        "Rights",
        "Right",
        "UsageLimits",
        "Objects",
        "Operations",
        "CurrentAttribute",
        "NewAttribute",
        "Certificate",
        "CertificateRequest",
        "AsynchronousCorrelationValues",
        "Ticket",
        "ProtocolVersion",
        "RevocationReason",
        "Pkcs11InputParameters",
        "Pkcs11Interface",
        "KeyBlock",
        "KeyValue",
        "KeyWrappingData",
        "OpaqueObject",
    ];
    for name in nested {
        let schema = format!("Nested/{name}");
        assert!(
            matches!(
                parse_schema(schema.as_bytes()),
                Ok(Some(ValidationSchema::Nested(_)))
            ),
            "nested schema {name} must be recognized"
        );
    }
}

#[test]
fn schema_parser_handles_generic_unsupported_and_malformed_identifiers() {
    assert!(matches!(parse_schema(b""), Ok(None)));
    assert!(matches!(
        parse_schema(b"Unsupported"),
        Ok(Some(ValidationSchema::Unsupported))
    ));
    assert_eq!(parse_schema(&[0xff]), Err(super::ERROR_INVALID_INPUT));
    assert_eq!(
        parse_schema(b"RequestPayload/Unknown"),
        Err(super::ERROR_INVALID_INPUT)
    );
    assert_eq!(
        parse_schema(b"Nested/Unknown"),
        Err(super::ERROR_INVALID_INPUT)
    );
    assert_eq!(
        parse_schema(b"Unknown/Create"),
        Err(super::ERROR_INVALID_INPUT)
    );
}

#[test]
fn byte_span_uses_null_for_empty_and_borrows_nonempty_bytes() {
    let empty = span(&[]);
    assert!(empty.data.is_null());
    assert_eq!(empty.length, 0);

    let bytes = [0x54, 0x01, 0x73];
    let nonempty = span(&bytes);
    assert_eq!(nonempty.data, bytes.as_ptr());
    assert_eq!(nonempty.length, bytes.len() as u64);
}

#[test]
fn request_and_nested_names_map_to_the_expected_typed_schemas() {
    assert!(matches!(
        parse_schema(b"RequestPayload/Cancel"),
        Ok(Some(ValidationSchema::RequestPayload(
            RequestPayloadSchema::Cancel
        )))
    ));
    assert!(matches!(
        parse_schema(b"Nested/KeyBlock"),
        Ok(Some(ValidationSchema::Nested(
            NestedStructureSchema::KeyBlock
        )))
    ));
}

#[test]
fn schema_parser_accepts_managed_object_nested_schema_names() {
    for (name, expected) in [
        ("PgpKey", NestedStructureSchema::PgpKey),
        ("SecretData", NestedStructureSchema::SecretData),
        ("SplitKey", NestedStructureSchema::SplitKey),
    ] {
        let schema = format!("Nested/{name}");
        assert_eq!(
            parse_schema(schema.as_bytes()),
            Ok(Some(ValidationSchema::Nested(expected))),
            "nested schema {name} must map to its Rust variant"
        );
    }
}
