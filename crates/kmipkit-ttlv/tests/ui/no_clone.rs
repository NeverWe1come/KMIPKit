// Verifies public API ownership, redaction, and borrowing invariants.
// Enforces non-Clone semantics for StructureView to ensure immutable borrow guarantees.
use kmipkit_ttlv::{Item, Structure, StructureView, Value, ValueView};

fn requires_clone<T: Clone>() {}

fn main() {
    requires_clone::<Value>();
    requires_clone::<ValueView<'static>>();
    requires_clone::<StructureView<'static>>();
    requires_clone::<Item>();
    requires_clone::<Structure>();
}
