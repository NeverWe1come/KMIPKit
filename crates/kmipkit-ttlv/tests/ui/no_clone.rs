// Requirement traceability: KMIPKIT-0004-FR-012, SC-006.
// Authority: approved KMIPKIT-0004 Rust value-model contract.
use kmipkit_ttlv::{Item, Structure, StructureView, Value, ValueView};

fn requires_clone<T: Clone>() {}

fn main() {
    requires_clone::<Value>();
    requires_clone::<ValueView<'static>>();
    requires_clone::<StructureView<'static>>();
    requires_clone::<Item>();
    requires_clone::<Structure>();
}
