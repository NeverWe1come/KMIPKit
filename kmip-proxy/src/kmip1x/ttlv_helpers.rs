//! Safe, zero-unsafe TTLV helper functions built on `kmipkit-ttlv`.
#![forbid(unsafe_code)]

use std::time::{SystemTime, UNIX_EPOCH};

use kmipkit_ttlv::{Item, RawTag, Structure, Value, ValueView};

use crate::kmip1x::tags::{
    STATUS_OPERATION_FAILED, STATUS_SUCCESS, TAG_BATCH_COUNT, TAG_BATCH_ITEM, TAG_OPERATION,
    TAG_PROTOCOL_VERSION, TAG_PROTOCOL_VERSION_MAJOR, TAG_PROTOCOL_VERSION_MINOR,
    TAG_RESPONSE_HEADER, TAG_RESPONSE_MESSAGE, TAG_RESPONSE_PAYLOAD, TAG_RESULT_MESSAGE,
    TAG_RESULT_REASON, TAG_RESULT_STATUS, TAG_TIME_STAMP, TAG_UNIQUE_BATCH_ITEM_ID,
};

/// Constructs a validated TTLV `Item` from a raw 24-bit KMIP tag and `Value`.
///
/// Allows legacy KMIP 1.X tags marked reserved in KMIP 2.0+.
///
/// # Errors
///
/// Returns an error string if `tag` is out of range, unallocated, or incompatible with `val`.
pub fn ttlv_item(tag: u32, val: Value) -> Result<Item, String> {
    let raw = RawTag::new(tag)
        .map_err(|e| format!("invalid raw tag 0x{tag:06X}: {e}"))?
        .try_checked_allow_reserved()
        .map_err(|e| format!("unchecked tag 0x{tag:06X}: {e}"))?;
    Item::new(raw, val).map_err(|e| format!("failed creating item 0x{tag:06X}: {e}"))
}

/// Builds a `Structure` from a list of `Item`s.
///
/// # Errors
///
/// Returns an error string if pushing a child item fails or the structure tag is invalid.
pub fn ttlv_struct(tag: u32, children: Vec<Item>) -> Result<Item, String> {
    let mut s = Structure::new();
    for child in children {
        s.try_push(child)
            .map_err(|e| format!("structure push failed for 0x{tag:06X}: {e}"))?;
    }
    ttlv_item(tag, Value::structure(s))
}

/// Clones an `Item` using `kmipkit-ttlv`'s fallible `try_clone`.
///
/// # Errors
///
/// Returns an error string if cloning exceeds `kmipkit-ttlv` limits.
pub fn clone_item(item: &Item) -> Result<Item, String> {
    item.try_clone()
        .map_err(|e| format!("failed cloning TTLV item 0x{:06X}: {e}", item.tag().raw()))
}

/// Extracts an owned `Value` from any `Item` so it can be re-tagged.
///
/// # Errors
///
/// Returns an error string if any nested child fails to clone or uses an unsupported value variant.
pub fn clone_item_value(item: &Item) -> Result<Value, String> {
    let mut out: Option<Result<Value, String>> = None;
    item.with_value(|view| {
        let res = match view {
            ValueView::Structure(s) => {
                let mut cloned = Structure::new();
                for child in s.children() {
                    match child.try_clone() {
                        Ok(c) => {
                            if let Err(e) = cloned.try_push(c) {
                                out = Some(Err(format!("structure push failed: {e}")));
                                return;
                            }
                        }
                        Err(e) => {
                            out = Some(Err(format!("child clone failed: {e}")));
                            return;
                        }
                    }
                }
                Ok(Value::structure(cloned))
            }
            ValueView::Integer(v) => Ok(Value::integer(*v)),
            ValueView::LongInteger(v) => Ok(Value::long_integer(*v)),
            ValueView::BigInteger(b) => Ok(Value::big_integer(b.to_vec())),
            ValueView::Enumeration(v) => Ok(Value::enumeration(*v)),
            ValueView::Boolean(v) => Ok(Value::boolean(*v)),
            ValueView::TextString(s) => Ok(Value::text_string((*s).to_string())),
            ValueView::ByteString(b) => Ok(Value::byte_string(b.to_vec())),
            ValueView::DateTime(v) => Ok(Value::date_time(*v)),
            ValueView::Interval(v) => Ok(Value::interval(*v)),
            ValueView::DateTimeExtended(v) => Ok(Value::date_time_extended(*v)),
            _ => Err("unsupported TTLV value type".to_string()),
        };
        out = Some(res);
    });
    out.unwrap_or_else(|| Err("empty TTLV value view".to_string()))
}

/// Re-tags an `Item` while preserving its exact underlying `Value`.
///
/// # Errors
///
/// Returns an error string if cloning the value or constructing the re-tagged `Item` fails.
pub fn retag_item(item: &Item, new_tag: u32) -> Result<Item, String> {
    let val = clone_item_value(item)?;
    ttlv_item(new_tag, val)
}

/// Returns all cloned children of a Structure `Item`.
pub fn children_of(item: &Item) -> Vec<Item> {
    let mut out = Vec::new();
    item.with_value(|view| {
        if let ValueView::Structure(s) = view {
            for c in s.children() {
                if let Ok(cloned) = c.try_clone() {
                    out.push(cloned);
                }
            }
        }
    });
    out
}

/// Finds the first child with the given raw tag inside a Structure `Item`.
pub fn find_child(item: &Item, tag: u32) -> Option<Item> {
    let mut found = None;
    item.with_value(|view| {
        if let ValueView::Structure(s) = view {
            found = s
                .children()
                .iter()
                .find(|c| c.tag().raw() == tag)
                .and_then(|c| c.try_clone().ok());
        }
    });
    found
}

/// Returns all children with the given raw tag inside a Structure `Item`.
pub fn find_children(item: &Item, tag: u32) -> Vec<Item> {
    let mut out = Vec::new();
    item.with_value(|view| {
        if let ValueView::Structure(s) = view {
            for c in s.children() {
                if c.tag().raw() == tag {
                    if let Ok(cloned) = c.try_clone() {
                        out.push(cloned);
                    }
                }
            }
        }
    });
    out
}

/// Extracts a `ValueView::Integer` from `item`, or `None` if the value has another TTLV type.
pub fn extract_integer(item: &Item) -> Option<i32> {
    let mut out = None;
    item.with_value(|v| {
        if let ValueView::Integer(n) = v {
            out = Some(*n);
        }
    });
    out
}

/// Extracts a `ValueView::Enumeration` from `item`, or `None` if the value has another TTLV type.
pub fn extract_enumeration(item: &Item) -> Option<u32> {
    let mut out = None;
    item.with_value(|v| {
        if let ValueView::Enumeration(e) = v {
            out = Some(*e);
        }
    });
    out
}

/// Extracts a `ValueView::Boolean` from `item`, or `None` if the value has another TTLV type.
pub fn extract_boolean(item: &Item) -> Option<bool> {
    let mut out = None;
    item.with_value(|v| {
        if let ValueView::Boolean(b) = v {
            out = Some(*b);
        }
    });
    out
}

/// Extracts an owned `String` from a `ValueView::TextString`, or `None` if not a text string.
pub fn extract_text_string(item: &Item) -> Option<String> {
    let mut out = None;
    item.with_value(|v| {
        if let ValueView::TextString(s) = v {
            out = Some((*s).to_string());
        }
    });
    out
}

/// Extracts an owned `Vec<u8>` from a `ValueView::ByteString`, or `None` if not a byte string.
pub fn extract_byte_string(item: &Item) -> Option<Vec<u8>> {
    let mut out = None;
    item.with_value(|v| {
        if let ValueView::ByteString(b) = v {
            out = Some(b.to_vec());
        }
    });
    out
}

/// Returns the current UNIX epoch timestamp in seconds (or `0` if before epoch).
#[must_use]
pub fn now_unix_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Builds a `ProtocolVersion` structure `(major, minor)`.
///
/// # Errors
///
/// Returns an error string if item or structure construction fails.
pub fn build_protocol_version(major: i32, minor: i32) -> Result<Item, String> {
    ttlv_struct(
        TAG_PROTOCOL_VERSION,
        vec![
            ttlv_item(TAG_PROTOCOL_VERSION_MAJOR, Value::integer(major))?,
            ttlv_item(TAG_PROTOCOL_VERSION_MINOR, Value::integer(minor))?,
        ],
    )
}

/// Builds a successful Response `Batch Item`.
///
/// # Errors
///
/// Returns an error string if building any TTLV child or the response batch item fails.
pub fn build_success_batch_item(
    op: u32,
    unique_batch_item_id: Option<&[u8]>,
    payload_items: Vec<Item>,
) -> Result<Item, String> {
    let mut children = vec![ttlv_item(TAG_OPERATION, Value::enumeration(op))?];
    if let Some(bid) = unique_batch_item_id {
        children.push(ttlv_item(
            TAG_UNIQUE_BATCH_ITEM_ID,
            Value::byte_string(bid.to_vec()),
        )?);
    }
    children.push(ttlv_item(
        TAG_RESULT_STATUS,
        Value::enumeration(STATUS_SUCCESS),
    )?);
    children.push(ttlv_struct(TAG_RESPONSE_PAYLOAD, payload_items)?);
    ttlv_struct(TAG_BATCH_ITEM, children)
}

/// Builds a failed Response `Batch Item`.
///
/// # Errors
///
/// Returns an error string if building any TTLV child or the error batch item fails.
pub fn build_error_batch_item(
    op: u32,
    unique_batch_item_id: Option<&[u8]>,
    reason: u32,
    message: &str,
) -> Result<Item, String> {
    let mut children = vec![ttlv_item(TAG_OPERATION, Value::enumeration(op))?];
    if let Some(bid) = unique_batch_item_id {
        children.push(ttlv_item(
            TAG_UNIQUE_BATCH_ITEM_ID,
            Value::byte_string(bid.to_vec()),
        )?);
    }
    children.push(ttlv_item(
        TAG_RESULT_STATUS,
        Value::enumeration(STATUS_OPERATION_FAILED),
    )?);
    children.push(ttlv_item(TAG_RESULT_REASON, Value::enumeration(reason))?);
    children.push(ttlv_item(
        TAG_RESULT_MESSAGE,
        Value::text_string(message.to_string()),
    )?);
    ttlv_struct(TAG_BATCH_ITEM, children)
}

/// Builds a full KMIP `ResponseMessage` for `(major, minor)`.
///
/// # Errors
///
/// Returns an error string if building the header or outer `ResponseMessage` fails.
pub fn build_response_message(
    major: i32,
    minor: i32,
    batch_items: Vec<Item>,
) -> Result<Item, String> {
    let header = ttlv_struct(
        TAG_RESPONSE_HEADER,
        vec![
            build_protocol_version(major, minor)?,
            ttlv_item(TAG_TIME_STAMP, Value::date_time(now_unix_seconds()))?,
            ttlv_item(TAG_BATCH_COUNT, Value::integer(batch_items.len() as i32))?,
        ],
    )?;
    let mut root_children = Vec::with_capacity(1 + batch_items.len());
    root_children.push(header);
    root_children.extend(batch_items);
    ttlv_struct(TAG_RESPONSE_MESSAGE, root_children)
}
