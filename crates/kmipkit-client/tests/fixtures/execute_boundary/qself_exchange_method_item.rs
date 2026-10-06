trait Transport {
    fn exchange(&mut self);
}

impl Client {
    fn execute(&mut self) {
        self.transport.exchange(&[], 1);
        let dispatch = <dyn Transport>::exchange;
        let _ = dispatch;
    }
}
