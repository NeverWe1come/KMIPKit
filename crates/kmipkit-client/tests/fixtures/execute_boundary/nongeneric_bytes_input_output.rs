pub struct Client;

impl Client {
    pub fn accept_bytes(&self, bytes: bytes::Bytes) {}

    pub fn accept_bytes_mut(&self, bytes: bytes::BytesMut) {}

    pub fn return_bytes(&self) -> bytes::Bytes {
        loop {}
    }

    pub fn return_bytes_mut(&self) -> bytes::BytesMut {
        loop {}
    }
}
