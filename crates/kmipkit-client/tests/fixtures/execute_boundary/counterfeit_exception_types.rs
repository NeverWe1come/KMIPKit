mod counterfeit {
    struct ClientBatchItem;

    impl ClientBatchItem {
        pub fn with_unique_batch_item_id(self, id: Vec<u8>) -> Self {
            self
        }
    }

    struct ClientMessageExtension;

    impl ClientMessageExtension {
        pub fn with_ttlv<R>(
            &self,
            callback: impl for<'a> FnOnce(StructureView<'a>) -> R,
        ) -> R {
            callback(StructureView::new())
        }
    }
}
