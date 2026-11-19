struct OperationEncodingPermit {
    _private: (),
}

impl OperationEncodingPermit {
    fn mint() -> Self {
        Self { _private: () }
    }

    fn helper() -> Self {
        Self { _private: () }
    }
}
