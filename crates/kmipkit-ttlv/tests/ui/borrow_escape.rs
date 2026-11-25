// Verifies public API ownership, redaction, and borrowing invariants.
// Enforces that borrowed views cannot escape item closure lifetimes.
use kmipkit_ttlv::{Item, ValueView};

fn borrowed_text(item: &Item) -> Option<&str> {
    item.with_value(|view| match view {
        ValueView::TextString(text) => Some(text),
        _ => None,
    })
}

fn main() {}
