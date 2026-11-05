//! Private bounded KMIP 2.1 XML codec; OASIS Profiles v2.1 §5.4.1.
//!
//! Events borrow the caller's bytes. DTDs are rejected, with no entity
//! resolver, schema processor, network path, or document DOM. Attribute
//! scratch, open element metadata and output use zeroizing owners.
use crate::wire_codec_common::{WireCodecError, WireCodecErrorKind};

mod characters;
mod decoder;
mod encoder;
mod names;
mod values;

pub(crate) fn decode(
    bytes: &[u8],
    limits: &kmipkit_ttlv::codec::CodecLimits,
) -> Result<kmipkit_ttlv::Item, WireCodecError> {
    decoder::decode(bytes, limits)
}

pub(crate) fn encode(
    item: &kmipkit_ttlv::Item,
    limits: &kmipkit_ttlv::codec::CodecLimits,
) -> Result<zeroize::Zeroizing<Vec<u8>>, WireCodecError> {
    encoder::emit_document(item, limits)
}

const NAMESPACE: &str = "urn:oasis:tc:kmip:xmlns";
const INVALID_DOCUMENT: WireCodecError = WireCodecError::new(WireCodecErrorKind::InvalidDocument);
const INVALID_VALUE: WireCodecError = WireCodecError::new(WireCodecErrorKind::InvalidValue);
const LIMIT: WireCodecError = WireCodecError::new(WireCodecErrorKind::LimitExceeded);
const ALLOCATION: WireCodecError = WireCodecError::new(WireCodecErrorKind::AllocationFailed);

#[cfg(test)]
#[path = "../tests/unit/xml_codec_tests.rs"]
mod tests;
