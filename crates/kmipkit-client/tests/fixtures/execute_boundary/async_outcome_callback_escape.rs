struct ClientOperationOutcome;

impl ClientOperationOutcome {
    pub fn correlation_bytes(&self) -> Vec<u8> {
        Vec::new()
    }
}
