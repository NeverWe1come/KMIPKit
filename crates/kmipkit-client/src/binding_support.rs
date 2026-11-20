//! Typed payload encoding for KMIPKit's synchronous native callbacks.
//!
//! These Rust entry points are implementation support for the bindings. They
//! reuse the S2C response encoder without exposing a generic TTLV encoder or
//! providing a route to submit unvalidated requests through `Client::execute`.
//! Returned buffers must remain alive until the callback returns. They clear
//! their current contents on drop; foreign copies remain caller owned.

use kmipkit_protocol::{ServerNotifyRequest, ServerPutRequest};
use kmipkit_ttlv::Structure;
use zeroize::Zeroizing;

use crate::s2c_encoding::{encode_structure_view, encode_ttlv_item};

/// Encodes the complete managed-object Item passed to a Put callback.
#[must_use]
pub fn put_object_ttlv(request: &ServerPutRequest) -> Zeroizing<Vec<u8>> {
    let mut bytes = Zeroizing::new(Vec::new());
    encode_ttlv_item(request.object_item(), &mut bytes);
    bytes
}

/// Encodes Put attributes, preserving absent versus present-empty Structures.
#[must_use]
pub fn put_attributes_ttlv(request: &ServerPutRequest) -> Option<Zeroizing<Vec<u8>>> {
    encode_attributes(request.attributes())
}

/// Encodes Notify attributes, preserving absent versus present-empty Structures.
#[must_use]
pub fn notify_attributes_ttlv(request: &ServerNotifyRequest) -> Option<Zeroizing<Vec<u8>>> {
    encode_attributes(request.attributes())
}

fn encode_attributes(attributes: Option<&Structure>) -> Option<Zeroizing<Vec<u8>>> {
    attributes.map(|attributes| {
        let mut bytes = Zeroizing::new(Vec::new());
        encode_structure_view(0x0042_0125, &attributes.view(), &mut bytes);
        bytes
    })
}

/// Encodes deleted Notify references inside the callback's Request Payload wrapper.
#[must_use]
pub fn notify_deleted_attributes_ttlv(request: &ServerNotifyRequest) -> Zeroizing<Vec<u8>> {
    let mut bytes = Zeroizing::new(Vec::new());
    // The historical callback format includes this wrapper even when empty.
    bytes.extend_from_slice(&[0x42, 0x00, 0x79, 0x01, 0, 0, 0, 0]);
    for reference in request.deleted_attribute_references() {
        encode_structure_view(0x0042_013B, &reference.view(), &mut bytes);
    }
    let length = u32::try_from(bytes.len() - 8).unwrap_or(0);
    bytes[4..8].copy_from_slice(&length.to_be_bytes());
    bytes
}
