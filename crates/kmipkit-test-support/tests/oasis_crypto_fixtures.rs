//! Fixture-derived operation-item evidence from the OASIS KMIP Test Cases
//! v2.1 Committee Note 01, §§2.99–2.101 and their pinned XML work products.
//! Operation semantics follow OASIS KMIP Specification v2.1 §§6.1.11 and
//! 6.1.17, Tables 196–198 and 214–216; opaque Data uses §7.9, Tables 360–361.
//! The adapter selects Encrypt/Decrypt item pairs from larger workflows; these
//! tests do not claim that any complete official Test Case passed.

use kmipkit_test_support::oasis_crypto_fixtures::{OasisCryptoFixture, OasisCryptoOperation};
use kmipkit_ttlv::{Item, Structure, ValueView};

const TC_ENC_1_21_ID: &str = "TC-STREAM-ENC-1-21";
const TC_ENC_2_21_ID: &str = "TC-STREAM-ENC-2-21";
const TC_ENCDEC_1_21_ID: &str = "TC-STREAM-ENCDEC-1-21";
const TIME_STAMP_TAG: u32 = 0x0042_0092;
const UNIQUE_IDENTIFIER_TAG: u32 = 0x0042_0094;
const CORRELATION_VALUE_TAG: u32 = 0x0042_00D6;
const DATA_TAG: u32 = 0x0042_00C2;
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
