use kmipkit_client::{ClientBatchItem, ClientRequest};
use kmipkit_ttlv::{Item, RawTag, Value};

fn main() {
    let item = ClientBatchItem::new(ClientRequest::discover_versions());
    let tag = RawTag::new(0x0054_0001)
        .expect("the test tag fits the KMIP tag width")
        .try_checked()
        .expect("the test tag uses the extension allocation");
    let arbitrary_item = Item::new(tag, Value::integer(7)).expect("the test Item is valid");
    let _ = item.with_extension(arbitrary_item);
}
