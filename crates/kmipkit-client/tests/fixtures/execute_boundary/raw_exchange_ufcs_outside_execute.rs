impl Client {
    fn execute(&mut self, request: TypedRequest) -> Result<Response, Error> {
        let permit = OperationEncodingPermit::mint();
        self.writer.encode(request, permit)?;
        self.transport.exchange(self.writer.bytes())
    }

    fn helper(&mut self, request: &[u8], max_response_bytes: usize) {
        let _ = Transport::exchange(&mut self.transport, request, max_response_bytes);
    }
}
