//! Private KMIP JSON encoding (OASIS KMIP Profiles v2.1 section 5.5).
use kmipkit_ttlv::{Item, codec::CodecLimits};
use zeroize::Zeroizing;
use crate::wire_codec_common::{WireCodecError, WireCodecErrorKind};
pub(crate) fn encode(_: &Item, _: &CodecLimits) -> Result<Zeroizing<Vec<u8>>, WireCodecError> { Err(WireCodecError::new(WireCodecErrorKind::InvalidDocument)) }
pub(crate) fn decode(_: &[u8], _: &CodecLimits) -> Result<Item, WireCodecError> { Err(WireCodecError::new(WireCodecErrorKind::InvalidDocument)) }
#[cfg(test)]
#[path = "../tests/unit/json_encoding_tests.rs"]
mod tests;
