//! KMIPKIT-0050 Authentication Request Header tests.
//!
//! OASIS KMIP v2.1 §8.2, Table 395 places Authentication after Attestation
//! Type and before Batch Error Continuation Option. The current client omits
//! Attestation Type, so Authentication follows Attestation Capable Indicator.

use kmipkit_protocol::{Authentication, RequestMessage};
use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::{Item, RawTag, Structure, Tag, Value, ValueView};

use crate::execute::{ClientBatch, ClientBatchItem, ClientRequest, request_message_for_test};

const REQUEST_HEADER: u32 = 0x0042_0077;
const AUTHENTICATION: u32 = 0x0042_0001;
const ATTESTATION_CAPABLE_INDICATOR: u32 = 0x0042_00D3;
const BATCH_ERROR_CONTINUATION_OPTION: u32 = 0x0042_000E;
const CREDENTIAL: u32 = 0x0042_0023;
const CREDENTIAL_TYPE: u32 = 0x0042_0024;
const CREDENTIAL_VALUE: u32 = 0x0042_0025;
const VENDOR_FIELD: u32 = 0x0054_0001;
const SECRET_SENTINEL: &str = "AUTHENTICATION_SECRET_SENTINEL";

fn tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("test tag fits TTLV width")
        .try_checked()
        .expect("test tag is allocated")
}

fn item(raw: u32, value: Value) -> Item {
    Item::new(tag(raw), value).expect("test item is valid")
}

fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut tree = Structure::new();
    for field in items {
        tree.try_push(field)
            .expect("test tree is within TTLV limits");
    }
    tree
}

fn authentication() -> Authentication {
    let value = structure([item(
        VENDOR_FIELD,
        Value::text_string(SECRET_SENTINEL.to_owned()),
    )]);
    let credential = structure([
        item(CREDENTIAL_TYPE, Value::enumeration(0x8FFF_0042)),
        item(CREDENTIAL_VALUE, Value::structure(value)),
    ]);
    Authentication::try_from_ttlv(structure([item(CREDENTIAL, Value::structure(credential))]))
        .expect("non-empty unknown Credential is retained as generic TTLV")
}

fn header_tags(request: &RequestMessage) -> Vec<u32> {
    request.with_ttlv(|message| {
        message
            .children()
            .iter()
            .find(|field| field.tag().raw() == REQUEST_HEADER)
            .map(|field| {
                field.with_value(|value| match value {
                    ValueView::Structure(header) => header
                        .children()
                        .iter()
                        .map(|field| field.tag().raw())
                        .collect(),
                    _ => Vec::new(),
                })
            })
            .unwrap_or_default()
    })
}

fn authentication_count(request: &RequestMessage) -> usize {
    header_tags(request)
        .into_iter()
        .filter(|tag| *tag == AUTHENTICATION)
        .count()
}

#[test]
fn authentication_is_emitted_once_in_request_header_order_for_one_item() {
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::discover_versions()))
        .with_authentication(authentication());
    let request = request_message_for_test(batch, &CodecLimits::defaults())
        .expect("typed Authentication is encoded in the request header");
    let tags = header_tags(&request);

    assert_eq!(authentication_count(&request), 1);
    assert!(
        tags.iter()
            .position(|tag| *tag == ATTESTATION_CAPABLE_INDICATOR)
            < tags.iter().position(|tag| *tag == AUTHENTICATION)
    );
}

#[test]
fn authentication_is_one_message_field_for_a_multi_item_batch() {
    let batch = ClientBatch::from_items([
        ClientBatchItem::new(ClientRequest::discover_versions()),
        ClientBatchItem::new(ClientRequest::discover_versions()),
    ])
    .with_authentication(authentication())
    .with_batch_error_continuation_option(1);
    let request = request_message_for_test(batch, &CodecLimits::defaults())
        .expect("shared header Authentication is encoded once");
    let tags = header_tags(&request);
    let auth_index = tags
        .iter()
        .position(|tag| *tag == AUTHENTICATION)
        .expect("Authentication is present");
    let batch_error_index = tags
        .iter()
        .position(|tag| *tag == BATCH_ERROR_CONTINUATION_OPTION)
        .expect("batch continuation follows Authentication");

    assert_eq!(authentication_count(&request), 1);
    assert!(auth_index < batch_error_index);
}

#[test]
fn authentication_is_omitted_by_default_and_batch_formatting_is_redacted() {
    let plain = ClientBatch::new(ClientBatchItem::new(ClientRequest::discover_versions()));
    let request = request_message_for_test(plain, &CodecLimits::defaults())
        .expect("unauthenticated request remains supported");
    assert_eq!(authentication_count(&request), 0);

    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::discover_versions()))
        .with_authentication(authentication());
    assert!(!format!("{batch:?}").contains(SECRET_SENTINEL));
    assert!(!batch.to_string().contains(SECRET_SENTINEL));
}
