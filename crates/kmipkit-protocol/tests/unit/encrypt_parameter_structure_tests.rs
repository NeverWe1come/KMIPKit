//! Encrypt and Decrypt Cryptographic Parameters Structure member validation
//! from OASIS KMIP v2.1 §4.16, Table 59, with allocated tag values from Table
//! 487. Each case supplies only the selected member, so §4.16 member-presence
//! rules do not independently reject the payload. These are source-derived
//! tests, not claims that an official OASIS Test Case passed.
//!
//! Traceability: `KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-BLOCK-CIPHER-MODE`,
//! `KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-COUNTER-LENGTH`,
//! `KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-CRYPTOGRAPHIC-ALGORITHM`,
//! `KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-DIGITAL-SIGNATURE-ALGORITHM`,
//! `KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-FIXED-FIELD-LENGTH`,
//! `KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-HASHING-ALGORITHM`,
//! `KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-INITIAL-COUNTER-VALUE`,
//! `KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-INVOCATION-FIELD-LENGTH`,
//! `KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-IV-LENGTH`,
//! `KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-KEY-ROLE-TYPE`,
//! `KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-MASK-GENERATOR`,
//! `KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-MASK-GENERATOR-HASHING-ALGORITHM`,
//! `KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-P-SOURCE`,
//! `KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-PADDING-METHOD`,
//! `KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-RANDOM-IV`,
//! `KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-SALT-LENGTH`,
//! `KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-TAG-LENGTH`, and
//! `KMIPKIT-ELEM-STRUCTURE-MEMBER-4-16-TRAILER-FIELD`.

use crate::decrypt::validate_request_payload as validate_decrypt_request_payload;
use crate::encrypt::validate_request_payload;
use crate::operation_test_support::item;
use kmipkit_ttlv::{Structure, Value};

const CRYPTOGRAPHIC_PARAMETERS: u32 = 0x0042_002B;

#[derive(Clone, Copy, Debug)]
enum TtlvItemType {
    Enumeration,
    Integer,
    Boolean,
    ByteString,
}

impl TtlvItemType {
    fn well_typed_value(self) -> Value {
        match self {
            Self::Enumeration => Value::enumeration(u32::MAX),
            Self::Integer => Value::integer(1),
            Self::Boolean => Value::boolean(true),
            Self::ByteString => Value::byte_string(vec![0xA5]),
        }
    }

    fn wrong_typed_value(self) -> Value {
        match self {
            Self::Enumeration | Self::ByteString => Value::integer(1),
            Self::Integer | Self::Boolean => Value::enumeration(u32::MAX),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct ParameterMember {
    name: &'static str,
    tag: u32,
    item_type: TtlvItemType,
}

const TABLE_59_MEMBERS: [ParameterMember; 18] = [
    ParameterMember {
        name: "Block Cipher Mode",
        tag: 0x0042_0011,
        item_type: TtlvItemType::Enumeration,
    },
    ParameterMember {
        name: "Cryptographic Algorithm",
        tag: 0x0042_0028,
        item_type: TtlvItemType::Enumeration,
    },
    ParameterMember {
        name: "Hashing Algorithm",
        tag: 0x0042_0038,
        item_type: TtlvItemType::Enumeration,
    },
    ParameterMember {
        name: "Padding Method",
        tag: 0x0042_005F,
        item_type: TtlvItemType::Enumeration,
    },
    ParameterMember {
        name: "Key Role Type",
        tag: 0x0042_0083,
        item_type: TtlvItemType::Enumeration,
    },
    ParameterMember {
        name: "Digital Signature Algorithm",
        tag: 0x0042_00AE,
        item_type: TtlvItemType::Enumeration,
    },
    ParameterMember {
        name: "Random IV",
        tag: 0x0042_00C5,
        item_type: TtlvItemType::Boolean,
    },
    ParameterMember {
        name: "IV Length",
        tag: 0x0042_00CD,
        item_type: TtlvItemType::Integer,
    },
    ParameterMember {
        name: "Tag Length",
        tag: 0x0042_00CE,
        item_type: TtlvItemType::Integer,
    },
    ParameterMember {
        name: "Fixed Field Length",
        tag: 0x0042_00CF,
        item_type: TtlvItemType::Integer,
    },
    ParameterMember {
        name: "Counter Length",
        tag: 0x0042_00D0,
        item_type: TtlvItemType::Integer,
    },
    ParameterMember {
        name: "Initial Counter Value",
        tag: 0x0042_00D1,
        item_type: TtlvItemType::Integer,
    },
    ParameterMember {
        name: "Invocation Field Length",
        tag: 0x0042_00D2,
        item_type: TtlvItemType::Integer,
    },
    ParameterMember {
        name: "Salt Length",
        tag: 0x0042_0100,
        item_type: TtlvItemType::Integer,
    },
    ParameterMember {
        name: "Mask Generator",
        tag: 0x0042_0101,
        item_type: TtlvItemType::Enumeration,
    },
    ParameterMember {
        name: "Mask Generator Hashing Algorithm",
        tag: 0x0042_0102,
        item_type: TtlvItemType::Enumeration,
    },
    ParameterMember {
        name: "P Source",
        tag: 0x0042_0103,
        item_type: TtlvItemType::ByteString,
    },
    ParameterMember {
        name: "Trailer Field",
        tag: 0x0042_0104,
        item_type: TtlvItemType::Integer,
    },
];

fn encrypt_payload(parameter_members: impl IntoIterator<Item = (u32, Value)>) -> Structure {
    let mut parameters = Structure::new();
    for (tag, value) in parameter_members {
        parameters
            .try_push(item(tag, value))
            .expect("one Table 59 member fits the Cryptographic Parameters structure");
    }

    let mut payload = Structure::new();
    payload
        .try_push(item(CRYPTOGRAPHIC_PARAMETERS, Value::structure(parameters)))
        .expect("Cryptographic Parameters fits the Encrypt payload");
    payload
}

fn decrypt_payload(parameter_members: impl IntoIterator<Item = (u32, Value)>) -> Structure {
    let mut parameters = Structure::new();
    for (tag, value) in parameter_members {
        parameters
            .try_push(item(tag, value))
            .expect("one Table 59 member fits the Cryptographic Parameters structure");
    }

    let mut payload = Structure::new();
    payload
        .try_push(item(CRYPTOGRAPHIC_PARAMETERS, Value::structure(parameters)))
        .expect("Cryptographic Parameters fits the Decrypt payload");
    payload
}

fn validate_member_cases(duplicate: bool) -> Vec<&'static str> {
    TABLE_59_MEMBERS
        .iter()
        .filter_map(|member| {
            let value = member.item_type.well_typed_value();
            let fields = if duplicate {
                vec![
                    (member.tag, value),
                    (member.tag, member.item_type.well_typed_value()),
                ]
            } else {
                vec![(member.tag, member.item_type.wrong_typed_value())]
            };
            validate_request_payload(&encrypt_payload(fields))
                .is_ok()
                .then_some(member.name)
        })
        .collect()
}

fn validate_decrypt_member_cases(duplicate: bool) -> Vec<&'static str> {
    TABLE_59_MEMBERS
        .iter()
        .filter_map(|member| {
            let value = member.item_type.well_typed_value();
            let fields = if duplicate {
                vec![
                    (member.tag, value),
                    (member.tag, member.item_type.well_typed_value()),
                ]
            } else {
                vec![(member.tag, member.item_type.wrong_typed_value())]
            };
            validate_decrypt_request_payload(&decrypt_payload(fields))
                .is_ok()
                .then_some(member.name)
        })
        .collect()
}

#[test]
fn encrypt_rejects_duplicate_members_for_every_table_59_parameter() {
    let accepted = validate_member_cases(true);
    assert!(
        accepted.is_empty(),
        "Encrypt accepted duplicate Table 59 singleton members: {accepted:?}"
    );
}

#[test]
fn encrypt_rejects_wrong_ttlv_item_type_for_every_table_59_parameter() {
    let accepted = validate_member_cases(false);
    assert!(
        accepted.is_empty(),
        "Encrypt accepted wrong TTLV Item Type for Table 59 members: {accepted:?}"
    );
}

#[test]
fn decrypt_rejects_duplicate_members_for_every_table_59_parameter() {
    let accepted = validate_decrypt_member_cases(true);
    assert!(
        accepted.is_empty(),
        "Decrypt accepted duplicate Table 59 singleton members: {accepted:?}"
    );
}

#[test]
fn decrypt_rejects_wrong_ttlv_item_type_for_every_table_59_parameter() {
    let accepted = validate_decrypt_member_cases(false);
    assert!(
        accepted.is_empty(),
        "Decrypt accepted wrong TTLV Item Type for Table 59 members: {accepted:?}"
    );
}
