use kmipkit_client::{ClientBatchItem, ClientRequest};

fn main() {
    let item = ClientBatchItem::new(ClientRequest::discover_versions());
    let raw_body = vec![0x42, 0x00, 0x01, 0x01, 0, 0, 0, 0];
    let _ = item.with_extension(raw_body);
}
