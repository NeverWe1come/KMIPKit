struct ClientMessageExtension;

impl ClientMessageExtension {
    pub fn with_ttlv<R>(
        &self,
        callback: impl for<'a> FnOnce(StructureView<'a>) -> R,
    ) -> R {
    }
}
