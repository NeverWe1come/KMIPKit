use kmipkit_ttlv::StructureView;

impl ClientErrorResponseTtlv {
    pub fn with_ttlv<R>(
        &self,
        callback: impl for<'a> FnOnce(StructureView<'a>) -> R,
    ) -> R {
        callback(self)
    }
}
