impl Client {
    fn execute(&mut self, request: TypedRequest) {
        self.writer.encode(request);
    }
}
