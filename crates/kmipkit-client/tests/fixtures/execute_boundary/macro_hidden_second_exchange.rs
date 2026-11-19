impl Client {
    fn execute(&mut self, request: TypedRequest) -> Result<Response, Error> {
        let permit = OperationEncodingPermit::mint();
        self.writer.encode(request, permit)?;
        let response = self.transport.exchange(&[], 1)?;
        let _hidden_second_exchange = vec![self.transport.exchange(&[], 1)];
        Ok(response)
    }
}
