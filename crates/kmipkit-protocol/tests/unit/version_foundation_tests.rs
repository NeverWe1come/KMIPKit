//! Unit tests for multi-version protocol foundation (KMIPKIT-0069).

use std::cmp::Ordering;

use crate::{
    DiscoverVersionsError, DiscoverVersionsRequest, ProtocolVersion, VersionGate, VersionGateError,
    VersionSpan,
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

    // V2_1 allows both Ping and Create
    assert!(VersionGate::validate_operation(ProtocolVersion::V2_1, ping_op).is_ok());
    assert!(VersionGate::validate_operation(ProtocolVersion::V2_1, create_op).is_ok());

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
fn test_multi_version_discover_versions_request_and_response() {
    let req =
        DiscoverVersionsRequest::with_versions(vec![ProtocolVersion::V2_1, ProtocolVersion::V2_0])
            .expect("valid catalogued versions");
    assert_eq!(
        req.protocol_versions(),
        &[ProtocolVersion::V2_1, ProtocolVersion::V2_0]
    );

    // Empty list fails validation
    let empty_err = DiscoverVersionsRequest::with_versions(vec![]).unwrap_err();
    assert_eq!(empty_err, DiscoverVersionsError::InvalidOfferedVersions);

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
}
