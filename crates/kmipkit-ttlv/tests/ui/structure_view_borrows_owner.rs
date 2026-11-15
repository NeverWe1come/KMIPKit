// Requirement traceability: KMIPKIT-0006-FR-017, SC-006.
use kmipkit_ttlv::Structure;

fn main() {
    let view = {
        let structure = Structure::new();
        structure.view()
    };
    let _ = view.children();
}
