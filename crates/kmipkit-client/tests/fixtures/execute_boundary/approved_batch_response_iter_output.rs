pub struct ClientBatchResponse {
    items: Vec<ClientBatchItemResponse>,
}

pub struct ClientBatchItemResponse;

impl ClientBatchResponse {
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &ClientBatchItemResponse> + '_ {
        self.items.iter()
    }
}
