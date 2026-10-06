struct ClientBatchItem;

impl ClientBatchItem {
    pub fn with_bytes(mut self, body: Vec<u8>) -> Self {
        self
    }
}
