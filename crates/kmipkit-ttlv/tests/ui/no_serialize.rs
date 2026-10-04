// Requirement traceability: KMIPKIT-0004-FR-010, SC-004.
// Authority: approved KMIPKIT-0004 Rust value-model contract.
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
