use kmipkit_protocol::extension::{self, ExtensionSchema};
use kmipkit_ttlv::ItemType;

fn main() {
    let schema = extension::scalar(ItemType::Integer).expect("Integer is supported");
    let _: ExtensionSchema = extension::with_callback(schema, |_| true)
        .expect("data-only schemas do not have executable callbacks");
}
