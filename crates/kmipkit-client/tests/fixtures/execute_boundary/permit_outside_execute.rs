fn prepare_permit() -> OperationEncodingPermit {
    OperationEncodingPermit::mint()
}

impl Client {
    fn execute(&mut self, request: TypedRequest) {
        self.writer.encode(request, prepare_permit());
    }
}
