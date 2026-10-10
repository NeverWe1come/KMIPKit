//! Fixture-derived operation-item evidence from the OASIS KMIP Test Cases
//! v2.1 Committee Note 01, §§2.99–2.101 and their pinned XML work products.
//! Operation semantics follow OASIS KMIP Specification v2.1 §§6.1.11 and
//! 6.1.17, Tables 196–198 and 214–216; opaque Data uses §7.9, Tables 360–361.
//! Operation Enumeration values are defined by §11.36.
//! The adapter selects Encrypt/Decrypt item pairs from larger workflows; these
//! tests do not claim that any complete official Test Case passed.

use kmipkit_protocol::{
    DecryptResponse, EncryptResponse, ResponseBatchItemView, ResponseMessage, ResultStatus,
    SecretBytes, UniqueIdentifier,
};
use kmipkit_test_support::oasis_crypto_fixtures::{
    OasisCryptoFixture, OasisCryptoFixtureError, OasisCryptoOperation, OasisCryptoOperationPair,
};
use kmipkit_ttlv::{Item, Structure, ValueView};

const TC_ENC_1_21_ID: &str = "TC-STREAM-ENC-1-21";
const TC_ENC_2_21_ID: &str = "TC-STREAM-ENC-2-21";
const TC_ENCDEC_1_21_ID: &str = "TC-STREAM-ENCDEC-1-21";
const TIME_STAMP_TAG: u32 = 0x0042_0092;
const UNIQUE_IDENTIFIER_TAG: u32 = 0x0042_0094;
const CORRELATION_VALUE_TAG: u32 = 0x0042_00D6;
const DATA_TAG: u32 = 0x0042_00C2;
const IV_COUNTER_NONCE_TAG: u32 = 0x0042_003D;
const AUTHENTICATED_ENCRYPTION_TAG: u32 = 0x0042_00FF;
const ENCRYPT_OPERATION: u32 = 0x0000_001F;
const DECRYPT_OPERATION: u32 = 0x0000_0020;
const SUCCESS: u32 = 0;
const TEST_NOW: i64 = 1_700_000_000;
const TEST_UNIQUE_IDENTIFIER_0: &str = "kmipkit-test-unique-id-0";
const TEST_CORRELATION_VALUE: &[u8] = b"kmipkit-test-correlation-value-0";
// These test-only values make the three accepted OASIS symbols deterministic.
const TC_ENC_1_STEP_1_DATA: &[u8] = &[
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16,
];

const TC_ENC_1_21_XML: &str =
    include_str!("../../../specification/oasis/kmip-2.1/fixtures/TC-STREAM-ENC-1-21.xml");
const TC_ENC_2_21_XML: &str =
    include_str!("../../../specification/oasis/kmip-2.1/fixtures/TC-STREAM-ENC-2-21.xml");
const TC_ENCDEC_1_21_XML: &str =
    include_str!("../../../specification/oasis/kmip-2.1/fixtures/TC-STREAM-ENCDEC-1-21.xml");
const SEQUENCE_PAIRING_CASE_ID: &str = "TC-T015-PAIRING-SYNTHETIC";
const SEQUENCE_PAIRING_XML: &str = r#"
<KMIP>
  <RequestMessage>
    <RequestHeader>
      <ProtocolVersion>
        <ProtocolVersionMajor type="Integer" value="2"/>
        <ProtocolVersionMinor type="Integer" value="1"/>
      </ProtocolVersion>
      <ClientCorrelationValue type="TextString" value="TC-T015-PAIRING-SYNTHETIC step=0"/>
      <BatchCount type="Integer" value="1"/>
    </RequestHeader>
    <BatchItem>
      <Operation type="Enumeration" value="Encrypt"/>
      <RequestPayload>
        <UniqueIdentifier type="TextString" value="paired-key-0"/>
        <Data type="ByteString" value="01020304"/>
      </RequestPayload>
    </BatchItem>
  </RequestMessage>
  <ResponseMessage>
    <ResponseHeader>
      <ProtocolVersion>
        <ProtocolVersionMajor type="Integer" value="2"/>
        <ProtocolVersionMinor type="Integer" value="1"/>
      </ProtocolVersion>
      <TimeStamp type="DateTime" value="$NOW"/>
      <BatchCount type="Integer" value="1"/>
    </ResponseHeader>
    <BatchItem>
      <Operation type="Enumeration" value="Encrypt"/>
      <ResultStatus type="Enumeration" value="Success"/>
      <ResponsePayload>
        <UniqueIdentifier type="TextString" value="paired-key-0"/>
        <Data type="ByteString" value="a0a1a2a3"/>
      </ResponsePayload>
    </BatchItem>
  </ResponseMessage>
  <RequestMessage>
    <RequestHeader>
      <ProtocolVersion>
        <ProtocolVersionMajor type="Integer" value="2"/>
        <ProtocolVersionMinor type="Integer" value="1"/>
      </ProtocolVersion>
      <ClientCorrelationValue type="TextString" value="TC-T015-PAIRING-SYNTHETIC step=1"/>
      <BatchCount type="Integer" value="1"/>
    </RequestHeader>
    <BatchItem>
      <Operation type="Enumeration" value="Encrypt"/>
      <RequestPayload>
        <UniqueIdentifier type="TextString" value="paired-key-1"/>
        <Data type="ByteString" value="11121314"/>
      </RequestPayload>
    </BatchItem>
  </RequestMessage>
  <ResponseMessage>
    <ResponseHeader>
      <ProtocolVersion>
        <ProtocolVersionMajor type="Integer" value="2"/>
        <ProtocolVersionMinor type="Integer" value="1"/>
      </ProtocolVersion>
      <TimeStamp type="DateTime" value="$NOW"/>
      <BatchCount type="Integer" value="1"/>
    </ResponseHeader>
    <BatchItem>
      <Operation type="Enumeration" value="Encrypt"/>
      <ResultStatus type="Enumeration" value="Success"/>
      <ResponsePayload>
        <UniqueIdentifier type="TextString" value="paired-key-1"/>
        <Data type="ByteString" value="b0b1b2b3"/>
      </ResponsePayload>
    </BatchItem>
  </ResponseMessage>
</KMIP>
"#;

#[derive(Debug, Eq, PartialEq)]
enum ValueSnapshot {
    Structure(Vec<ItemSnapshot>),
    Integer(i32),
    LongInteger(i64),
    BigInteger(Vec<u8>),
    Enumeration(u32),
    Boolean(bool),
    TextString(String),
    ByteString(Vec<u8>),
    DateTime(i64),
    Interval(u32),
    DateTimeExtended(i64),
    Unsupported,
}

#[derive(Debug, Eq, PartialEq)]
struct ItemSnapshot {
    tag: u32,
    value: ValueSnapshot,
}

#[derive(Debug, Eq, PartialEq)]
struct PairSnapshot {
    case_id: String,
    source_sequence: usize,
    step_identity: String,
    operation: OasisCryptoOperation,
    request: Vec<ItemSnapshot>,
    response: Vec<ItemSnapshot>,
}

fn snapshot_fixture(fixture: &OasisCryptoFixture) -> Vec<PairSnapshot> {
    fixture
        .operation_pairs()
        .iter()
        .map(|pair| PairSnapshot {
            case_id: pair.case_id().to_owned(),
            source_sequence: pair.source_sequence(),
            step_identity: pair.step_identity().to_owned(),
            operation: pair.operation(),
            request: snapshot_structure(pair.request_message()),
            response: snapshot_structure(pair.response_message()),
        })
        .collect()
}

fn snapshot_structure(structure: &Structure) -> Vec<ItemSnapshot> {
    structure
        .view()
        .children()
        .iter()
        .map(snapshot_item)
        .collect()
}

fn snapshot_item(item: &Item) -> ItemSnapshot {
    ItemSnapshot {
        tag: item.tag().raw(),
        value: item.with_value(snapshot_value),
    }
}

fn snapshot_value(value: ValueView<'_>) -> ValueSnapshot {
    match value {
        ValueView::Structure(value) => {
            ValueSnapshot::Structure(value.children().iter().map(snapshot_item).collect())
        }
        ValueView::Integer(value) => ValueSnapshot::Integer(*value),
        ValueView::LongInteger(value) => ValueSnapshot::LongInteger(*value),
        ValueView::BigInteger(value) => ValueSnapshot::BigInteger(value.to_vec()),
        ValueView::Enumeration(value) => ValueSnapshot::Enumeration(*value),
        ValueView::Boolean(value) => ValueSnapshot::Boolean(*value),
        ValueView::TextString(value) => ValueSnapshot::TextString((*value).to_owned()),
        ValueView::ByteString(value) => ValueSnapshot::ByteString(value.to_vec()),
        ValueView::DateTime(value) => ValueSnapshot::DateTime(*value),
        ValueView::Interval(value) => ValueSnapshot::Interval(*value),
        ValueView::DateTimeExtended(value) => ValueSnapshot::DateTimeExtended(*value),
        _ => ValueSnapshot::Unsupported,
    }
}

const SYMBOL_TOKENS: [&str; 3] = ["$NOW", "$UNIQUE_IDENTIFIER_0", "$CORRELATION_VALUE"];

fn contains_symbol(value: &str) -> bool {
    SYMBOL_TOKENS.iter().any(|symbol| value.contains(symbol))
}

fn contains_symbol_bytes(value: &[u8]) -> bool {
    SYMBOL_TOKENS.iter().any(|symbol| {
        value
            .windows(symbol.len())
            .any(|candidate| candidate == symbol.as_bytes())
    })
}

fn has_unresolved_symbol(structure: &Structure) -> bool {
    structure.view().children().iter().any(|item| {
        item.with_value(|value| match value {
            ValueView::Structure(value) => has_unresolved_symbol_view(value.children()),
            ValueView::TextString(value) => contains_symbol(value),
            ValueView::ByteString(value) => contains_symbol_bytes(value),
            _ => false,
        })
    })
}

fn has_unresolved_symbol_view(items: &[Item]) -> bool {
    items.iter().any(|item| {
        item.with_value(|value| match value {
            ValueView::Structure(value) => has_unresolved_symbol_view(value.children()),
            ValueView::TextString(value) => contains_symbol(value),
            ValueView::ByteString(value) => contains_symbol_bytes(value),
            _ => false,
        })
    })
}

fn contains_value(structure: &Structure, tag: u32, expected: &ValueSnapshot) -> bool {
    contains_value_in_items(structure.view().children(), tag, expected)
}

fn contains_value_in_items(items: &[Item], tag: u32, expected: &ValueSnapshot) -> bool {
    items.iter().any(|item| {
        let matches_value =
            item.tag().raw() == tag && item.with_value(|value| &snapshot_value(value) == expected);
        let nested_match = item.with_value(|value| match value {
            ValueView::Structure(value) => contains_value_in_items(value.children(), tag, expected),
            _ => false,
        });
        matches_value || nested_match
    })
}

fn response_payload_snapshot(item: ResponseBatchItemView<'_>) -> Vec<ItemSnapshot> {
    item.with_response_payload(|payload| payload.children().iter().map(snapshot_item).collect())
        .expect("each selected fixture response has a response payload")
}

fn snapshot_field(items: &[ItemSnapshot], tag: u32) -> Option<&ItemSnapshot> {
    items.iter().find(|item| item.tag == tag)
}

fn assert_response_secret_matches_fixture(
    actual: Option<&SecretBytes>,
    payload: &[ItemSnapshot],
    tag: u32,
) {
    match (actual, snapshot_field(payload, tag)) {
        (None, None) => {}
        (
            Some(actual),
            Some(ItemSnapshot {
                value: ValueSnapshot::ByteString(expected),
                ..
            }),
        ) => actual.with_bytes(|actual| assert_eq!(actual, expected)),
        _ => panic!("typed response must retain the fixture's optional Byte String field"),
    }
}

fn assert_fixture_response_table_order(operation: OasisCryptoOperation, payload: &[ItemSnapshot]) {
    let allowed_order: &[u32] = match operation {
        OasisCryptoOperation::Encrypt => &[
            UNIQUE_IDENTIFIER_TAG,
            DATA_TAG,
            IV_COUNTER_NONCE_TAG,
            CORRELATION_VALUE_TAG,
            AUTHENTICATED_ENCRYPTION_TAG,
        ],
        OasisCryptoOperation::Decrypt => &[UNIQUE_IDENTIFIER_TAG, DATA_TAG, CORRELATION_VALUE_TAG],
    };
    let tags: Vec<_> = payload.iter().map(|item| item.tag).collect();
    assert_eq!(tags.first(), Some(&UNIQUE_IDENTIFIER_TAG));
    assert_eq!(
        tags.iter()
            .filter(|tag| **tag == UNIQUE_IDENTIFIER_TAG)
            .count(),
        1,
        "a successful fixture response has exactly one Unique Identifier"
    );
    let ranks: Vec<_> = tags
        .iter()
        .map(|tag| {
            allowed_order
                .iter()
                .position(|allowed| allowed == tag)
                .expect("the fixture response contains only fields defined by its success table")
        })
        .collect();
    assert!(
        ranks.windows(2).all(|pair| pair[0] < pair[1]),
        "fixture response fields retain their Table 215 or Table 197 order"
    );
}

fn assert_fixture_success_identifier(payload: &[ItemSnapshot]) {
    assert!(matches!(
        snapshot_field(payload, UNIQUE_IDENTIFIER_TAG),
        Some(ItemSnapshot {
            value: ValueSnapshot::TextString(value),
            ..
        }) if value == TEST_UNIQUE_IDENTIFIER_0
    ));
}

#[test]
fn fixture_adapter_accounts_for_each_in_scope_pair_by_source_sequence_and_step() {
    // §2.99 and §2.100 each contribute ten Encrypt items. The linked §2.100
    // XML contains no Decrypt item despite its HTML description. The linked
    // §2.101 XML contributes four Encrypt and four Decrypt items despite its
    // Encrypt-only HTML description; only the executable fixture items count.
    let cases = [
        (
            TC_ENC_1_21_ID,
            TC_ENC_1_21_XML,
            &[1_usize, 2, 3, 4, 5, 6, 7, 8, 9, 10][..],
            &[OasisCryptoOperation::Encrypt; 10][..],
        ),
        (
            TC_ENC_2_21_ID,
            TC_ENC_2_21_XML,
            &[1_usize, 2, 3, 4, 5, 6, 7, 8, 9, 10][..],
            &[OasisCryptoOperation::Encrypt; 10][..],
        ),
        (
            TC_ENCDEC_1_21_ID,
            TC_ENCDEC_1_21_XML,
            &[1_usize, 2, 3, 4, 5, 6, 7, 8][..],
            &[
                OasisCryptoOperation::Encrypt,
                OasisCryptoOperation::Decrypt,
                OasisCryptoOperation::Encrypt,
                OasisCryptoOperation::Encrypt,
                OasisCryptoOperation::Encrypt,
                OasisCryptoOperation::Decrypt,
                OasisCryptoOperation::Decrypt,
                OasisCryptoOperation::Decrypt,
            ][..],
        ),
    ];

    let mut total_pairs = 0;
    let mut encrypt_pairs = 0;
    let mut decrypt_pairs = 0;
    for (case_id, xml, expected_sequences, expected_operations) in cases {
        let fixture = OasisCryptoFixture::from_xml(case_id, xml)
            .expect("the pinned OASIS XML fixture has valid in-scope operation items");
        let pairs = fixture.operation_pairs();

        assert_eq!(pairs.len(), expected_sequences.len());
        for ((pair, expected_sequence), expected_operation) in pairs
            .iter()
            .zip(expected_sequences)
            .zip(expected_operations)
        {
            assert_eq!(pair.case_id(), case_id);
            assert_eq!(pair.source_sequence(), *expected_sequence);
            assert_eq!(pair.operation(), *expected_operation);
            assert_eq!(
                pair.step_identity(),
                format!("{case_id} step={expected_sequence}"),
                "the RequestHeader ClientCorrelationValue step identity is retained"
            );
            match pair.operation() {
                OasisCryptoOperation::Encrypt => encrypt_pairs += 1,
                OasisCryptoOperation::Decrypt => decrypt_pairs += 1,
            }
        }
        total_pairs += pairs.len();
    }

    assert_eq!(total_pairs, 28, "no in-scope fixture pair may be skipped");
    assert_eq!(encrypt_pairs, 24);
    assert_eq!(decrypt_pairs, 4);
}

#[test]
fn fixture_adapter_pairs_each_request_with_its_source_sequence_response() {
    let fixture = OasisCryptoFixture::from_xml(SEQUENCE_PAIRING_CASE_ID, SEQUENCE_PAIRING_XML)
        .expect("the synthetic fixture has two valid Encrypt exchanges");
    let pairs = fixture.operation_pairs();

    assert_eq!(pairs.len(), 2);
    let expected = [
        (
            0,
            "TC-T015-PAIRING-SYNTHETIC step=0",
            "paired-key-0",
            vec![0x01, 0x02, 0x03, 0x04],
            vec![0xA0, 0xA1, 0xA2, 0xA3],
        ),
        (
            1,
            "TC-T015-PAIRING-SYNTHETIC step=1",
            "paired-key-1",
            vec![0x11, 0x12, 0x13, 0x14],
            vec![0xB0, 0xB1, 0xB2, 0xB3],
        ),
    ];

    for (pair, (sequence, step_identity, unique_identifier, request_data, response_data)) in
        pairs.iter().zip(expected)
    {
        assert_eq!(pair.case_id(), SEQUENCE_PAIRING_CASE_ID);
        assert_eq!(pair.source_sequence(), sequence);
        assert_eq!(pair.step_identity(), step_identity);
        assert_eq!(pair.operation(), OasisCryptoOperation::Encrypt);
        let expected_identifier = ValueSnapshot::TextString(unique_identifier.to_owned());
        assert!(contains_value(
            pair.request_message(),
            UNIQUE_IDENTIFIER_TAG,
            &expected_identifier,
        ));
        assert!(contains_value(
            pair.response_message(),
            UNIQUE_IDENTIFIER_TAG,
            &expected_identifier,
        ));
        assert!(contains_value(
            pair.request_message(),
            DATA_TAG,
            &ValueSnapshot::ByteString(request_data),
        ));
        assert!(contains_value(
            pair.response_message(),
            DATA_TAG,
            &ValueSnapshot::ByteString(response_data),
        ));
    }
}

#[test]
fn fixture_adapter_resolves_selected_symbols_deterministically() {
    let first = OasisCryptoFixture::from_xml(TC_ENC_1_21_ID, TC_ENC_1_21_XML)
        .expect("the fixture's out-of-scope setup symbols are not inspected");
    let second = OasisCryptoFixture::from_xml(TC_ENC_1_21_ID, TC_ENC_1_21_XML)
        .expect("re-reading the fixture yields the same deterministic substitutions");

    assert_eq!(snapshot_fixture(&first), snapshot_fixture(&second));
    assert!(
        first.operation_pairs().iter().all(|pair| {
            !has_unresolved_symbol(pair.request_message())
                && !has_unresolved_symbol(pair.response_message())
        }),
        "$NOW, $UNIQUE_IDENTIFIER_0, and $CORRELATION_VALUE are resolved in selected messages"
    );

    let first_encrypt = first
        .operation_pairs()
        .iter()
        .find(|pair| pair.source_sequence() == 1)
        .expect("source sequence 1 is the first in-scope Encrypt request");
    assert!(contains_value(
        first_encrypt.response_message(),
        TIME_STAMP_TAG,
        &ValueSnapshot::DateTime(TEST_NOW),
    ));
    assert!(contains_value(
        first_encrypt.request_message(),
        UNIQUE_IDENTIFIER_TAG,
        &ValueSnapshot::TextString(TEST_UNIQUE_IDENTIFIER_0.to_owned()),
    ));
    let first_multipart_encrypt = first
        .operation_pairs()
        .iter()
        .find(|pair| pair.source_sequence() == 2)
        .expect("source sequence 2 is the first multipart Encrypt request");
    assert!(contains_value(
        first_multipart_encrypt.response_message(),
        CORRELATION_VALUE_TAG,
        &ValueSnapshot::ByteString(TEST_CORRELATION_VALUE.to_vec()),
    ));
    assert!(contains_value(
        first_encrypt.request_message(),
        DATA_TAG,
        &ValueSnapshot::ByteString(TC_ENC_1_STEP_1_DATA.to_vec()),
    ));
}

#[test]
fn fixture_adapter_ignores_unrecognized_symbols_in_out_of_scope_setup_messages() {
    // TC-STREAM-ENC-1-21 step 0 is a Create setup item containing $NOW-3600.
    // The selected Encrypt pairs still parse; setup and cleanup symbols are
    // outside this adapter's substitution contract.
    let fixture = OasisCryptoFixture::from_xml(TC_ENC_1_21_ID, TC_ENC_1_21_XML)
        .expect("the out-of-scope $NOW-3600 token is not rejected");

    assert_eq!(fixture.operation_pairs().len(), 10);
    assert!(
        fixture
            .operation_pairs()
            .iter()
            .all(|pair| pair.operation() == OasisCryptoOperation::Encrypt)
    );
}

#[test]
fn fixture_adapter_filters_unknown_setup_symbol_and_keeps_all_28_pairs() {
    assert!(TC_ENC_1_21_XML.contains("$NOW-3600"));
    let changed_setup_symbol =
        TC_ENC_1_21_XML.replace("$NOW-3600", "$UNKNOWN_OUT_OF_SCOPE_SETUP_SYMBOL");
    assert!(changed_setup_symbol.contains("$UNKNOWN_OUT_OF_SCOPE_SETUP_SYMBOL"));

    let fixtures = [
        OasisCryptoFixture::from_xml(TC_ENC_1_21_ID, &changed_setup_symbol)
            .expect("the unknown setup token is ignored after operation filtering"),
        OasisCryptoFixture::from_xml(TC_ENC_2_21_ID, TC_ENC_2_21_XML)
            .expect("the second pinned fixture parses"),
        OasisCryptoFixture::from_xml(TC_ENCDEC_1_21_ID, TC_ENCDEC_1_21_XML)
            .expect("the Encrypt/Decrypt pinned fixture parses"),
    ];
    let mut total_pairs = 0;
    let mut encrypt_pairs = 0;
    let mut decrypt_pairs = 0;
    for fixture in fixtures {
        for pair in fixture.operation_pairs() {
            total_pairs += 1;
            match pair.operation() {
                OasisCryptoOperation::Encrypt => encrypt_pairs += 1,
                OasisCryptoOperation::Decrypt => decrypt_pairs += 1,
            }
        }
    }

    assert_eq!(total_pairs, 28);
    assert_eq!(encrypt_pairs, 24);
    assert_eq!(decrypt_pairs, 4);
}

#[test]
fn fixture_adapter_rejects_unknown_symbols_in_selected_operation_messages() {
    assert!(TC_ENCDEC_1_21_XML.contains("$CORRELATION_VALUE"));
    let modified_xml =
        TC_ENCDEC_1_21_XML.replace("$CORRELATION_VALUE", "$UNKNOWN_SELECTED_OPERATION_SYMBOL");
    assert!(modified_xml.contains("$UNKNOWN_SELECTED_OPERATION_SYMBOL"));

    assert!(OasisCryptoFixture::from_xml(TC_ENCDEC_1_21_ID, &modified_xml).is_err());

    // A recognized token prefix does not make the unsupported relative-time
    // form valid inside an in-scope response header.
    let selected_relative_time = TC_ENC_1_21_XML.replace("$NOW", "$NOW-3600");
    assert!(OasisCryptoFixture::from_xml(TC_ENC_1_21_ID, &selected_relative_time).is_err());
}

#[test]
fn fixture_adapter_preserves_unknown_extension_tags_and_enumeration_values() {
    // This synthetic node uses the allocated vendor extension range to prove
    // that the adapter retains generic TTLV values without assigning meaning.
    let xml = SEQUENCE_PAIRING_XML.replacen(
        "<Data type=\"ByteString\" value=\"01020304\"/>",
        "<Data type=\"ByteString\" value=\"01020304\"/><VendorPrivateValue tag=\"0x54ABCD\" type=\"Enumeration\" value=\"0xDEADBEEF\"/>",
        1,
    );
    let fixture = OasisCryptoFixture::from_xml(SEQUENCE_PAIRING_CASE_ID, &xml)
        .expect("generic extension values remain representable in selected messages");
    let first_request = fixture
        .operation_pairs()
        .first()
        .expect("the synthetic fixture has an in-scope operation pair")
        .request_message();

    assert!(contains_value(
        first_request,
        0x0054_ABCD,
        &ValueSnapshot::Enumeration(0xDEAD_BEEF),
    ));
}

#[test]
fn fixture_adapter_rejects_malformed_xml_without_echoing_document_contents() {
    const SECRET_SENTINEL: &str = "KMIPKIT_XML_SECRET_SENTINEL_73";
    let malformed_xml = format!(
        "<KMIP><RequestMessage><Data type=\"TextString\" value=\"{SECRET_SENTINEL}\"</RequestMessage>"
    );

    let error = OasisCryptoFixture::from_xml("TC-T015-MALFORMED", &malformed_xml)
        .expect_err("an unclosed XML attribute is rejected");
    let diagnostic = format!("{error:?} {error}");

    assert!(
        !diagnostic.contains(SECRET_SENTINEL),
        "fixture diagnostics must not echo raw XML values"
    );
}

#[test]
fn fixture_adapter_rejects_doctypes_before_parsing_xml() {
    let xml = format!(
        "<!DOCTYPE KMIP [<!ENTITY external SYSTEM \"file:///sensitive/path\">]>{SEQUENCE_PAIRING_XML}"
    );

    assert_eq!(
        OasisCryptoFixture::from_xml(SEQUENCE_PAIRING_CASE_ID, &xml).unwrap_err(),
        OasisCryptoFixtureError::XmlLimitOrDtd
    );
}

#[test]
fn fixture_adapter_enforces_ttlv_depth_before_resolving_deep_items() {
    let mut deep_items =
        "<VendorPrivateValue tag=\"0x54ABCD\" type=\"Enumeration\" value=\"0xDEADBEEF\"/>"
            .to_owned();
    for _ in 0..62 {
        deep_items = format!("<VendorNested tag=\"0x54ABCD\">{deep_items}</VendorNested>");
    }
    let request_with_deep_items = SEQUENCE_PAIRING_XML.replacen(
        "</RequestPayload>",
        &format!("{deep_items}</RequestPayload>"),
        1,
    );

    assert_eq!(
        OasisCryptoFixture::from_xml(SEQUENCE_PAIRING_CASE_ID, &request_with_deep_items)
            .unwrap_err(),
        OasisCryptoFixtureError::TtlvLimit
    );
}

#[test]
fn fixture_derived_success_responses_preserve_all_24_encrypt_and_4_decrypt_items() {
    // OASIS KMIP Specification v2.1 §6.1.17 Table 215 and §6.1.11 Table 197
    // define the success payloads; §§7.4, 7.8, and 7.9 define the optional
    // Encrypt tag, multipart Correlation Value, and Byte String Data. These
    // are fixture-derived operation-item assertions from Test Cases v2.1 CN01
    // §§2.99–2.101, never complete official Test Case pass claims.
    //
    // Traceability: `KMIPKIT-REQ-SPEC-6.1-001-002`,
    // `KMIPKIT-ELEM-OP-C2S-ENCRYPT`,
    // `KMIPKIT-ELEM-OP-C2S-DECRYPT`,
    // `KMIPKIT-TEST-CN01-2-99`, `KMIPKIT-TEST-CN01-2-100`,
    // `KMIPKIT-TEST-CN01-2-101`,
    // `KMIPKIT-ELEM-OPERATION-STRUCTURE-7-4-AUTHENTICATED-ENCRYPTION-TAG`,
    // `KMIPKIT-ELEM-OPERATION-STRUCTURE-7-8-CORRELATION-VALUE`, and
    // `KMIPKIT-ELEM-OPERATION-STRUCTURE-7-9-DATA`.
    let cases = [
        (
            TC_ENC_1_21_ID,
            TC_ENC_1_21_XML,
            &[1_usize, 2, 3, 4, 5, 6, 7, 8, 9, 10][..],
            &[OasisCryptoOperation::Encrypt; 10][..],
        ),
        (
            TC_ENC_2_21_ID,
            TC_ENC_2_21_XML,
            &[1_usize, 2, 3, 4, 5, 6, 7, 8, 9, 10][..],
            &[OasisCryptoOperation::Encrypt; 10][..],
        ),
        (
            TC_ENCDEC_1_21_ID,
            TC_ENCDEC_1_21_XML,
            &[1_usize, 2, 3, 4, 5, 6, 7, 8][..],
            &[
                OasisCryptoOperation::Encrypt,
                OasisCryptoOperation::Decrypt,
                OasisCryptoOperation::Encrypt,
                OasisCryptoOperation::Encrypt,
                OasisCryptoOperation::Encrypt,
                OasisCryptoOperation::Decrypt,
                OasisCryptoOperation::Decrypt,
                OasisCryptoOperation::Decrypt,
            ][..],
        ),
    ];

    let mut encrypt_responses = 0;
    let mut decrypt_responses = 0;
    for (case_id, xml, sequences, operations) in cases {
        let fixture = OasisCryptoFixture::from_xml(case_id, xml)
            .expect("the pinned fixture resolves selected response symbols deterministically");
        let pairs = fixture.into_operation_pairs();
        assert_eq!(pairs.len(), sequences.len());
        for ((pair, expected_sequence), expected_operation) in
            pairs.into_iter().zip(sequences).zip(operations)
        {
            match assert_fixture_success_response(
                pair,
                case_id,
                *expected_sequence,
                *expected_operation,
            ) {
                OasisCryptoOperation::Encrypt => encrypt_responses += 1,
                OasisCryptoOperation::Decrypt => decrypt_responses += 1,
            }
        }
    }

    assert_eq!(
        encrypt_responses, 24,
        "no in-scope Encrypt response is skipped"
    );
    assert_eq!(
        decrypt_responses, 4,
        "no in-scope Decrypt response is skipped"
    );
}

fn assert_fixture_success_response(
    pair: OasisCryptoOperationPair,
    case_id: &str,
    expected_sequence: usize,
    expected_operation: OasisCryptoOperation,
) -> OasisCryptoOperation {
    assert_eq!(pair.case_id(), case_id);
    assert_eq!(pair.source_sequence(), expected_sequence);
    assert_eq!(
        pair.step_identity(),
        format!("{case_id} step={expected_sequence}"),
        "the source RequestHeader step identity remains paired with this response"
    );
    assert_eq!(pair.operation(), expected_operation);
    let fixture_identifier = ValueSnapshot::TextString(TEST_UNIQUE_IDENTIFIER_0.to_owned());
    assert!(contains_value(
        pair.request_message(),
        UNIQUE_IDENTIFIER_TAG,
        &fixture_identifier,
    ));
    assert!(contains_value(
        pair.response_message(),
        UNIQUE_IDENTIFIER_TAG,
        &fixture_identifier,
    ));
    assert!(
        !has_unresolved_symbol(pair.response_message()),
        "selected fixture response symbols have deterministic substitutions"
    );

    let message = ResponseMessage::try_from_ttlv(pair.into_response_message())
        .expect("the selected fixture response is a structurally valid KMIP message");
    let response_item = message
        .batch_items()
        .next()
        .expect("each fixture pair retains its response batch item");
    let expected_operation_value = match expected_operation {
        OasisCryptoOperation::Encrypt => ENCRYPT_OPERATION,
        OasisCryptoOperation::Decrypt => DECRYPT_OPERATION,
    };
    assert_eq!(response_item.operation(), Some(expected_operation_value));
    assert_eq!(
        response_item.result_status(),
        Some(ResultStatus::from_raw(SUCCESS))
    );

    let payload = response_payload_snapshot(response_item);
    assert_fixture_response_table_order(expected_operation, &payload);
    assert_fixture_success_identifier(&payload);
    let expected_identifier = UniqueIdentifier::TextString(TEST_UNIQUE_IDENTIFIER_0.to_owned());

    match expected_operation {
        OasisCryptoOperation::Encrypt => {
            let response = EncryptResponse::try_from_response_item(response_item)
                .expect("a fixture Encrypt success matches Table 215");
            assert_eq!(response.result().status(), ResultStatus::from_raw(SUCCESS));
            assert_eq!(response.unique_identifier(), Some(&expected_identifier));
            assert_response_secret_matches_fixture(response.data(), &payload, DATA_TAG);
            assert_response_secret_matches_fixture(
                response.iv_counter_nonce(),
                &payload,
                IV_COUNTER_NONCE_TAG,
            );
            assert_response_secret_matches_fixture(
                response.correlation_value(),
                &payload,
                CORRELATION_VALUE_TAG,
            );
            assert_response_secret_matches_fixture(
                response.authenticated_encryption_tag(),
                &payload,
                AUTHENTICATED_ENCRYPTION_TAG,
            );
        }
        OasisCryptoOperation::Decrypt => {
            let response = DecryptResponse::try_from_response_item(response_item)
                .expect("a fixture Decrypt success matches Table 197");
            assert_eq!(response.result().status(), ResultStatus::from_raw(SUCCESS));
            assert_eq!(response.unique_identifier(), Some(&expected_identifier));
            assert_response_secret_matches_fixture(response.data(), &payload, DATA_TAG);
            assert_response_secret_matches_fixture(
                response.correlation_value(),
                &payload,
                CORRELATION_VALUE_TAG,
            );
        }
    }
    expected_operation
}
