struct ClientBatchItem;

impl ClientBatchItem {
    pub fn with_unique_batch_item_id(mut self, id: Vec<u8>) -> Self {
        self
    }
}
