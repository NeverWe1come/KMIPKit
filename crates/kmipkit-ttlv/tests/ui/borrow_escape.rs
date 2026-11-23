// Requirement traceability: functional requirement, SC-006.
// Authority: approved specification requirement Rust value-model contract.
use kmipkit_ttlv::{Item, ValueView};

fn borrowed_text(item: &Item) -> Option<&str> {
    item.with_value(|view| match view {
        ValueView::TextString(text) => Some(text),
        _ => None,
    })
}

fn main() {}
