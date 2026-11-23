// Verifies public API ownership, redaction, and borrowing invariants.
// Authority: approved specification requirement Rust value-model contract.
use kmipkit_ttlv::{Item, Structure, StructureView, Value, ValueView};
use serde::Serialize;

fn requires_serialize<T: Serialize>() {}

fn main() {
    requires_serialize::<Value>();
    requires_serialize::<ValueView<'static>>();
    requires_serialize::<StructureView<'static>>();
    requires_serialize::<Item>();
    requires_serialize::<Structure>();
}
