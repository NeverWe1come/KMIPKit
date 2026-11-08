//! Unit tests for OASIS KMIP 3.0 operations, system objects, and `VersionGate` (KMIPKIT-0071).

use crate::{ObjectClass, ObjectType, ProtocolVersion, VersionGate, VersionGateError};

const CREATE_USER_OPERATION: u32 = 0x0000_003C;
const CREATE_GROUP_OPERATION: u32 = 0x0000_003D;
const DEACTIVATE_OPERATION: u32 = 0x0000_003E;
const ENCAPSULATE_OPERATION: u32 = 0x0000_003F;
const DECAPSULATE_OPERATION: u32 = 0x0000_0040;
const RE_ENCRYPT_OPERATION: u32 = 0x0000_0041;

#[test]
fn kmip30_allows_all_six_new_kmip30_operations() {
    let kmip30_ops = [
        CREATE_USER_OPERATION,
        CREATE_GROUP_OPERATION,
        DEACTIVATE_OPERATION,
        ENCAPSULATE_OPERATION,
        DECAPSULATE_OPERATION,
        RE_ENCRYPT_OPERATION,
    ];

    for op in kmip30_ops {
        assert!(
            VersionGate::validate_operation(ProtocolVersion::V3_0, op).is_ok(),
            "Operation 0x{op:08X} must be permitted under KMIP 3.0"
        );
    }
}

#[test]
fn kmip20_and_kmip21_prohibit_all_six_kmip30_operations() {
    let kmip30_ops = [
        CREATE_USER_OPERATION,
        CREATE_GROUP_OPERATION,
        DEACTIVATE_OPERATION,
        ENCAPSULATE_OPERATION,
        DECAPSULATE_OPERATION,
        RE_ENCRYPT_OPERATION,
    ];

    for op in kmip30_ops {
        // Prohibited in KMIP 2.0
        let err20 = VersionGate::validate_operation(ProtocolVersion::V2_0, op)
            .expect_err("KMIP 3.0 operation must be rejected under KMIP 2.0");
        assert_eq!(
            err20,
            VersionGateError::OperationNotSupportedInVersion {
                operation: op,
                negotiated: ProtocolVersion::V2_0,
                introduced: ProtocolVersion::V3_0,
            }
        );

        // Prohibited in KMIP 2.1
        let err21 = VersionGate::validate_operation(ProtocolVersion::V2_1, op)
            .expect_err("KMIP 3.0 operation must be rejected under KMIP 2.1");
        assert_eq!(
            err21,
            VersionGateError::OperationNotSupportedInVersion {
                operation: op,
                negotiated: ProtocolVersion::V2_1,
                introduced: ProtocolVersion::V3_0,
            }
        );
    }
}

#[test]
fn kmip30_object_class_and_system_object_types() {
    assert_eq!(ObjectClass::CryptographicObject.to_raw(), 1);
    assert_eq!(ObjectClass::SystemObject.to_raw(), 2);
    assert_eq!(ObjectClass::MetadataObject.to_raw(), 3);

    assert_eq!(
        ObjectClass::from_raw(1),
        Some(ObjectClass::CryptographicObject)
    );
    assert_eq!(ObjectClass::from_raw(2), Some(ObjectClass::SystemObject));
    assert_eq!(ObjectClass::from_raw(3), Some(ObjectClass::MetadataObject));
    assert_eq!(ObjectClass::from_raw(99), None);

    // KMIP 3.0 System Objects in ObjectType
    assert_eq!(ObjectType::User.to_raw(), 0x0000_000B);
    assert_eq!(ObjectType::Group.to_raw(), 0x0000_000C);
    assert_eq!(ObjectType::Credential.to_raw(), 0x0000_000D);

    assert_eq!(ObjectType::from_raw(0x0000_000B), ObjectType::User);
    assert_eq!(ObjectType::from_raw(0x0000_000C), ObjectType::Group);
    assert_eq!(ObjectType::from_raw(0x0000_000D), ObjectType::Credential);
}
