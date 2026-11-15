impl Client {
    fn execute(&mut self) {
        self.transport.exchange(&[], 1);
        let _ = matches!(hidden!(), _);
    }
}
