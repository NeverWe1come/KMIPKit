//! Shared operation-payload extension handling for diagnostic operations.

use crate::ProtocolError;
use crate::cryptographic_operation::request_error;
use kmipkit_ttlv::{Item, Structure};

pub(crate) fn append_extensions(
    payload: &mut Structure,
    extensions: Vec<Item>,
    known_tags: &[u32],
) -> Result<(), ProtocolError> {
    for extension in extensions {
        if known_tags.contains(&extension.tag().raw()) {
            return Err(request_error());
        }
        payload.try_push(extension).map_err(|_| request_error())?;
    }
    Ok(())
}
