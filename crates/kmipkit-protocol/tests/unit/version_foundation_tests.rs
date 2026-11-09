//! Unit tests for multi-version protocol foundation (KMIPKIT-0069).

use std::cmp::Ordering;
use std::error::Error;

use crate::{
    CodecContext, DiscoverVersionsError, DiscoverVersionsRequest, ProtocolVersion, VersionGate,
    VersionGateError, VersionSpan,
};

#[test]
fn test_protocol_version_constants_and_ordering() {
    assert_eq!(ProtocolVersion::V2_0.major(), 2);
    assert_eq!(ProtocolVersion::V2_0.minor(), 0);
    assert_eq!(ProtocolVersion::V2_1.major(), 2);
    assert_eq!(ProtocolVersion::V2_1.minor(), 1);
    assert_eq!(ProtocolVersion::V3_0.major(), 3);
    assert_eq!(ProtocolVersion::V3_0.minor(), 0);

    assert!(ProtocolVersion::V2_0 < ProtocolVersion::V2_1);
    assert!(ProtocolVersion::V2_1 < ProtocolVersion::V3_0);
    assert_eq!(
        ProtocolVersion::V2_0.cmp(&ProtocolVersion::V2_1),
        Ordering::Less
    );

    assert_eq!(format!("{}", ProtocolVersion::V2_0), "2.0");
    assert_eq!(format!("{}", ProtocolVersion::V2_1), "2.1");
    assert_eq!(format!("{}", ProtocolVersion::V3_0), "3.0");
}

#[test]
fn test_protocol_version_catalog_membership() {
    assert!(ProtocolVersion::V2_0.is_catalogued());
    assert!(ProtocolVersion::V2_1.is_catalogued());
    assert!(ProtocolVersion::V3_0.is_catalogued());

    // Legacy 1.x is explicitly excluded from client catalog
    assert!(!ProtocolVersion::from_raw(1, 0).is_catalogued());
    assert!(!ProtocolVersion::from_raw(1, 4).is_catalogued());
    // Future uncatalogued versions are not yet catalogued (Option B)
    assert!(!ProtocolVersion::from_raw(3, 1).is_catalogued());
    assert!(!ProtocolVersion::from_raw(4, 0).is_catalogued());
}

#[test]
fn test_version_span_membership() {
    let span_2_1_only = VersionSpan {
        introduced: ProtocolVersion::V2_1,
        deprecated: None,
        removed: None,
    };
    assert!(!span_2_1_only.is_supported_in(ProtocolVersion::V2_0));
    assert!(span_2_1_only.is_supported_in(ProtocolVersion::V2_1));
    assert!(span_2_1_only.is_supported_in(ProtocolVersion::V3_0));

    let span_deprecated_in_3 = VersionSpan {
        introduced: ProtocolVersion::V2_0,
        deprecated: Some(ProtocolVersion::V2_1),
        removed: Some(ProtocolVersion::V3_0),
    };
    assert!(span_deprecated_in_3.is_supported_in(ProtocolVersion::V2_0));
    assert!(span_deprecated_in_3.is_supported_in(ProtocolVersion::V2_1));
    assert!(!span_deprecated_in_3.is_supported_in(ProtocolVersion::V3_0));
}

#[test]
fn test_version_gate_validates_operations_and_spans() {
    // Operation 0x0000_003B (Ping) was introduced in 2.1
    let ping_op = 0x0000_003B;
    let create_op = 0x0000_0001; // Introduced in 2.0
    let kmip3_op = 0x0000_003C; // Introduced in 3.0
    let unknown_op = 0x0000_9999;

    assert!(VersionGate::operation_span(create_op).is_some());
    assert!(VersionGate::operation_span(ping_op).is_some());
    assert!(VersionGate::operation_span(kmip3_op).is_some());
    assert_eq!(VersionGate::operation_span(unknown_op), None);

    // V2_1 allows Create and Ping, but rejects KMIP 3.0 operation
    assert!(VersionGate::validate_operation(ProtocolVersion::V2_1, ping_op).is_ok());
    assert!(VersionGate::validate_operation(ProtocolVersion::V2_1, create_op).is_ok());
    assert_eq!(
        VersionGate::validate_operation(ProtocolVersion::V2_1, kmip3_op).unwrap_err(),
        VersionGateError::OperationNotSupportedInVersion {
            operation: kmip3_op,
            negotiated: ProtocolVersion::V2_1,
            introduced: ProtocolVersion::V3_0,
        }
    );

    // V3_0 allows KMIP 3.0 operation
    assert!(VersionGate::validate_operation(ProtocolVersion::V3_0, kmip3_op).is_ok());

    // V2_0 allows Create but rejects Ping
    assert!(VersionGate::validate_operation(ProtocolVersion::V2_0, create_op).is_ok());
    let err = VersionGate::validate_operation(ProtocolVersion::V2_0, ping_op).unwrap_err();
    assert_eq!(
        err,
        VersionGateError::OperationNotSupportedInVersion {
            operation: ping_op,
            negotiated: ProtocolVersion::V2_0,
            introduced: ProtocolVersion::V2_1,
        }
    );

    // Uncatalogued version (e.g. 1.4 or 4.0) is rejected immediately
    let uncat_err =
        VersionGate::validate_operation(ProtocolVersion::from_raw(1, 4), create_op).unwrap_err();
    assert_eq!(
        uncat_err,
        VersionGateError::UnsupportedProtocolVersion(ProtocolVersion::from_raw(1, 4))
    );
}

#[test]
fn test_version_gate_validates_tags() {
    let rotate_attr = 0x0042_014A; // Rotate Automatic (2.1)
    let pqc_attr = 0x0042_0153; // PQC (3.0)
    let unknown_tag = 0x0042_9999;

    assert!(VersionGate::tag_span(rotate_attr).is_some());
    assert!(VersionGate::tag_span(pqc_attr).is_some());
    assert_eq!(VersionGate::tag_span(unknown_tag), None);

    // KMIP 2.0 rejects Rotate Automatic
    assert_eq!(
        VersionGate::validate_tag(ProtocolVersion::V2_0, rotate_attr).unwrap_err(),
        VersionGateError::AttributeNotSupportedInVersion {
            tag: rotate_attr,
            negotiated: ProtocolVersion::V2_0,
            introduced: ProtocolVersion::V2_1,
        }
    );

    // KMIP 2.1 accepts Rotate Automatic, rejects PQC attribute
    assert!(VersionGate::validate_tag(ProtocolVersion::V2_1, rotate_attr).is_ok());
    assert_eq!(
        VersionGate::validate_tag(ProtocolVersion::V2_1, pqc_attr).unwrap_err(),
        VersionGateError::AttributeNotSupportedInVersion {
            tag: pqc_attr,
            negotiated: ProtocolVersion::V2_1,
            introduced: ProtocolVersion::V3_0,
        }
    );

    // KMIP 3.0 accepts PQC attribute
    assert!(VersionGate::validate_tag(ProtocolVersion::V3_0, pqc_attr).is_ok());

    // Uncatalogued version is rejected
    assert_eq!(
        VersionGate::validate_tag(ProtocolVersion::from_raw(1, 2), rotate_attr).unwrap_err(),
        VersionGateError::UnsupportedProtocolVersion(ProtocolVersion::from_raw(1, 2))
    );
}

#[test]
fn test_version_gate_uses_exact_kmip_30_tag_assignments() {
    // Object Defaults and Protection Storage Masks are assigned by KMIP 2.0/2.1.
    let object_defaults = 0x0042_0153;
    let protection_storage_masks = 0x0042_015F;
    // NIST Security Category is assigned in KMIP 3.0 §11.58, Table 605.
    let nist_security_category = 0x0042_01C2;

    assert_eq!(VersionGate::tag_span(object_defaults), None);
    assert_eq!(VersionGate::tag_span(protection_storage_masks), None);
    assert_eq!(
        VersionGate::tag_span(nist_security_category),
        Some(VersionSpan::since(ProtocolVersion::V3_0))
    );
    assert!(VersionGate::validate_tag(ProtocolVersion::V2_1, object_defaults).is_ok());
    assert!(VersionGate::validate_tag(ProtocolVersion::V2_1, protection_storage_masks).is_ok());
    assert_eq!(
        VersionGate::validate_tag(ProtocolVersion::V2_1, nist_security_category).unwrap_err(),
        VersionGateError::AttributeNotSupportedInVersion {
            tag: nist_security_category,
            negotiated: ProtocolVersion::V2_1,
            introduced: ProtocolVersion::V3_0,
        }
    );
    assert!(VersionGate::validate_tag(ProtocolVersion::V3_0, nist_security_category).is_ok());
}

#[test]
fn test_version_gate_error_formatting() {
    let err1 = VersionGateError::UnsupportedProtocolVersion(ProtocolVersion::from_raw(1, 2));
    assert!(format!("{err1}").contains("is not catalogued or supported"));

    let err2 = VersionGateError::OperationNotSupportedInVersion {
        operation: 0x0000_003B,
        negotiated: ProtocolVersion::V2_0,
        introduced: ProtocolVersion::V2_1,
    };
    assert!(format!("{err2}").contains("requires KMIP 2.1 or newer, but session is 2.0"));

    let err3 = VersionGateError::AttributeNotSupportedInVersion {
        tag: 0x0042_014A,
        negotiated: ProtocolVersion::V2_0,
        introduced: ProtocolVersion::V2_1,
    };
    assert!(format!("{err3}").contains("requires KMIP 2.1 or newer, but session is 2.0"));
}

#[test]
fn test_codec_context_initialization() {
    let ctx = CodecContext::new(ProtocolVersion::V2_1);
    assert_eq!(ctx.version, ProtocolVersion::V2_1);
}

#[test]
fn test_multi_version_discover_versions_request_and_response() {
    let req =
        DiscoverVersionsRequest::with_versions(vec![ProtocolVersion::V2_1, ProtocolVersion::V2_0])
            .expect("valid catalogued versions");
    assert_eq!(
        req.protocol_versions(),
        &[ProtocolVersion::V2_1, ProtocolVersion::V2_0]
    );

    let payload = req.to_ttlv_payload().expect("serializes to ttlv payload");
    assert_eq!(payload.view().children().len(), 2);

    // Empty list fails validation
    let empty_err = DiscoverVersionsRequest::with_versions(vec![]).unwrap_err();
    assert_eq!(empty_err, DiscoverVersionsError::InvalidOfferedVersions);
    assert!(format!("{empty_err}").contains("offered versions list is invalid"));
    assert!(empty_err.source().is_none());

    // Other error display formatting
    assert!(format!("{}", DiscoverVersionsError::MalformedProtocolVersion).contains("malformed"));
    assert!(
        format!("{}", DiscoverVersionsError::UnofferedProtocolVersion).contains("did not offer")
    );

    // Uncatalogued version fails validation
    let uncat_err = DiscoverVersionsRequest::with_versions(vec![
        ProtocolVersion::V2_1,
        ProtocolVersion::from_raw(1, 4),
    ])
    .unwrap_err();
    assert_eq!(uncat_err, DiscoverVersionsError::InvalidOfferedVersions);

    // Default remains 2.1 only (Zero-Breakage)
    let default_req = DiscoverVersionsRequest::new();
    assert_eq!(default_req.protocol_versions(), &[ProtocolVersion::V2_1]);
    let default_payload = default_req.to_ttlv_payload().expect("serializes to ttlv");
    assert_eq!(default_payload.view().children().len(), 1);
}
