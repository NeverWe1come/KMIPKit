use kmipkit_test_support::oasis_crypto_fixtures::{
    OasisProfileFixture, OasisProfileFixtureBindings,
};

#[test]
fn parses_all_source_ordered_exchanges_without_filtering() {
    let xml = include_str!("../../../specification/oasis/kmip-2.1/fixtures/BL-M-1-21.xml");
    let fixture =
        OasisProfileFixture::from_xml("BL-M-1-21", xml, OasisProfileFixtureBindings::default())
            .expect("pinned full profile fixture parses");

    assert_eq!(fixture.case_id(), "BL-M-1-21");
    assert_eq!(
        fixture
            .exchanges()
            .iter()
            .map(|exchange| exchange.operation())
            .collect::<Vec<_>>(),
        [0x34, 0x03, 0x18, 0x0a, 0x14, 0x2c, 0x34]
    );
    assert_eq!(
        fixture
            .exchanges()
            .iter()
            .map(|exchange| exchange.step())
            .collect::<Vec<_>>(),
        [0, 1, 2, 3, 4, 5, 6]
    );
}
