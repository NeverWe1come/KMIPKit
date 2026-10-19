use kmipkit_test_support::oasis_crypto_fixtures::{
    OasisCryptoFixtureError, OasisProfileFixture, OasisProfileFixtureBindings,
};
use kmipkit_ttlv::ValueView;

const RESPONSE_HEADER_TAG: u32 = 0x0042_007A;
const TIME_STAMP_TAG: u32 = 0x0042_0092;

fn parse_first_profile_timestamp(value: &str) -> Result<i64, OasisCryptoFixtureError> {
    let baseline = include_str!("../../../specification/oasis/kmip-2.1/fixtures/BL-M-1-21.xml");
    let xml = baseline.replacen("$NOW", value, 1);
    let fixture =
        OasisProfileFixture::from_xml("BL-M-1-21", &xml, OasisProfileFixtureBindings::default())?;
    let response = fixture
        .exchanges()
        .first()
        .expect("the pinned baseline fixture has a first exchange")
        .response_message();
    let response_view = response.view();
    let response_header = response_view
        .children()
        .iter()
        .find(|item| item.tag().raw() == RESPONSE_HEADER_TAG)
        .expect("the first response has a Response Header");
    let time_stamp = response_header.with_value(|value| {
        let ValueView::Structure(header) = value else {
            return None;
        };
        let item = header
            .children()
            .iter()
            .find(|item| item.tag().raw() == TIME_STAMP_TAG)?;
        item.with_value(|value| match value {
            ValueView::DateTime(seconds) => Some(*seconds),
            _ => None,
        })
    });
    Ok(time_stamp.expect("the first response has a DateTime Time Stamp"))
}

#[test]
// This oracle table intentionally enumerates every pinned official profile fixture.
#[allow(clippy::too_many_lines)]
fn parses_all_pinned_profile_fixtures_in_exact_source_order() {
    let cases: &[(&str, &str, &[u32])] = &[
        (
            "BL-M-1-21",
            include_str!("../../../specification/oasis/kmip-2.1/fixtures/BL-M-1-21.xml"),
            &[0x34, 0x03, 0x08, 0x0a, 0x14, 0x2c, 0x34],
        ),
        (
            "BL-M-2-21",
            include_str!("../../../specification/oasis/kmip-2.1/fixtures/BL-M-2-21.xml"),
            &[0x34, 0x03, 0x12, 0x09, 0x09, 0x0b, 0x34],
        ),
        (
            "BL-M-3-21",
            include_str!("../../../specification/oasis/kmip-2.1/fixtures/BL-M-3-21.xml"),
            &[0x34, 0x03, 0x0a, 0x0b, 0x09, 0x09, 0x34],
        ),
        (
            "BL-M-4-21",
            include_str!("../../../specification/oasis/kmip-2.1/fixtures/BL-M-4-21.xml"),
            &[0x34, 0x03, 0x08, 0x0a, 0x0b, 0x0a, 0x34],
        ),
        (
            "BL-M-5-21",
            include_str!("../../../specification/oasis/kmip-2.1/fixtures/BL-M-5-21.xml"),
            &[0x34, 0x03, 0x0a, 0x0d, 0x0d, 0x0b, 0x34],
        ),
        (
            "BL-M-6-21",
            include_str!("../../../specification/oasis/kmip-2.1/fixtures/BL-M-6-21.xml"),
            &[0x34, 0x03, 0x0a, 0x0b, 0x0b, 0x34],
        ),
        (
            "BL-M-7-21",
            include_str!("../../../specification/oasis/kmip-2.1/fixtures/BL-M-7-21.xml"),
            &[0x34, 0x03, 0x0e, 0x13, 0x0b, 0x34],
        ),
        (
            "BL-M-8-21",
            include_str!("../../../specification/oasis/kmip-2.1/fixtures/BL-M-8-21.xml"),
            &[0x34, 0x03, 0x0d, 0x14, 0x0b, 0x0a, 0x34],
        ),
        (
            "BL-M-9-21",
            include_str!("../../../specification/oasis/kmip-2.1/fixtures/BL-M-9-21.xml"),
            &[0x34, 0x03, 0x14, 0x0b, 0x34],
        ),
        (
            "BL-M-10-21",
            include_str!("../../../specification/oasis/kmip-2.1/fixtures/BL-M-10-21.xml"),
            &[0x34, 0x03, 0x0b, 0x0e, 0x34],
        ),
        (
            "BL-M-11-21",
            include_str!("../../../specification/oasis/kmip-2.1/fixtures/BL-M-11-21.xml"),
            &[0x34, 0x03, 0x13, 0x0b, 0x34],
        ),
        (
            "BL-M-12-21",
            include_str!("../../../specification/oasis/kmip-2.1/fixtures/BL-M-12-21.xml"),
            &[0x34, 0x03, 0x0a, 0x0b, 0x34],
        ),
        (
            "BL-M-13-21",
            include_str!("../../../specification/oasis/kmip-2.1/fixtures/BL-M-13-21.xml"),
            &[0x34, 0x03, 0x0b, 0x0a, 0x0b, 0x34],
        ),
        (
            "SKLC-M-1-21",
            include_str!("../../../specification/oasis/kmip-2.1/fixtures/SKLC-M-1-21.xml"),
            &[0x01, 0x0b, 0x14],
        ),
        (
            "SKLC-M-2-21",
            include_str!("../../../specification/oasis/kmip-2.1/fixtures/SKLC-M-2-21.xml"),
            &[0x01, 0x0b, 0x12, 0x0b, 0x14, 0x13, 0x0b, 0x14],
        ),
        (
            "SKLC-M-3-21",
            include_str!("../../../specification/oasis/kmip-2.1/fixtures/SKLC-M-3-21.xml"),
            &[0x01, 0x0b, 0x12, 0x0b, 0x0e, 0x13, 0x0b, 0x14],
        ),
    ];

    for (case_id, xml, expected_operations) in cases {
        let fixture =
            OasisProfileFixture::from_xml(case_id, xml, OasisProfileFixtureBindings::default())
                .unwrap_or_else(|error| panic!("{case_id} failed: {error}"));
        assert_eq!(fixture.case_id(), *case_id);
        assert_eq!(
            fixture
                .exchanges()
                .iter()
                .flat_map(|exchange| exchange.operations().iter().copied())
                .collect::<Vec<_>>(),
            *expected_operations,
            "{case_id} operation order"
        );
        assert_eq!(
            fixture
                .exchanges()
                .iter()
                .map(kmipkit_test_support::oasis_crypto_fixtures::OasisProfileExchange::step)
                .collect::<Vec<_>>(),
            (0..fixture.exchanges().len()).collect::<Vec<_>>(),
            "{case_id} source step indexes"
        );
    }
}

#[test]
fn rejects_unknown_symbols_operations_malformed_pairs_and_reordered_steps() {
    let baseline = include_str!("../../../specification/oasis/kmip-2.1/fixtures/BL-M-1-21.xml");
    let unknown_symbol = baseline.replacen("$NOW", "$NOW_2", 1);
    assert_eq!(
        OasisProfileFixture::from_xml(
            "BL-M-1-21",
            &unknown_symbol,
            OasisProfileFixtureBindings::default()
        )
        .err(),
        Some(OasisCryptoFixtureError::UnknownSymbol)
    );

    let unknown_operation = baseline.replacen("value=\"Interop\"", "value=\"NoSuchOperation\"", 1);
    assert!(
        OasisProfileFixture::from_xml(
            "BL-M-1-21",
            &unknown_operation,
            OasisProfileFixtureBindings::default()
        )
        .is_err()
    );

    let malformed_pair = baseline.replacen("<ResponseMessage>", "<RequestMessage>", 1);
    assert!(
        OasisProfileFixture::from_xml(
            "BL-M-1-21",
            &malformed_pair,
            OasisProfileFixtureBindings::default()
        )
        .is_err()
    );

    let sklc = include_str!("../../../specification/oasis/kmip-2.1/fixtures/SKLC-M-1-21.xml");
    let reordered = sklc.replacen("step=1", "step=2", 1);
    assert_eq!(
        OasisProfileFixture::from_xml(
            "SKLC-M-1-21",
            &reordered,
            OasisProfileFixtureBindings::default()
        )
        .err(),
        Some(OasisCryptoFixtureError::ReorderedSteps)
    );

    let dated = include_str!("../../../specification/oasis/kmip-2.1/fixtures/BL-M-3-21.xml");
    let malformed_date = dated.replacen("2000-02-01T", "2000x02-01T", 1);
    assert_eq!(
        OasisProfileFixture::from_xml(
            "BL-M-3-21",
            &malformed_date,
            OasisProfileFixtureBindings::default()
        )
        .err(),
        Some(OasisCryptoFixtureError::InvalidValue)
    );
}

#[test]
fn profile_fixture_rejects_invalid_document_shapes() {
    let baseline = include_str!("../../../specification/oasis/kmip-2.1/fixtures/BL-M-1-21.xml");
    let wrong_root =
        baseline
            .replacen("<KMIP>", "<NotKmip>", 1)
            .replacen("</KMIP>", "</NotKmip>", 1);
    let one_message = "<KMIP><RequestMessage/></KMIP>";
    let wrong_message_order = "<KMIP><ResponseMessage/><RequestMessage/></KMIP>";
    let cases = [
        (
            "empty case identifier",
            "",
            baseline,
            OasisCryptoFixtureError::XmlLimitOrDtd,
        ),
        (
            "doctype declaration",
            "BL-M-1-21",
            "<!DOCTYPE KMIP><KMIP/>",
            OasisCryptoFixtureError::XmlLimitOrDtd,
        ),
        (
            "malformed XML",
            "BL-M-1-21",
            "<KMIP>",
            OasisCryptoFixtureError::MalformedXml,
        ),
        (
            "wrong root element",
            "BL-M-1-21",
            &wrong_root,
            OasisCryptoFixtureError::InvalidFixtureShape,
        ),
        (
            "empty fixture",
            "BL-M-1-21",
            "<KMIP/>",
            OasisCryptoFixtureError::UnpairedMessages,
        ),
        (
            "unpaired request",
            "BL-M-1-21",
            one_message,
            OasisCryptoFixtureError::UnpairedMessages,
        ),
        (
            "response before request",
            "BL-M-1-21",
            wrong_message_order,
            OasisCryptoFixtureError::InvalidFixtureShape,
        ),
    ];

    for (description, case_id, xml, expected_error) in cases {
        assert_profile_fixture_error(case_id, xml, expected_error, description);
    }
}

#[test]
fn profile_fixture_rejects_invalid_batch_shapes() {
    let baseline = include_str!("../../../specification/oasis/kmip-2.1/fixtures/BL-M-1-21.xml");
    let wrong_batch_header_order =
        baseline.replacen("<RequestHeader>", "<BatchItem/><RequestHeader>", 1);
    let zero_batch_count = baseline.replacen(
        "<BatchCount type=\"Integer\" value=\"1\"/>",
        "<BatchCount type=\"Integer\" value=\"0\"/>",
        1,
    );
    let mismatched_batch_count = baseline.replacen(
        "<BatchCount type=\"Integer\" value=\"1\"/>",
        "<BatchCount type=\"Integer\" value=\"2\"/>",
        1,
    );
    let malformed_batch_shape = baseline
        .replacen("<RequestPayload>", "<OtherPayload>", 1)
        .replacen("</RequestPayload>", "</OtherPayload>", 1);
    let mismatched_operations = baseline.replacen("value=\"Interop\"", "value=\"Register\"", 1);
    let operation = "<Operation type=\"Enumeration\" value=\"Interop\"/>";
    let batch_ids = baseline.replacen(
        operation,
        &format!("{operation}\n    <UniqueBatchItemID type=\"ByteString\" value=\"request-id\"/>"),
        2,
    );
    let mismatched_batch_ids =
        batch_ids.replacen("value=\"request-id\"", "value=\"response-id\"", 1);
    let cases = [
        (
            "batch item before header",
            "BL-M-1-21",
            &wrong_batch_header_order,
            OasisCryptoFixtureError::UnsupportedFixtureConstruct,
        ),
        (
            "zero batch count",
            "BL-M-1-21",
            &zero_batch_count,
            OasisCryptoFixtureError::UnsupportedFixtureConstruct,
        ),
        (
            "batch count mismatch",
            "BL-M-1-21",
            &mismatched_batch_count,
            OasisCryptoFixtureError::UnsupportedFixtureConstruct,
        ),
        (
            "unsupported batch item shape",
            "BL-M-1-21",
            &malformed_batch_shape,
            OasisCryptoFixtureError::UnsupportedFixtureConstruct,
        ),
        (
            "request and response operations differ",
            "BL-M-1-21",
            &mismatched_operations,
            OasisCryptoFixtureError::MismatchedOperations,
        ),
        (
            "request and response batch identifiers differ",
            "BL-M-1-21",
            &mismatched_batch_ids,
            OasisCryptoFixtureError::MismatchedBatchItemIdentifiers,
        ),
    ];

    for (description, case_id, xml, expected_error) in cases {
        assert_profile_fixture_error(case_id, xml, expected_error, description);
    }
}

fn assert_profile_fixture_error(
    case_id: &str,
    xml: &str,
    expected_error: OasisCryptoFixtureError,
    description: &str,
) {
    assert_eq!(
        OasisProfileFixture::from_xml(case_id, xml, OasisProfileFixtureBindings::default()).err(),
        Some(expected_error),
        "{description}"
    );
}

#[test]
fn profile_fixture_errors_have_stable_payload_free_display() {
    for (error, expected) in [
        (
            OasisCryptoFixtureError::MalformedXml,
            "OASIS fixture XML is malformed",
        ),
        (
            OasisCryptoFixtureError::XmlLimitOrDtd,
            "OASIS fixture XML exceeds limits or contains a DTD",
        ),
        (
            OasisCryptoFixtureError::XmlNodeLimit,
            "OASIS fixture XML exceeds the configured node limit",
        ),
        (
            OasisCryptoFixtureError::InvalidFixtureShape,
            "OASIS fixture message shape is invalid",
        ),
        (
            OasisCryptoFixtureError::UnpairedMessages,
            "OASIS fixture request and response messages are unpaired",
        ),
        (
            OasisCryptoFixtureError::MismatchedOperations,
            "OASIS fixture request and response operations differ",
        ),
        (
            OasisCryptoFixtureError::MismatchedBatchItemIdentifiers,
            "OASIS profile fixture request and response batch IDs differ",
        ),
        (
            OasisCryptoFixtureError::DuplicateSequence,
            "OASIS fixture contains a duplicate operation sequence",
        ),
        (
            OasisCryptoFixtureError::UnknownSymbol,
            "OASIS fixture selected message contains an unsupported symbol",
        ),
        (
            OasisCryptoFixtureError::InvalidTag,
            "OASIS fixture selected message contains an invalid tag",
        ),
        (
            OasisCryptoFixtureError::UnsupportedItemType,
            "OASIS fixture selected message contains an unsupported TTLV Item Type",
        ),
        (
            OasisCryptoFixtureError::InvalidValue,
            "OASIS fixture selected message contains an invalid TTLV value",
        ),
        (
            OasisCryptoFixtureError::TtlvLimit,
            "OASIS fixture selected message exceeds TTLV model limits",
        ),
        (
            OasisCryptoFixtureError::InvalidCatalog,
            "KMIPKit normative catalog cannot resolve fixture values",
        ),
        (
            OasisCryptoFixtureError::UnsupportedOperation,
            "OASIS profile fixture operation is unsupported",
        ),
        (
            OasisCryptoFixtureError::UnsupportedFixtureConstruct,
            "OASIS profile fixture construct is unsupported",
        ),
        (
            OasisCryptoFixtureError::ReorderedSteps,
            "OASIS profile fixture source steps are reordered",
        ),
    ] {
        assert_eq!(error.to_string(), expected);
    }
}

#[test]
fn profile_fixture_enforces_xml_node_limit_before_parsing_messages() {
    const XML_NODES_ABOVE_LIMIT: usize = 250_001;
    let baseline = include_str!("../../../specification/oasis/kmip-2.1/fixtures/BL-M-1-21.xml");
    let comments = "<!--x-->".repeat(XML_NODES_ABOVE_LIMIT);
    let xml = baseline.replacen("<KMIP>", &format!("<KMIP>{comments}"), 1);

    let error =
        OasisProfileFixture::from_xml("BL-M-1-21", &xml, OasisProfileFixtureBindings::default())
            .expect_err("the profile XML node limit is enforced before message parsing");
    assert_eq!(error, OasisCryptoFixtureError::XmlNodeLimit);
    assert_eq!(
        error.to_string(),
        "OASIS fixture XML exceeds the configured node limit"
    );
}

#[test]
fn profile_fixture_normalizes_rfc3339_timestamps_to_unix_seconds() {
    // KMIP Specification v2.1 §9.20 and Table 425 identify Time Stamp as a
    // DateTime field. These RFC 3339 strings exercise this fixture adapter's
    // conversion only; they do not assign a date-time syntax to OASIS XML.
    for (literal, expected_seconds) in [
        ("2024-02-29T01:30:00+01:30", 1_709_164_800),
        ("1969-12-31T23:59:59Z", -1),
        ("1970-01-01T00:30:00-01:00", 5_400),
        ("2000-04-30T12:00:00Z", 957_096_000),
        ("1700000000", 1_700_000_000),
    ] {
        assert_eq!(
            parse_first_profile_timestamp(literal)
                .expect("a valid calendar date and timezone are accepted"),
            expected_seconds,
            "{literal}"
        );
    }
}

#[test]
fn profile_fixture_rejects_invalid_rfc3339_timestamps_without_echoing_values() {
    for literal in [
        "2024/02-29T01:30:00Z",
        "20x4-02-29T01:30:00Z",
        "2024-0x-29T01:30:00Z",
        "2024-13-01T01:30:00Z",
        "1900-02-29T01:30:00Z",
        "2024-04-31T01:30:00Z",
        "2024-02-29T24:30:00Z",
        "2024-02-29T01:30:00+24:00",
        "2024-02-29T01:30:00+01:60",
        "2024-02-29T01:30:00+01-30",
    ] {
        let error = parse_first_profile_timestamp(literal)
            .expect_err("invalid date or timezone fields are rejected");
        assert_eq!(error, OasisCryptoFixtureError::InvalidValue, "{literal}");
        assert!(
            !error.to_string().contains(literal),
            "date-time diagnostics must not echo fixture values"
        );
    }
}
