struct ClientBatchItem;

impl ClientBatchItem {
    pub fn with_view(&self, callback: impl for<'a> FnOnce(kmipkit_ttlv::StructureView<'a>)) {}
}
