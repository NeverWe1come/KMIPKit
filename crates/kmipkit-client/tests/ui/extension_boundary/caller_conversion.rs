use kmipkit_client::extension_registry::ClientRequestMessageExtension;
use kmipkit_client::{ClientBatchItem, ClientRequest};

struct CallerValue;

impl From<CallerValue> for ClientRequestMessageExtension {
    fn from(_: CallerValue) -> Self {
        unreachable!("compile-fail fixture is never run")
    }
}

fn main() {
    let item = ClientBatchItem::new(ClientRequest::discover_versions());
    let _ = item.with_extension(CallerValue);
}
