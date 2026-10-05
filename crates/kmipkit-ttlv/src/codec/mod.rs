//! Bounded decoding for one complete generic TTLV item.
//!
//! The decoder preserves Structure child order and represented values while
//! rejecting malformed framing, unsupported Item Types, and Tags that cannot
//! enter the checked generic model.

mod decoder;

pub use decoder::{DecodeError, DecodeErrorKind, decode};
