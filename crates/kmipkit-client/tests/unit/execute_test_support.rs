//! Shared, deterministic TTLV fixtures for client execution tests.

use kmipkit_transport::{RequestOptions, Transport, TransportError, TransportResponse};
use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::{Item, RawTag, Structure, StructureView, Tag, Value, ValueView};

use crate::execute::encode_message_for_test;
use crate::{ClientBatchOutcome, ClientOperation, ClientRequest};

pub(crate) fn assert_lifecycle_request_debug(request: &ClientRequest, expected_name: &str) {
    let formatted = format!("{request:?}");
    assert!(
        formatted.starts_with(expected_name),
        "request debug output should identify {expected_name}"
    );
}

pub(crate) fn assert_lifecycle_outcome_surface(
    outcome: &ClientBatchOutcome,
    expected_operation: ClientOperation,
    response_debug_name: &str,
    display_prefix: &str,
) {
    let response = outcome.response();
    assert_eq!(
        outcome.certify_response().is_some(),
        expected_operation == ClientOperation::Certify
    );
    assert_eq!(
        response.certify().is_some(),
        expected_operation == ClientOperation::Certify
    );
    assert_eq!(
        outcome.check_response().is_some(),
        expected_operation == ClientOperation::Check
    );
    assert_eq!(
        response.check().is_some(),
        expected_operation == ClientOperation::Check
    );
    assert_eq!(
        outcome.obtain_lease_response().is_some(),
        expected_operation == ClientOperation::ObtainLease
    );
    assert_eq!(
        response.obtain_lease().is_some(),
        expected_operation == ClientOperation::ObtainLease
    );
    assert_eq!(
        outcome.recertify_response().is_some(),
        expected_operation == ClientOperation::Recertify
    );
    assert_eq!(
        response.recertify().is_some(),
        expected_operation == ClientOperation::Recertify
    );
    assert_eq!(
        outcome.rekey_response().is_some(),
        expected_operation == ClientOperation::Rekey
    );
    assert_eq!(
        response.rekey().is_some(),
        expected_operation == ClientOperation::Rekey
    );
    assert_eq!(
        outcome.rekey_key_pair_response().is_some(),
        expected_operation == ClientOperation::RekeyKeyPair
    );
    assert_eq!(
        response.rekey_key_pair().is_some(),
        expected_operation == ClientOperation::RekeyKeyPair
    );
    assert_eq!(
        outcome.reprovision_response().is_some(),
        expected_operation == ClientOperation::ReProvision
    );
    assert_eq!(
        response.reprovision().is_some(),
        expected_operation == ClientOperation::ReProvision
    );
    assert_eq!(
        outcome.register_response().is_some(),
        expected_operation == ClientOperation::Register
    );
    assert_eq!(
        response.register().is_some(),
        expected_operation == ClientOperation::Register
    );
    assert_eq!(
        outcome.revoke_response().is_some(),
        expected_operation == ClientOperation::Revoke
    );
    assert_eq!(
        response.revoke().is_some(),
        expected_operation == ClientOperation::Revoke
    );

    assert_eq!(
        response.result().status().raw(),
        outcome.result().status().raw()
    );
    assert!(
        format!("{response:?}").starts_with(response_debug_name),
        "response debug output should identify {response_debug_name}"
    );
    let outcome_debug = format!("{outcome:?}");
    assert!(
        outcome_debug.contains(response_debug_name.trim_end_matches('(')),
        "outcome debug output should identify {response_debug_name}"
    );
    assert!(
        outcome.to_string().starts_with(display_prefix),
        "outcome display should start with {display_prefix}"
    );
}

pub(crate) struct TestTransport(Box<dyn Transport>);

impl TestTransport {
    pub(crate) fn new<T: Transport + 'static>(transport: T) -> Self {
        Self(Box::new(transport))
    }

    pub(crate) fn exchange_with_options(
        &mut self,
        request: &[u8],
        max_response_bytes: usize,
        _options: &RequestOptions,
    ) -> Result<TransportResponse, TransportError> {
        self.0.exchange(request, max_response_bytes)
    }
}

pub(crate) const DISCOVER_VERSIONS_OPERATION: u32 = 0x0000_001E;
const RESPONSE_HEADER: u32 = 0x0042_007A;
const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
const TIME_STAMP: u32 = 0x0042_0092;
const BATCH_COUNT: u32 = 0x0042_000D;
const BATCH_ITEM: u32 = 0x0042_000F;
const OPERATION: u32 = 0x0042_005C;
const UNIQUE_BATCH_ITEM_ID: u32 = 0x0042_0093;
const RESULT_STATUS: u32 = 0x0042_007F;
const RESULT_REASON: u32 = 0x0042_007E;
const RESULT_MESSAGE: u32 = 0x0042_007D;
const ASYNCHRONOUS_CORRELATION_VALUE: u32 = 0x0042_0006;
const RESPONSE_PAYLOAD: u32 = 0x0042_007C;
const PROTOCOL_VERSION_FIELD: u32 = 0x0042_0069;
const MESSAGE_EXTENSION: u32 = 0x0042_0051;
const VENDOR_IDENTIFICATION: u32 = 0x0042_009D;
const CRITICALITY_INDICATOR: u32 = 0x0042_0026;
const VENDOR_EXTENSION: u32 = 0x0042_009C;
const EXTENSION_PAYLOAD_TAG: u32 = 0x0054_0173;

pub(crate) fn structure_contains_tag(tree: &StructureView<'_>, tag: u32) -> bool {
    tree.children().iter().any(|item| {
        item.tag().raw() == tag
            || item.with_value(|value| match value {
                ValueView::Structure(nested) => structure_contains_tag(&nested, tag),
                _ => false,
            })
    })
}

pub(crate) fn structure_contains_byte_string(
    tree: &StructureView<'_>,
    tag: u32,
    expected: &[u8],
) -> bool {
    tree.children().iter().any(|item| {
        item.with_value(|value| match value {
            ValueView::ByteString(bytes) => item.tag().raw() == tag && bytes == expected,
            ValueView::Structure(nested) => structure_contains_byte_string(&nested, tag, expected),
            _ => false,
        })
    })
}

pub(crate) fn asynchronous_response_bytes(
    operation: u32,
    status: u32,
    reason: Option<u32>,
    correlation_value: Option<&[u8]>,
    payload: Option<Structure>,
) -> Vec<u8> {
    one_item_response_bytes(
        operation,
        status,
        reason,
        None,
        None,
        correlation_value,
        payload,
    )
}

pub(crate) fn asynchronous_response_with_batch_id_bytes(
    operation: u32,
    status: u32,
    reason: Option<u32>,
    correlation_value: Option<&[u8]>,
    unique_batch_item_id: Option<&[u8]>,
    payload: Option<Structure>,
) -> Vec<u8> {
    one_item_response_bytes(
        operation,
        status,
        reason,
        None,
        unique_batch_item_id,
        correlation_value,
        payload,
    )
}

pub(crate) fn one_item_response_bytes(
    operation: u32,
    status: u32,
    reason: Option<u32>,
    result_message: Option<&str>,
    unique_batch_item_id: Option<&[u8]>,
    asynchronous_correlation_value: Option<&[u8]>,
    payload: Option<Structure>,
) -> Vec<u8> {
    let response_version = structure([
        item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
    ]);
    let header = structure([
        item(PROTOCOL_VERSION, Value::structure(response_version)),
        item(TIME_STAMP, Value::date_time(1)),
        item(BATCH_COUNT, Value::integer(1)),
    ]);
    let mut fields = vec![item(OPERATION, Value::enumeration(operation))];
    if let Some(id) = unique_batch_item_id {
        fields.push(item(UNIQUE_BATCH_ITEM_ID, Value::byte_string(id.to_vec())));
    }
    fields.push(item(RESULT_STATUS, Value::enumeration(status)));
    if let Some(reason) = reason {
        fields.push(item(RESULT_REASON, Value::enumeration(reason)));
    }
    if let Some(message) = result_message {
        fields.push(item(RESULT_MESSAGE, Value::text_string(message.to_owned())));
    }
    if let Some(correlation) = asynchronous_correlation_value {
        fields.push(item(
            ASYNCHRONOUS_CORRELATION_VALUE,
            Value::byte_string(correlation.to_vec()),
        ));
    }
    if let Some(payload) = payload {
        fields.push(item(RESPONSE_PAYLOAD, Value::structure(payload)));
    }
    let tree = structure([
        item(RESPONSE_HEADER, Value::structure(header)),
        item(BATCH_ITEM, Value::structure(structure(fields))),
    ]);
    encode_message_for_test(tree, &CodecLimits::defaults())
        .expect("one-item test response is encodable")
}

pub(crate) fn operation_batch_response_bytes(
    items: impl IntoIterator<Item = (u32, Vec<u8>, Structure)>,
) -> Vec<u8> {
    let items = items.into_iter().collect::<Vec<_>>();
    let response_version = structure([
        item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
    ]);
    let header = structure([
        item(PROTOCOL_VERSION, Value::structure(response_version)),
        item(TIME_STAMP, Value::date_time(1)),
        item(
            BATCH_COUNT,
            Value::integer(i32::try_from(items.len()).expect("fixture item count fits i32")),
        ),
    ]);
    let mut message = vec![item(RESPONSE_HEADER, Value::structure(header))];
    message.extend(items.into_iter().map(|(operation, id, payload)| {
        item(
            BATCH_ITEM,
            Value::structure(structure([
                item(OPERATION, Value::enumeration(operation)),
                item(UNIQUE_BATCH_ITEM_ID, Value::byte_string(id)),
                item(RESULT_STATUS, Value::enumeration(0)),
                item(RESPONSE_PAYLOAD, Value::structure(payload)),
            ])),
        )
    }));
    encode_message_for_test(structure(message), &CodecLimits::defaults())
        .expect("operation batch test response is encodable")
}

pub(crate) fn test_item(raw_tag: u32, value: Value) -> Item {
    item(raw_tag, value)
}

pub(crate) fn test_structure(items: impl IntoIterator<Item = Item>) -> Structure {
    structure(items)
}

#[derive(Clone, Debug)]
pub(crate) struct ResponseItemFixture {
    pub(crate) operation: u32,
    pub(crate) unique_batch_item_id: Option<Vec<u8>>,
    pub(crate) result_status: u32,
    pub(crate) asynchronous_correlation_value: Option<Vec<u8>>,
    pub(crate) extension_criticality: Option<bool>,
    pub(crate) empty_supported_versions: bool,
    pub(crate) supported_versions: Vec<(i32, i32)>,
}

impl ResponseItemFixture {
    pub(crate) fn success(id: Option<&[u8]>) -> Self {
        Self {
            operation: DISCOVER_VERSIONS_OPERATION,
            unique_batch_item_id: id.map(<[u8]>::to_vec),
            result_status: 0,
            asynchronous_correlation_value: None,
            extension_criticality: None,
            empty_supported_versions: false,
            supported_versions: vec![(2, 1)],
        }
    }

    pub(crate) fn pending(id: Option<&[u8]>, correlation: &[u8]) -> Self {
        Self {
            result_status: 2,
            asynchronous_correlation_value: Some(correlation.to_vec()),
            ..Self::success(id)
        }
    }

    pub(crate) fn failure(id: Option<&[u8]>) -> Self {
        Self {
            result_status: 1,
            ..Self::success(id)
        }
    }

    pub(crate) fn with_empty_supported_versions(mut self) -> Self {
        self.empty_supported_versions = true;
        self
    }

    pub(crate) fn with_supported_version(mut self, version: (i32, i32)) -> Self {
        self.supported_versions = vec![version];
        self
    }

    pub(crate) fn with_supported_versions(mut self, versions: Vec<(i32, i32)>) -> Self {
        self.supported_versions = versions;
        self
    }
}

pub(crate) fn response_bytes(version: (i32, i32), items: &[ResponseItemFixture]) -> Vec<u8> {
    let response_version = structure([
        item(PROTOCOL_VERSION_MAJOR, Value::integer(version.0)),
        item(PROTOCOL_VERSION_MINOR, Value::integer(version.1)),
    ]);
    let header = structure([
        item(PROTOCOL_VERSION, Value::structure(response_version)),
        item(TIME_STAMP, Value::date_time(1)),
        item(
            BATCH_COUNT,
            Value::integer(i32::try_from(items.len()).expect("test item count fits i32")),
        ),
    ]);
    let mut message = vec![item(RESPONSE_HEADER, Value::structure(header))];
    message.extend(items.iter().map(response_item));
    let tree = structure(message);
    let limits = CodecLimits::defaults();
    encode_message_for_test(tree, &limits).expect("test response is encodable")
}

fn response_item(fixture: &ResponseItemFixture) -> Item {
    let mut fields = vec![item(OPERATION, Value::enumeration(fixture.operation))];
    if let Some(id) = &fixture.unique_batch_item_id {
        fields.push(item(UNIQUE_BATCH_ITEM_ID, Value::byte_string(id.clone())));
    }
    fields.push(item(
        RESULT_STATUS,
        Value::enumeration(fixture.result_status),
    ));
    if fixture.result_status == 1 {
        fields.push(item(RESULT_REASON, Value::enumeration(1)));
        fields.push(item(
            RESULT_MESSAGE,
            Value::text_string("fixture failure".to_owned()),
        ));
    }
    if let Some(value) = &fixture.asynchronous_correlation_value {
        fields.push(item(
            ASYNCHRONOUS_CORRELATION_VALUE,
            Value::byte_string(value.clone()),
        ));
    }
    if matches!(fixture.result_status, 0 | 2) {
        fields.push(item(
            RESPONSE_PAYLOAD,
            Value::structure(response_payload(
                fixture.empty_supported_versions,
                &fixture.supported_versions,
            )),
        ));
    }
    if let Some(criticality) = fixture.extension_criticality {
        fields.push(item(
            MESSAGE_EXTENSION,
            Value::structure(extension(criticality)),
        ));
    }
    item(BATCH_ITEM, Value::structure(structure(fields)))
}

fn response_payload(empty_supported_versions: bool, versions: &[(i32, i32)]) -> Structure {
    let mut payload = Structure::new();
    if !empty_supported_versions {
        for (major, minor) in versions {
            payload
                .try_push(item(
                    PROTOCOL_VERSION_FIELD,
                    Value::structure(structure([
                        item(PROTOCOL_VERSION_MAJOR, Value::integer(*major)),
                        item(PROTOCOL_VERSION_MINOR, Value::integer(*minor)),
                    ])),
                ))
                .expect("fixture response payload fits model depth limits");
        }
    }
    payload
}

fn extension(criticality: bool) -> Structure {
    structure([
        item(
            VENDOR_IDENTIFICATION,
            Value::text_string("FixtureVendor".to_owned()),
        ),
        item(CRITICALITY_INDICATOR, Value::boolean(criticality)),
        item(
            VENDOR_EXTENSION,
            Value::structure(structure([item(
                EXTENSION_PAYLOAD_TAG,
                Value::byte_string(b"extension fixture".to_vec()),
            )])),
        ),
    ])
}

fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut structure = Structure::new();
    for item in items {
        structure
            .try_push(item)
            .expect("fixture structure fits model depth limits");
    }
    structure
}

fn item(raw_tag: u32, value: Value) -> Item {
    Item::new(tag(raw_tag), value).expect("fixture tag and value form a valid Item")
}

fn tag(raw_tag: u32) -> Tag {
    RawTag::new(raw_tag)
        .expect("fixture tag fits TTLV width")
        .try_checked()
        .expect("fixture tag is allocated")
}
