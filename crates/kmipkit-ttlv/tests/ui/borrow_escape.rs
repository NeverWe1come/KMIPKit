// Requirement traceability: KMIPKIT-0004-FR-012, SC-006.
// Authority: approved KMIPKIT-0004 Rust value-model contract.
use kmipkit_ttlv::{Item, ValueView};

fn borrowed_text(item: &Item) -> Option<&str> {
    item.with_value(|view| match view {
        ValueView::TextString(text) => Some(text),
        _ => None,
    })
}

fn main() {}
