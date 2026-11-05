//! Private KMIP 2.1 XML item codec (Profiles v2.1 §5.4.1).
use kmipkit_ttlv::Item;
use kmipkit_ttlv::codec::CodecLimits;
use zeroize::Zeroizing;
use crate::wire_codec_common::{WireCodecError, WireCodecErrorKind};

pub(crate) fn encode(_item: &Item, _limits: &CodecLimits) -> Result<Zeroizing<Vec<u8>>, WireCodecError> {
    Err(WireCodecError::new(WireCodecErrorKind::InvalidDocument))
}

pub(crate) fn decode(_bytes: &[u8], _limits: &CodecLimits) -> Result<Item, WireCodecError> {
    Err(WireCodecError::new(WireCodecErrorKind::InvalidDocument))
}

#[cfg(test)]
#[path = "../tests/unit/xml_codec_tests.rs"]
mod tests;
