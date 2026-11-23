// Requirement traceability: functional requirement, SC-006.
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
