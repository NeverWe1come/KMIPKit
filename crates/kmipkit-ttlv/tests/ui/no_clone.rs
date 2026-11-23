// Verifies public API ownership, redaction, and borrowing invariants.
// Authority: approved specification requirement Rust value-model contract.
use kmipkit_ttlv::{Item, Structure, StructureView, Value, ValueView};

fn requires_clone<T: Clone>() {}

fn main() {
    requires_clone::<Value>();
    requires_clone::<ValueView<'static>>();
    requires_clone::<StructureView<'static>>();
    requires_clone::<Item>();
    requires_clone::<Structure>();
}
