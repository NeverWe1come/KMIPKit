impl Client {
    pub fn with_transport<T: Transport>(transport: T) -> Self {
        Self { transport }
    }
}
