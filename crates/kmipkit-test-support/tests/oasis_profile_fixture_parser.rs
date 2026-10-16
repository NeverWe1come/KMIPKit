use kmipkit_test_support::oasis_crypto_fixtures::{
    OasisCryptoFixtureError, OasisProfileFixture, OasisProfileFixtureBindings,
};

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
