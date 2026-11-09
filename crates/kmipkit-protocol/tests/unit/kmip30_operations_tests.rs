//! Unit tests for OASIS KMIP 3.0 operations, system objects, and `VersionGate` (KMIPKIT-0071).

use crate::{ObjectClass, ObjectType, ProtocolVersion, VersionGate, VersionGateError};

const CREATE_GROUP_OPERATION: u32 = 0x0000_003C;
const OBLITERATE_OPERATION: u32 = 0x0000_003D;
const CREATE_USER_OPERATION: u32 = 0x0000_003E;
const CREATE_CREDENTIAL_OPERATION: u32 = 0x0000_003F;
const DEACTIVATE_OPERATION: u32 = 0x0000_0040;
const ENCAPSULATE_OPERATION: u32 = 0x0000_0041;
const DECAPSULATE_OPERATION: u32 = 0x0000_0042;

#[test]
fn kmip30_allows_the_seven_operations_assigned_in_table_583() {
    let kmip30_ops = [
        CREATE_GROUP_OPERATION,
        OBLITERATE_OPERATION,
        CREATE_USER_OPERATION,
        CREATE_CREDENTIAL_OPERATION,
        DEACTIVATE_OPERATION,
        ENCAPSULATE_OPERATION,
        DECAPSULATE_OPERATION,
    ];

    for op in kmip30_ops {
        assert!(
            VersionGate::validate_operation(ProtocolVersion::V3_0, op).is_ok(),
            "Operation 0x{op:08X} must be permitted under KMIP 3.0"
        );
    }
}

#[test]
fn kmip20_and_kmip21_prohibit_all_seven_kmip30_operations() {
    let kmip30_ops = [
        CREATE_GROUP_OPERATION,
        OBLITERATE_OPERATION,
        CREATE_USER_OPERATION,
        CREATE_CREDENTIAL_OPERATION,
        DEACTIVATE_OPERATION,
        ENCAPSULATE_OPERATION,
        DECAPSULATE_OPERATION,
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
fn version_gate_rejects_unassigned_standard_operation_codes_but_keeps_extensions() {
    // KMIP 3.0 §11.37, Table 583 assigns operation codes through 0x42;
    // 0x43 has no assigned Operation name in the pinned source catalog.
    assert!(VersionGate::validate_operation(ProtocolVersion::V3_0, 0x43).is_err());
    assert!(VersionGate::validate_operation(ProtocolVersion::V2_1, 0x9999).is_err());

    // Unknown extension values remain representable and can be sent as
    // vendor-defined operations within the KMIP Enumeration extension range.
    assert!(VersionGate::validate_operation(ProtocolVersion::V3_0, 0x8000_0001).is_ok());
}

// OASIS KMIP 2.1 §11.36 Table 470; KMIP 3.0 §§11.34–11.37 Tables 581–583.
#[test]
fn versioned_named_values_match_the_pinned_2_1_and_3_0_tables() {
    use crate::named_values_generated::values::versions::{v2_1, v3_0};

    let operations_2_1 = [
        (0x36, "Set Defaults"),
        (0x37, "Set Constraints"),
        (0x38, "Get Constraints"),
        (0x39, "Query Asynchronous Requests"),
        (0x3A, "Process"),
        (0x3B, "Ping"),
    ];
    for (raw, expected) in operations_2_1 {
        assert_eq!(v2_1::enumeration::operation::name(raw), Some(expected));
        assert_eq!(
            VersionGate::operation_span(raw).unwrap().introduced,
            ProtocolVersion::V2_1
        );
    }

    let operations_3_0 = [
        (CREATE_GROUP_OPERATION, "Create Group"),
        (OBLITERATE_OPERATION, "Obliterate"),
        (CREATE_USER_OPERATION, "Create User"),
        (CREATE_CREDENTIAL_OPERATION, "Create Credential"),
        (DEACTIVATE_OPERATION, "Deactivate"),
        (ENCAPSULATE_OPERATION, "Encapsulate"),
        (DECAPSULATE_OPERATION, "Decapsulate"),
    ];
    for (raw, expected) in operations_3_0 {
        assert_eq!(v3_0::enumeration::operation::name(raw), Some(expected));
        assert_eq!(
            VersionGate::operation_span(raw).unwrap().introduced,
            ProtocolVersion::V3_0
        );
    }
    assert_eq!(v3_0::enumeration::operation::name(0x43), None);
    assert_eq!(VersionGate::operation_span(0x43), None);
}

#[test]
fn kmip30_object_class_and_system_object_types() {
    use crate::named_values_generated::values::versions::v3_0;

    assert_eq!(ObjectClass::USER.to_raw(), 1);
    assert_eq!(ObjectClass::SYSTEM.to_raw(), 2);
    assert_eq!(ObjectClass::from_raw(1).to_raw(), 1);
    assert_eq!(ObjectClass::from_raw(2).to_raw(), 2);
    assert_eq!(ObjectClass::from_raw(3).to_raw(), 3);
    assert_eq!(ObjectClass::from_raw(0x8000_0001).to_raw(), 0x8000_0001);
    assert_eq!(v3_0::enumeration::object_class::name(1), Some("User"));
    assert_eq!(v3_0::enumeration::object_class::name(2), Some("System"));
    assert_eq!(v3_0::enumeration::object_class::name(3), None);

    // KMIP 3.0 System Objects in ObjectType
    assert_eq!(ObjectType::User.to_raw(), 0x0000_000B);
    assert_eq!(ObjectType::Group.to_raw(), 0x0000_000C);
    assert_eq!(ObjectType::from_raw(0x0000_000D).to_raw(), 0x0000_000D);
    assert_eq!(ObjectType::from_raw(0x0000_000E).to_raw(), 0x0000_000E);
    assert_eq!(ObjectType::from_raw(0x0000_000F).to_raw(), 0x0000_000F);
    assert_eq!(ObjectType::from_raw(0x0000_0010).to_raw(), 0x0000_0010);
    assert_eq!(ObjectType::PASSWORD_CREDENTIAL.to_raw(), 0x0000_000D);
    assert_eq!(ObjectType::DEVICE_CREDENTIAL.to_raw(), 0x0000_000E);
    assert_eq!(
        ObjectType::ONE_TIME_PASSWORD_CREDENTIAL.to_raw(),
        0x0000_000F
    );
    assert_eq!(ObjectType::HASHED_PASSWORD_CREDENTIAL.to_raw(), 0x0000_0010);

    assert_eq!(ObjectType::from_raw(0x0000_000B), ObjectType::User);
    assert_eq!(ObjectType::from_raw(0x0000_000C), ObjectType::Group);
    assert_eq!(
        v3_0::enumeration::object_type::name(0x0D),
        Some("Password Credential")
    );
    assert_eq!(
        v3_0::enumeration::object_type::name(0x0E),
        Some("Device Credential")
    );
    assert_eq!(
        v3_0::enumeration::object_type::name(0x0F),
        Some("One Time Password Credential")
    );
    assert_eq!(
        v3_0::enumeration::object_type::name(0x10),
        Some("Hashed Password Credential")
    );
    assert_eq!(v3_0::enumeration::object_type::name(0x06), None);
}
