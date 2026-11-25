// Verifies public API ownership, redaction, and borrowing invariants.
use kmipkit_ttlv::Structure;

fn main() {
    let view = {
        let structure = Structure::new();
        structure.view()
    };
    let _ = view.children();
}
