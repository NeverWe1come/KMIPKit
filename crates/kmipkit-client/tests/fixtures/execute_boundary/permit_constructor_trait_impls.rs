struct OperationEncodingPermit;

impl Default for OperationEncodingPermit {
    fn default() -> Self {
        Self::mint()
    }
}

impl Clone for OperationEncodingPermit {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl Copy for OperationEncodingPermit {}

fn construct_another_permit() {
    let _permit: OperationEncodingPermit = Default::default();
}

impl Client {
    fn execute(&mut self, request: TypedRequest) -> Result<Response, Error> {
        let permit = OperationEncodingPermit::mint();
        self.writer.encode(request, permit)?;
        self.transport.exchange(self.writer.bytes())
    }
}
