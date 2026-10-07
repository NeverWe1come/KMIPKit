use std::mem::size_of;

use kmipkit_protocol::extension::ExtensionSchema;

#[test]
fn extension_schema_clone_handle_stays_pointer_sized() {
    assert!(
        size_of::<ExtensionSchema>() <= 2 * size_of::<usize>(),
        "schema composition clones must copy a shared handle, not recursively owned trees"
    );
}