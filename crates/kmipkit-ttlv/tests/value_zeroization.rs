//! Requirement traceability: KMIPKIT-0004-FR-012 and SC-006.
//! Authority: approved KMIPKIT-0004 specification and Rust value-model contract.

#![cfg(not(coverage))]
#![allow(unexpected_cfgs, dead_code)]
#![forbid(unsafe_code)]

include!("../src/value.rs");

#[path = "../src/error.rs"]
mod error;
pub use error::ModelError;

pub use kmipkit_ttlv::{RawTag, Tag};

#[path = "../src/item.rs"]
mod item;
pub use item::Item;

#[path = "../src/structure.rs"]
pub mod structure;

mod value {
    #[cfg(test)]
    mod tests {
        use std::{cell::Cell, rc::Rc};

        use zeroize::Zeroize;

        use super::super::{Secret, Structure, Value, ValueView};
        use crate::{Item, RawTag, Tag};

        struct DropSpy(Rc<Cell<usize>>);

        impl Zeroize for DropSpy {
            fn zeroize(&mut self) {
                self.0.set(self.0.get() + 1);
            }
        }

        fn checked_tag() -> Tag {
            RawTag::new(0x0042_0173)
                .expect("the test tag fits the KMIP width")
                .try_checked()
                .expect("the test tag is assigned in KMIP 2.1")
        }

        fn structure_with_secret_text() -> Structure {
            let child = Item::new(
                checked_tag(),
                Value::text_string(String::from("nested-secret-sentinel")),
            )
            .expect("the checked tag and value form an item");
            let mut structure = Structure::new();
            structure
                .try_push(child)
                .expect("the text leaf does not deepen the structure");
            structure
        }

        fn nested_text_is_empty(value: &Value) -> bool {
            match value.as_view() {
                ValueView::Structure(structure) => match structure.children().first() {
                    Some(item) => item.with_value(|view| match view {
                        ValueView::TextString(text) => text.is_empty(),
                        _ => false,
                    }),
                    None => false,
                },
                _ => false,
            }
        }

        #[test]
        fn secret_drop_calls_zeroize_on_its_boxed_payload() {
            let call_count = Rc::new(Cell::new(0));

            drop(Secret::new(DropSpy(Rc::clone(&call_count))));

            assert_eq!(call_count.get(), 1);
        }

        #[test]
        fn live_zeroize_clears_each_payload_variant_and_nested_structure() {
            let mut values = vec![
                Value::structure(structure_with_secret_text()),
                Value::integer(-1_234_567),
                Value::long_integer(-6_543_210_987),
                Value::big_integer(vec![0xD1, 0xE2, 0xA3]),
                Value::enumeration(0xDEAD_BEEF),
                Value::boolean(true),
                Value::text_string(String::from("live-text-secret-sentinel")),
                Value::byte_string(vec![0x53, 0x45, 0x43]),
                Value::date_time(9_876_543_210),
                Value::interval(0xA1B2_C3D4),
                Value::date_time_extended(-8_765_432_109),
            ];

            for value in &mut values {
                value.inner.boxed.as_mut().zeroize();
            }

            let cleared = values
                .iter()
                .skip(1)
                .map(|value| match value.as_view() {
                    ValueView::Integer(number) => *number == 0,
                    ValueView::LongInteger(number) => *number == 0,
                    ValueView::BigInteger(bytes) => bytes.is_empty(),
                    ValueView::Enumeration(number) => *number == 0,
                    ValueView::Boolean(boolean) => !boolean,
                    ValueView::TextString(text) => text.is_empty(),
                    ValueView::ByteString(bytes) => bytes.is_empty(),
                    ValueView::DateTime(number) => *number == 0,
                    ValueView::Interval(number) => *number == 0,
                    ValueView::DateTimeExtended(number) => *number == 0,
                    ValueView::Structure(_) => false,
                })
                .collect::<Vec<_>>();

            assert_eq!(cleared, [true; 10]);
            assert!(nested_text_is_empty(&values[0]));
        }
    }
}
