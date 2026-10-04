// Requirement traceability: KMIPKIT-0004-FR-012, SC-006.
// Authority: approved KMIPKIT-0004 Rust value-model contract.
use kmipkit_ttlv::{Item, Structure, StructureView, Value, ValueView};

fn requires_copy<T: Copy>() {}

fn main() {
    requires_copy::<Value>();
    requires_copy::<ValueView<'static>>();
    requires_copy::<StructureView<'static>>();
    requires_copy::<Item>();
    requires_copy::<Structure>();
}
