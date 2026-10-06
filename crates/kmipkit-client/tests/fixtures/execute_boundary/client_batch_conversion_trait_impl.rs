pub struct ClientBatch;

impl From<Vec<u8>> for ClientBatch {
    fn from(_: Vec<u8>) -> Self {
        Self
    }
}
