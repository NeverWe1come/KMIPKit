struct ClientOperationOutcome;

impl ClientOperationOutcome {
    pub fn with_asynchronous_correlation_value<R>(
        &self,
        callback: impl for<'a> FnOnce(&'a [u8]) -> R,
    ) -> Option<R> {
        None
    }

    pub fn with_cancel_echo<R>(
        &self,
        callback: impl for<'a> FnOnce(&'a [u8]) -> R,
    ) -> Option<R> {
        None
    }

    pub fn with_response_payload<R>(
        &self,
        callback: impl for<'a> FnOnce(StructureView<'a>) -> R,
    ) -> Option<R> {
        None
    }
}
