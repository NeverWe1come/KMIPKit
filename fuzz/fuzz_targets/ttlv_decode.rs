#![no_main]
#![forbid(unsafe_code)]

use kmipkit_protocol::{
    DecryptResponse, EncryptResponse, RequestMessage, ResponseMessage, ResultStatus,
};
use kmipkit_ttlv::codec::{CodecLimits, decode_with_limits};
use kmipkit_ttlv::{Item, Structure, Value, ValueView};
use libfuzzer_sys::fuzz_target;

const MAX_INPUT_BYTES: usize = 4 * 1024;
const MAX_STRUCTURE_DEPTH: usize = 64;
const MAX_ITEMS: usize = 512;
const ENCRYPT_OPERATION: u32 = 0x0000_001F;
const DECRYPT_OPERATION: u32 = 0x0000_0020;

fn clone_structure(source: &kmipkit_ttlv::StructureView<'_>) -> Option<Structure> {
    let mut copy = Structure::new();
    for child in source.children() {
        let value = child.with_value(clone_value)?;
        copy.try_push(Item::new(child.tag(), value).ok()?).ok()?;
    }
    Some(copy)
}

fn clone_value(source: ValueView<'_>) -> Option<Value> {
    match source {
        ValueView::Structure(value) => Some(Value::structure(clone_structure(&value)?)),
        ValueView::Integer(value) => Some(Value::integer(*value)),
        ValueView::LongInteger(value) => Some(Value::long_integer(*value)),
        ValueView::BigInteger(value) => Some(Value::big_integer(value.to_vec())),
        ValueView::Enumeration(value) => Some(Value::enumeration(*value)),
        ValueView::Boolean(value) => Some(Value::boolean(*value)),
        ValueView::TextString(value) => Some(Value::text_string((*value).to_owned())),
        ValueView::ByteString(value) => Some(Value::byte_string(value.to_vec())),
        ValueView::DateTime(value) => Some(Value::date_time(*value)),
        ValueView::Interval(value) => Some(Value::interval(*value)),
        ValueView::DateTimeExtended(value) => Some(Value::date_time_extended(*value)),
        _ => None,
    }
}

fn fuzz_validated_message_paths(decoded: &Item) {
    let Some(request_tree) = decoded.with_value(|value| match value {
        ValueView::Structure(structure) => clone_structure(&structure),
        _ => None,
    }) else {
        return;
    };
    if let Ok(request) = RequestMessage::try_from_ttlv(request_tree) {
        for item in request.batch_items() {
            if matches!(
                item.operation(),
                Some(ENCRYPT_OPERATION | DECRYPT_OPERATION)
            ) {
                let _ = item.with_request_payload(|payload| {
                    for field in payload.children() {
                        let _ = field.item_type();
                    }
                });
            }
        }
    }

    let Some(response_tree) = decoded.with_value(|value| match value {
        ValueView::Structure(structure) => clone_structure(&structure),
        _ => None,
    }) else {
        return;
    };
    if let Ok(response) = ResponseMessage::try_from_ttlv(response_tree) {
        for item in response.batch_items() {
            let _ = item.result_reason().map(|reason| {
                let _ = reason.raw();
                let _ = reason.known_name();
            });
            match (item.operation(), item.result_status()) {
                (Some(ENCRYPT_OPERATION), Some(status)) if status == ResultStatus::from_raw(2) => {
                    let _ = EncryptResponse::try_from_pending_response_item(item);
                }
                (Some(ENCRYPT_OPERATION), _) => {
                    let _ = EncryptResponse::try_from_response_item(item);
                }
                (Some(DECRYPT_OPERATION), Some(status)) if status == ResultStatus::from_raw(2) => {
                    let _ = DecryptResponse::try_from_pending_response_item(item);
                }
                (Some(DECRYPT_OPERATION), _) => {
                    let _ = DecryptResponse::try_from_response_item(item);
                }
                _ => {}
            }
        }
    }
}

fuzz_target!(|data: &[u8]| {
    if data.len() > MAX_INPUT_BYTES {
        return;
    }

    let Ok(limits) = CodecLimits::new(MAX_INPUT_BYTES, MAX_STRUCTURE_DEPTH, MAX_ITEMS) else {
        return;
    };
    if let Ok(decoded) = decode_with_limits(data, &limits) {
        fuzz_validated_message_paths(&decoded);
    }
});
