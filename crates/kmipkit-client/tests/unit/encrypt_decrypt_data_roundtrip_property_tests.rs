//! Property coverage for exact Encrypt/Decrypt Data serialization round trips.
//!
//! References: KMIP Specification v2.1 §6.1.11, Table 196; §6.1.17, Table
//! 214; §7.9, Tables 360–361. These are source-derived properties, not
//! official OASIS Test Cases. Traceability: `KMIPKIT-0019-FR-001`,
//! `KMIPKIT-0019-FR-009`, and `KMIPKIT-0019-FR-012`.

use std::fmt;

use kmipkit_protocol::{
    DecryptRequest, EncryptRequest, OperationData, SecretBytes, UniqueIdentifier,
};
use kmipkit_ttlv::codec::{CodecLimits, decode_with_limits};
use kmipkit_ttlv::{StructureView, ValueView};
use quickcheck::{Arbitrary, Gen, QuickCheck};

use crate::execute::{encode_message_for_test, request_message_for_test};
use crate::{ClientBatch, ClientBatchItem, ClientRequest};

const PROPERTY_SEED: u64 = 0x4b4d_4950_4441_5441;
const PROPERTY_RUNS: u64 = 256;
const MAX_OPERATION_DATA_BYTES: usize = 256;

const ENCRYPT_OPERATION: u32 = 0x0000_001F;
const DECRYPT_OPERATION: u32 = 0x0000_0020;
const BATCH_ITEM: u32 = 0x0042_000F;
const OPERATION: u32 = 0x0042_005C;
const REQUEST_PAYLOAD: u32 = 0x0042_0079;
const DATA: u32 = 0x0042_00C2;
const KEY_IDENTIFIER_SENTINEL: &str = "T042_KEY_IDENTIFIER_SENTINEL";

#[derive(Clone)]
struct OperationDataCase {
    variant: u8,
    bytes: Vec<u8>,
    enumeration: u32,
    integer: i32,
}

impl fmt::Debug for OperationDataCase {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("OperationDataCase([REDACTED])")
    }
}

impl Arbitrary for OperationDataCase {
    fn arbitrary(generator: &mut Gen) -> Self {
        let mut bytes = Vec::<u8>::arbitrary(generator);
        bytes.truncate(MAX_OPERATION_DATA_BYTES);
        Self {
            variant: u8::arbitrary(generator),
            bytes,
            enumeration: u32::arbitrary(generator),
            integer: i32::arbitrary(generator),
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(std::iter::empty())
    }
}

fn operation_data(case: &OperationDataCase) -> OperationData {
    match case.variant % 3 {
        0 => OperationData::ByteString(SecretBytes::new(case.bytes.clone())),
        1 => OperationData::Enumeration(case.enumeration),
        _ => OperationData::Integer(case.integer),
    }
}

fn request_bytes(case: &OperationDataCase, encrypt: bool) -> Option<Vec<u8>> {
    let limits = CodecLimits::defaults();
    let identifier = Some(UniqueIdentifier::TextString(
        KEY_IDENTIFIER_SENTINEL.to_owned(),
    ));
    let data = Some(operation_data(case));
    let request = if encrypt {
        ClientRequest::encrypt(EncryptRequest::new(identifier, data))
    } else {
        ClientRequest::decrypt(DecryptRequest::new(identifier, data))
    };
    let request_message =
        request_message_for_test(ClientBatch::new(ClientBatchItem::new(request)), &limits).ok()?;
    encode_message_for_test(request_message.into_ttlv(), &limits).ok()
}

fn structure_child<'a>(
    structure: &'a StructureView<'a>,
    tag: u32,
) -> Option<&'a kmipkit_ttlv::Item> {
    structure
        .children()
        .iter()
        .find(|child| child.tag().raw() == tag)
}

fn value_structure<R>(
    value: ValueView<'_>,
    callback: impl FnOnce(&StructureView<'_>) -> R,
) -> Option<R> {
    match value {
        ValueView::Structure(structure) => Some(callback(&structure)),
        _ => None,
    }
}

fn decoded_data_matches(bytes: &[u8], operation: u32, case: &OperationDataCase) -> bool {
    let limits = CodecLimits::defaults();
    let Ok(decoded) = decode_with_limits(bytes, &limits) else {
        return false;
    };
    decoded.with_value(|root_value| {
        value_structure(root_value, |root| {
            let Some(batch_item) = structure_child(root, BATCH_ITEM) else {
                return false;
            };
            batch_item.with_value(|batch_value| {
                value_structure(batch_value, |batch| {
                    let operation_matches = structure_child(batch, OPERATION)
                        .is_some_and(|field| {
                            field.with_value(|value| {
                                matches!(value, ValueView::Enumeration(actual) if *actual == operation)
                            })
                        });
                    let data_matches = structure_child(batch, REQUEST_PAYLOAD)
                        .is_some_and(|payload| {
                            payload.with_value(|payload_value| {
                                value_structure(payload_value, |payload| {
                                    structure_child(payload, DATA).is_some_and(|data| {
                                        match case.variant % 3 {
                                            0 => data.with_value(|value| {
                                                matches!(value, ValueView::ByteString(actual) if *actual == case.bytes)
                                            }),
                                            1 => data.with_value(|value| {
                                                matches!(value, ValueView::Enumeration(actual) if *actual == case.enumeration)
                                            }),
                                            _ => data.with_value(|value| {
                                                matches!(value, ValueView::Integer(actual) if *actual == case.integer)
                                            }),
                                        }
                                    })
                                })
                            })
                            .unwrap_or(false)
                        });
                    operation_matches && data_matches
                })
            })
            .unwrap_or(false)
        })
        .unwrap_or(false)
    })
}

#[allow(clippy::needless_pass_by_value)]
fn arbitrary_data_round_trips_through_ttlv_for_both_operations(case: OperationDataCase) -> bool {
    request_bytes(&case, true)
        .is_some_and(|bytes| decoded_data_matches(&bytes, ENCRYPT_OPERATION, &case))
        && request_bytes(&case, false)
            .is_some_and(|bytes| decoded_data_matches(&bytes, DECRYPT_OPERATION, &case))
}

#[test]
fn arbitrary_data_variants_round_trip_exactly_through_encrypt_and_decrypt_ttlv() {
    QuickCheck::new()
        .rng(Gen::from_size_and_seed(64, PROPERTY_SEED))
        .tests(PROPERTY_RUNS)
        .quickcheck(
            arbitrary_data_round_trips_through_ttlv_for_both_operations
                as fn(OperationDataCase) -> bool,
        );
}
