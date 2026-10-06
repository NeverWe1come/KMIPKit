impl Client {
    fn execute(&mut self, request: TypedRequest) -> Result<Response, Error> {
        let permit = OperationEncodingPermit::mint();
        self.writer.encode(request, permit)?;
        self.transport.exchange(self.writer.bytes())
    }

    fn helper(&self) {
        let _second_permit = <OperationEncodingPermit>::mint();
    }
}
