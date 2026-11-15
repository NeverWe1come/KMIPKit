//! Payload-free errors shared by the private JSON and XML codecs.
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WireCodecErrorKind {
    InvalidDocument,
    InvalidValue,
    LimitExceeded,
    AllocationFailed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WireCodecError {
    kind: WireCodecErrorKind,
}

impl WireCodecError {
    pub const fn new(kind: WireCodecErrorKind) -> Self {
        Self { kind }
    }

    pub const fn kind(self) -> WireCodecErrorKind {
        self.kind
    }
}

impl fmt::Display for WireCodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "wire codec error: {:?}", self.kind)
    }
}

impl std::error::Error for WireCodecError {}
