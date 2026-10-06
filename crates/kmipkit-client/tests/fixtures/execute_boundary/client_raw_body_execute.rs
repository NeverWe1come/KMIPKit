impl Client {
    pub fn execute_raw(&mut self, caller_body: &[u8]) {
        self.transport.exchange(caller_body);
    }
}
