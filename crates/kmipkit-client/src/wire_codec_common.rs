//! Shared payload-free private codec errors.
use std::fmt;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WireCodecErrorKind { InvalidDocument, InvalidValue, LimitExceeded, AllocationFailed }
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WireCodecError { kind: WireCodecErrorKind }
impl WireCodecError {
    pub(crate) const fn new(kind: WireCodecErrorKind) -> Self { Self { kind } }
    pub(crate) const fn kind(self) -> WireCodecErrorKind { self.kind }
}
impl fmt::Display for WireCodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "wire codec: {:?}", self.kind) }
}
impl std::error::Error for WireCodecError {}
