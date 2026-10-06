struct ClientBatch;
struct ClientBatchItem;

impl ClientBatch {
    pub fn from_items(items: impl IntoIterator<Item = ClientBatchItem>) -> Self {
        Self
    }
}
