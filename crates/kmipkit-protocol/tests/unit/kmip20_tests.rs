//! OASIS KMIP 2.0 interoperability and conformance tests (KMIPKIT-0070).

use crate::{ProtocolVersion, VersionGate, VersionGateError};

const REGISTER_OPAQUE_CONTEXT: u32 = 0x0000_0036;
const REGISTER_OPAQUE: u32 = 0x0000_0037;
const SET_CONSTRAINTS: u32 = 0x0000_0038;
const GET_CONSTRAINTS: u32 = 0x0000_0039;
const EXPORT: u32 = 0x0000_003A;
const IMPORT: u32 = 0x0000_003B;

const CRYPTOGRAPHIC_ALGORITHM_TAG: u32 = 0x0042_0028;
const CRYPTOGRAPHIC_LENGTH_TAG: u32 = 0x0042_002A;
const CRYPTOGRAPHIC_PARAMETERS_TAG: u32 = 0x0042_002B;
const CRYPTOGRAPHIC_USAGE_MASK_TAG: u32 = 0x0042_002C;
const DIGEST_TAG: u32 = 0x0042_0032;
const KEY_VALUE_PRESENT_TAG: u32 = 0x0042_00BF;
const KEY_FORMAT_TYPE_TAG: u32 = 0x0042_0044;

#[test]
fn kmip20_allows_standard_operations() {
    let allowed_ops = [
        0x0000_0001, // Create
        0x0000_0002, // Create Key Pair
        0x0000_0003, // Register
        0x0000_0008, // Locate
        0x0000_000A, // Get
        0x0000_000B, // Get Attributes
        0x0000_000C, // Get Attribute List
        0x0000_000D, // Add Attribute
        0x0000_000E, // Modify Attribute
        0x0000_000F, // Delete Attribute
        0x0000_0012, // Activate
        0x0000_0013, // Revoke
        0x0000_0014, // Destroy
        0x0000_0018, // Query
        0x0000_001E, // Discover Versions
        0x0000_001F, // Encrypt
        0x0000_0020, // Decrypt
        0x0000_0021, // Sign
        0x0000_0022, // Signature Verify
        0x0000_0023, // MAC
        0x0000_0024, // MAC Verify
        0x0000_0025, // RNG Retrieve
        0x0000_0026, // RNG Seed
        0x0000_0027, // Hash
    ];

    for op in allowed_ops {
        assert!(
            VersionGate::validate_operation(ProtocolVersion::V2_0, op).is_ok(),
            "Operation 0x{op:08X} must be permitted under KMIP 2.0"
        );
    }
}

#[test]
fn kmip20_prohibits_all_six_kmip21_operations() {
    let prohibited_ops = [
        REGISTER_OPAQUE_CONTEXT,
        REGISTER_OPAQUE,
        SET_CONSTRAINTS,
        GET_CONSTRAINTS,
        EXPORT,
        IMPORT,
    ];

    for op in prohibited_ops {
        let err = VersionGate::validate_operation(ProtocolVersion::V2_0, op)
            .expect_err("KMIP 2.1 operation must be prohibited under KMIP 2.0");
        assert_eq!(
            err,
            VersionGateError::OperationNotSupportedInVersion {
                operation: op,
                negotiated: ProtocolVersion::V2_0,
                introduced: ProtocolVersion::V2_1,
            }
        );

        // Under 2.1, all 6 operations are permitted
        assert!(VersionGate::validate_operation(ProtocolVersion::V2_1, op).is_ok());
    }
}

#[test]
fn kmip20_prohibits_mutation_of_seven_cryptographic_attributes() {
    let rotate_only_attrs = [
        CRYPTOGRAPHIC_ALGORITHM_TAG,
        CRYPTOGRAPHIC_LENGTH_TAG,
        CRYPTOGRAPHIC_PARAMETERS_TAG,
        CRYPTOGRAPHIC_USAGE_MASK_TAG,
        DIGEST_TAG,
        KEY_VALUE_PRESENT_TAG,
        KEY_FORMAT_TYPE_TAG,
    ];

    for tag in rotate_only_attrs {
        let err = VersionGate::validate_attribute_mutation(ProtocolVersion::V2_0, tag)
            .expect_err("Cryptographic attribute mutation must be prohibited under KMIP 2.0");
        assert_eq!(
            err,
            VersionGateError::AttributeMutationProhibitedInVersion {
                tag,
                version: ProtocolVersion::V2_0,
            }
        );

        // Under 2.1, attribute mutation is permitted
        assert!(VersionGate::validate_attribute_mutation(ProtocolVersion::V2_1, tag).is_ok());
    }
}
