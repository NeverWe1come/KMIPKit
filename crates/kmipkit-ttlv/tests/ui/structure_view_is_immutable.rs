// Requirement traceability: KMIPKIT-0006-FR-017, SC-006.
use kmipkit_ttlv::{Item, Structure};

fn main() {
    let structure = Structure::new();
    let view = structure.view();
    let _mutable_children: &mut [Item] = view.children();
}
