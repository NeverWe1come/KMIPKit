impl Client {
    fn execute(&mut self, request: TypedRequest) {
        let first = OperationEncodingPermit::mint();
        let second = OperationEncodingPermit::mint();
        self.writer.encode(request, first);
        self.writer.encode(request, second);
    }
}
