#[derive(Default, Clone, Copy)]
struct OperationEncodingPermit;

fn construct_another_permit() {
    let permit: OperationEncodingPermit = Default::default();
    let cloned = permit.clone();
    let copied = permit;
    let _ = (cloned, copied);
}

impl Client {
    fn execute(&mut self, request: TypedRequest) -> Result<Response, Error> {
        let permit = OperationEncodingPermit::mint();
        self.writer.encode(request, permit)?;
        self.transport.exchange(self.writer.bytes())
    }
}
