impl Client {
    fn execute(&mut self, request: TypedRequest) {
        let permit = OperationEncodingPermit::mint();
        self.writer.encode(request, permit);
        self.writer.encode(request, permit);
    }
}
