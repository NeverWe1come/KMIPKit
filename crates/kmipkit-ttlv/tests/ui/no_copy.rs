// Verifies public API ownership, redaction, and borrowing invariants.
// Authority: approved specification requirement Rust value-model contract.
use kmipkit_ttlv::{Item, Structure, StructureView, Value, ValueView};

fn requires_copy<T: Copy>() {}

fn main() {
    requires_copy::<Value>();
    requires_copy::<ValueView<'static>>();
    requires_copy::<StructureView<'static>>();
    requires_copy::<Item>();
    requires_copy::<Structure>();
}
