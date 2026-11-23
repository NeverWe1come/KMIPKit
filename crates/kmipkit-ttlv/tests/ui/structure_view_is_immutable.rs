// Verifies public API ownership, redaction, and borrowing invariants.
use kmipkit_ttlv::{Item, Structure};

fn main() {
    let structure = Structure::new();
    let view = structure.view();
    let _mutable_children: &mut [Item] = view.children();
}
