//! Secret-safe KMIP Ticket model requirements from OASIS KMIP v2.1 §7.39, Table 391.
//!
//! Ticket Type remains an open Enumeration, Ticket Value remains secret byte
//! data, and structurally valid unknown children retain their exact order.

use crate::{SecretBytes, Ticket, TicketType};
use kmipkit_ttlv::{Item, RawTag, Structure, Tag, Value};
use quickcheck::QuickCheck;

const TICKET_TYPE: u32 = 0x0042_014A;
const TICKET_VALUE: u32 = 0x0042_014B;
const COMMENT: u32 = 0x0042_0042;
const VENDOR_EXTENSION: u32 = 0x0054_1234;
const SECRET_SENTINEL: &[u8] = b"ticket-value-secret-sentinel";

fn tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("fixture tag fits the TTLV tag width")
        .try_checked()
        .expect("fixture tag is allocated by KMIP 2.1")
}

fn item(raw_tag: u32, value: Value) -> Item {
    Item::new(tag(raw_tag), value).expect("checked fixture tag and value form an item")
}

fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut result = Structure::new();
    for child in items {
        result
            .try_push(child)
            .expect("fixture stays within the TTLV nesting limit");
    }
    result
}

fn valid_ttlv(ticket_type: u32, value: Vec<u8>) -> Structure {
    structure([
        item(TICKET_TYPE, Value::enumeration(ticket_type)),
        item(TICKET_VALUE, Value::byte_string(value)),
    ])
}

#[test]
fn ticket_constructor_owns_value_and_redacts_debug_and_display() {
    let ticket = Ticket::new(
        TicketType::from_raw(1),
        SecretBytes::new(SECRET_SENTINEL.to_vec()),
    );

    assert_eq!(ticket.ticket_type().raw(), 1);
    ticket.with_ticket_value(|value| assert_eq!(value, SECRET_SENTINEL));

    let debug = format!("{ticket:?}");
    let display = ticket.to_string();
    assert!(!debug.contains("ticket-value-secret-sentinel"));
    assert!(!display.contains("ticket-value-secret-sentinel"));
    assert!(debug.contains("REDACTED"));
    assert!(display.contains("redacted"));
}

#[test]
fn ticket_type_preserves_unassigned_and_vendor_enumeration_values() {
    for raw in [0, 2, 0x7FFF_FFFF, 0x8000_0001, u32::MAX] {
        assert_eq!(TicketType::from_raw(raw).raw(), raw);
    }
}

#[test]
fn ttlv_parse_preserves_unknown_children_and_original_order() {
    let parsed = Ticket::try_from_ttlv(structure([
        item(COMMENT, Value::text_string("opaque metadata".to_owned())),
        item(TICKET_VALUE, Value::byte_string(SECRET_SENTINEL.to_vec())),
        item(VENDOR_EXTENSION, Value::enumeration(0xF123_4567)),
        item(TICKET_TYPE, Value::enumeration(0x8000_0042)),
    ]))
    .expect("valid Ticket fields with unknown children are retained");

    assert_eq!(parsed.ticket_type().raw(), 0x8000_0042);
    parsed.with_ticket_value(|value| assert_eq!(value, SECRET_SENTINEL));
    let restored = parsed.into_ttlv();
    let tags = restored
        .view()
        .children()
        .iter()
        .map(|child| child.tag().raw())
        .collect::<Vec<_>>();
    assert_eq!(tags, [COMMENT, TICKET_VALUE, VENDOR_EXTENSION, TICKET_TYPE]);
}

#[test]
fn ttlv_parse_rejects_missing_duplicate_and_wrong_type_required_fields() {
    let malformed = [
        structure([item(TICKET_VALUE, Value::byte_string(vec![1]))]),
        structure([item(TICKET_TYPE, Value::enumeration(1))]),
        structure([
            item(TICKET_TYPE, Value::enumeration(1)),
            item(TICKET_TYPE, Value::enumeration(1)),
            item(TICKET_VALUE, Value::byte_string(vec![1])),
        ]),
        structure([
            item(TICKET_TYPE, Value::text_string("Login".to_owned())),
            item(TICKET_VALUE, Value::byte_string(vec![1])),
        ]),
        structure([
            item(TICKET_TYPE, Value::enumeration(1)),
            item(TICKET_VALUE, Value::structure(Structure::new())),
        ]),
    ];

    for ticket in malformed {
        assert!(Ticket::try_from_ttlv(ticket).is_err());
    }
}

#[test]
fn malformed_secret_value_error_never_formats_payload_bytes() {
    let ticket = structure([
        item(TICKET_TYPE, Value::enumeration(1)),
        item(
            TICKET_VALUE,
            Value::text_string(String::from_utf8(SECRET_SENTINEL.to_vec()).expect("ASCII")),
        ),
    ]);

    let error = Ticket::try_from_ttlv(ticket)
        .err()
        .expect("wrong Ticket Value item type is rejected");
    let formatted = format!("{error} {error:?}");
    assert!(!formatted.contains("ticket-value-secret-sentinel"));
}

#[test]
fn ticket_ttlv_round_trip_preserves_arbitrary_raw_type_and_secret_bytes() {
    fn round_trip(raw_type: u32, value: Vec<u8>) -> bool {
        let parsed = match Ticket::try_from_ttlv(valid_ttlv(raw_type, value.clone())) {
            Ok(ticket) => ticket,
            Err(_) => return false,
        };
        if parsed.ticket_type().raw() != raw_type {
            return false;
        }
        let exact_value = parsed.with_ticket_value(|bytes| bytes == value);
        if !exact_value {
            return false;
        }
        match Ticket::try_from_ttlv(parsed.into_ttlv()) {
            Ok(restored) => {
                restored.ticket_type().raw() == raw_type
                    && restored.with_ticket_value(|bytes| bytes == value)
            }
            Err(_) => false,
        }
    }

    QuickCheck::new()
        .tests(128)
        .quickcheck(round_trip as fn(u32, Vec<u8>) -> bool);
}