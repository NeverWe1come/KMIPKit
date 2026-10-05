use kmipkit_ttlv::ItemType;
use kmipkit_ttlv::codec::{DecodeErrorKind, decode};

const MAX_FIXTURE_BYTES: usize = 4096;

#[derive(Clone, Copy)]
enum Expected {
    Valid(ItemType),
    Rejected(DecodeErrorKind),
}

struct FixtureCase {
    name: &'static str,
    source: &'static str,
    attribution: &'static str,
    expected: Expected,
}

// Each vector's source comment is repeated here so the executable case and its
// checked-in fixture cannot silently diverge in traceability.
const FIXTURES: &[FixtureCase] = &[
    FixtureCase {
        name: "valid-integer-min.hex",
        source: include_str!("fixtures/valid-integer-min.hex"),
        attribution: "# Attribution: OASIS Key Management Interoperability Protocol Specification Version 2.1, Sections 10.1.1, 10.1.2, 10.1.3, 10.1.5, 11.23, and 11.56; KMIPKIT-0005-NR-001, NR-002, NR-004, NR-005, and NR-006.",
        expected: Expected::Valid(ItemType::Integer),
    },
    FixtureCase {
        name: "valid-long-integer-minus-one.hex",
        source: include_str!("fixtures/valid-long-integer-minus-one.hex"),
        attribution: "# Attribution: OASIS Key Management Interoperability Protocol Specification Version 2.1, Sections 10.1.1, 10.1.2, 10.1.3, 11.23, and 11.56; KMIPKIT-0005-NR-001, NR-002, NR-004, and NR-006.",
        expected: Expected::Valid(ItemType::LongInteger),
    },
    FixtureCase {
        name: "valid-big-integer-sign-extension.hex",
        source: include_str!("fixtures/valid-big-integer-sign-extension.hex"),
        attribution: "# Attribution: OASIS Key Management Interoperability Protocol Specification Version 2.1, Sections 10.1.1, 10.1.2, 10.1.3, 11.23, and 11.56; KMIPKIT-0005-NR-001, NR-002, NR-003, NR-004, and NR-006.",
        expected: Expected::Valid(ItemType::BigInteger),
    },
    FixtureCase {
        name: "valid-boolean-true.hex",
        source: include_str!("fixtures/valid-boolean-true.hex"),
        attribution: "# Attribution: OASIS Key Management Interoperability Protocol Specification Version 2.1, Sections 10.1.1, 10.1.2, 10.1.3, 11.23, and 11.56; KMIPKIT-0005-NR-001, NR-002, NR-004, and NR-006.",
        expected: Expected::Valid(ItemType::Boolean),
    },
    FixtureCase {
        name: "valid-date-time.hex",
        source: include_str!("fixtures/valid-date-time.hex"),
        attribution: "# Attribution: OASIS Key Management Interoperability Protocol Specification Version 2.1, Sections 10.1.1, 10.1.2, 10.1.3, 11.23, and 11.56; KMIPKIT-0005-NR-001, NR-002, NR-004, and NR-006.",
        expected: Expected::Valid(ItemType::DateTime),
    },
    FixtureCase {
        name: "valid-interval.hex",
        source: include_str!("fixtures/valid-interval.hex"),
        attribution: "# Attribution: OASIS Key Management Interoperability Protocol Specification Version 2.1, Sections 10.1.1, 10.1.2, 10.1.3, 10.1.5, 11.23, and 11.56; KMIPKIT-0005-NR-001, NR-002, NR-004, NR-005, and NR-006.",
        expected: Expected::Valid(ItemType::Interval),
    },
    FixtureCase {
        name: "valid-date-time-extended.hex",
        source: include_str!("fixtures/valid-date-time-extended.hex"),
        attribution: "# Attribution: OASIS Key Management Interoperability Protocol Specification Version 2.1, Sections 10.1.1, 10.1.2, 10.1.3, 11.23, and 11.56; KMIPKIT-0005-NR-001, NR-002, NR-004, and NR-006.",
        expected: Expected::Valid(ItemType::DateTimeExtended),
    },
    FixtureCase {
        name: "valid-extension-enumeration.hex",
        source: include_str!("fixtures/valid-extension-enumeration.hex"),
        attribution: "# Attribution: OASIS Key Management Interoperability Protocol Specification Version 2.1, Sections 10.1.1, 10.1.2, 10.1.3, 10.1.5, 11.23, and 11.56; KMIPKIT-0005-NR-001, NR-002, NR-004, NR-005, and NR-006.",
        expected: Expected::Valid(ItemType::Enumeration),
    },
    FixtureCase {
        name: "valid-text-string-padding.hex",
        source: include_str!("fixtures/valid-text-string-padding.hex"),
        attribution: "# Attribution: OASIS Key Management Interoperability Protocol Specification Version 2.1, Sections 10.1.1, 10.1.2, 10.1.3, 10.1.5, 11.23, and 11.56; KMIPKIT-0005-NR-001, NR-002, NR-004, NR-005, and NR-006.",
        expected: Expected::Valid(ItemType::TextString),
    },
    FixtureCase {
        name: "valid-byte-string-padding.hex",
        source: include_str!("fixtures/valid-byte-string-padding.hex"),
        attribution: "# Attribution: OASIS Key Management Interoperability Protocol Specification Version 2.1, Sections 10.1.1, 10.1.2, 10.1.3, 10.1.5, 11.23, and 11.56; KMIPKIT-0005-NR-001, NR-002, NR-004, NR-005, and NR-006.",
        expected: Expected::Valid(ItemType::ByteString),
    },
    FixtureCase {
        name: "valid-structure-child.hex",
        source: include_str!("fixtures/valid-structure-child.hex"),
        attribution: "# Attribution: OASIS Key Management Interoperability Protocol Specification Version 2.1, Sections 10.1.1, 10.1.2, 10.1.3, 10.1.5, 11.23, and 11.56; KMIPKIT-0005-NR-001, NR-002, NR-004, NR-005, and NR-006. KMIPKit generic child-order behavior is KMIPKIT-0005-FR-005; this does not validate a Structure schema.",
        expected: Expected::Valid(ItemType::Structure),
    },
    FixtureCase {
        name: "malformed-truncated-header.hex",
        source: include_str!("fixtures/malformed-truncated-header.hex"),
        attribution: "# Attribution: OASIS Key Management Interoperability Protocol Specification Version 2.1, Sections 10.1.1, 10.1.2, and 10.1.3; KMIPKIT-0005-NR-001, NR-002, and NR-004.",
        expected: Expected::Rejected(DecodeErrorKind::TruncatedHeader),
    },
    FixtureCase {
        name: "malformed-truncated-value.hex",
        source: include_str!("fixtures/malformed-truncated-value.hex"),
        attribution: "# Attribution: OASIS Key Management Interoperability Protocol Specification Version 2.1, Sections 10.1.2 and 10.1.3; KMIPKIT-0005-NR-002 and NR-004.",
        expected: Expected::Rejected(DecodeErrorKind::TruncatedValue),
    },
    FixtureCase {
        name: "malformed-fixed-width-length.hex",
        source: include_str!("fixtures/malformed-fixed-width-length.hex"),
        attribution: "# Attribution: OASIS Key Management Interoperability Protocol Specification Version 2.1, Sections 10.1.2 and 10.1.3; KMIPKIT-0005-NR-002 and NR-004.",
        expected: Expected::Rejected(DecodeErrorKind::InvalidItemLength),
    },
    FixtureCase {
        name: "malformed-boolean-value.hex",
        source: include_str!("fixtures/malformed-boolean-value.hex"),
        attribution: "# Attribution: OASIS Key Management Interoperability Protocol Specification Version 2.1, Section 10.1.2; KMIPKIT-0005-NR-002.",
        expected: Expected::Rejected(DecodeErrorKind::InvalidBoolean),
    },
    FixtureCase {
        name: "malformed-big-integer-length.hex",
        source: include_str!("fixtures/malformed-big-integer-length.hex"),
        attribution: "# Attribution: OASIS Key Management Interoperability Protocol Specification Version 2.1, Sections 10.1.2 and 10.1.3; KMIPKIT-0005-NR-003 and NR-004.",
        expected: Expected::Rejected(DecodeErrorKind::InvalidItemLength),
    },
    FixtureCase {
        name: "malformed-invalid-utf8.hex",
        source: include_str!("fixtures/malformed-invalid-utf8.hex"),
        attribution: "# Attribution: OASIS Key Management Interoperability Protocol Specification Version 2.1, Section 10.1.2; KMIPKIT-0005-NR-002.",
        expected: Expected::Rejected(DecodeErrorKind::InvalidUtf8),
    },
    FixtureCase {
        name: "malformed-missing-padding.hex",
        source: include_str!("fixtures/malformed-missing-padding.hex"),
        attribution: "# Attribution: OASIS Key Management Interoperability Protocol Specification Version 2.1, Section 10.1.5; KMIPKIT-0005-NR-005.",
        expected: Expected::Rejected(DecodeErrorKind::InvalidPaddingExtent),
    },
    FixtureCase {
        name: "malformed-structure-boundary.hex",
        source: include_str!("fixtures/malformed-structure-boundary.hex"),
        attribution: "# Attribution: OASIS Key Management Interoperability Protocol Specification Version 2.1, Sections 10.1.2, 10.1.3, and 10.1.5; KMIPKIT-0005-NR-002, NR-004, and NR-005.",
        expected: Expected::Rejected(DecodeErrorKind::StructureBoundary),
    },
    FixtureCase {
        name: "project-empty-big-integer.hex",
        source: include_str!("fixtures/project-empty-big-integer.hex"),
        attribution: "# Attribution: KMIPKit requirement KMIPKIT-0005-FR-006 rejects an empty Big Integer; the OASIS Key Management Interoperability Protocol Specification Version 2.1, Section 10.1.2, does not define a non-empty minimum.",
        expected: Expected::Rejected(DecodeErrorKind::EmptyBigInteger),
    },
    FixtureCase {
        name: "project-trailing-bytes.hex",
        source: include_str!("fixtures/project-trailing-bytes.hex"),
        attribution: "# Attribution: KMIPKit requirement KMIPKIT-0005-FR-004 requires exactly one complete item per decode call.",
        expected: Expected::Rejected(DecodeErrorKind::TrailingBytes),
    },
    FixtureCase {
        name: "project-reserved-tag.hex",
        source: include_str!("fixtures/project-reserved-tag.hex"),
        attribution: "# Attribution: KMIPKit requirement KMIPKIT-0005-FR-010 and accepted ADR-0011 reject received Reserved Tags; OASIS Key Management Interoperability Protocol Specification Version 2.1, Chapter 11 and Section 11.56, provide the reserved-tag rule and allocation context (KMIPKIT-0005-NR-006).",
        expected: Expected::Rejected(DecodeErrorKind::ReservedTag),
    },
];

#[test]
fn attributed_fixtures_match_the_public_decoder_contract() {
    for fixture in FIXTURES {
        assert!(
            fixture
                .source
                .lines()
                .any(|line| line == fixture.attribution),
            "{} is missing its inline source/requirement attribution",
            fixture.name
        );
        let bytes = decode_hex_fixture(fixture.name, fixture.source);

        match fixture.expected {
            Expected::Valid(expected_type) => {
                let item = decode(&bytes).unwrap_or_else(|error| {
                    panic!(
                        "{} should decode as {expected_type:?}, got {error}",
                        fixture.name
                    )
                });
                assert_eq!(item.item_type(), expected_type, "{}", fixture.name);
            }
            Expected::Rejected(expected_kind) => {
                let error = decode(&bytes).expect_err("malformed/project fixture is rejected");
                assert_eq!(error.kind(), expected_kind, "{}", fixture.name);
            }
        }
    }
}

fn decode_hex_fixture(name: &str, source: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    for line in source.lines() {
        let data = line.split_once('#').map_or(line, |(data, _)| data);
        for octet in data.split_ascii_whitespace() {
            assert_eq!(octet.len(), 2, "{name} contains a non-byte token {octet:?}");
            let value = u8::from_str_radix(octet, 16)
                .unwrap_or_else(|error| panic!("{name} contains invalid hex: {error}"));
            assert!(
                bytes.len() < MAX_FIXTURE_BYTES,
                "{name} exceeds fixture bound"
            );
            bytes.push(value);
        }
    }
    assert!(!bytes.is_empty(), "{name} must contain a wire vector");
    bytes
}
