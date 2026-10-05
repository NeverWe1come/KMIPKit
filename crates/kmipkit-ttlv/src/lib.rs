//! In-memory, typed representations of generic KMIP TTLV item values.
//!
//! This crate checks 24-bit raw tag representation, tag allocation, the
//! relationship between a value representation and its Item Type, and the
//! local maximum Structure depth of 64. It retains Structure child order and
//! exposes payloads through borrowed callbacks. It does not encode or decode
//! TTLV, validate TTLV framing or padding, validate schema-specific field
//! order or cardinality, or establish extension or operation semantics. A
//! value in this model is not thereby declared wire-valid or protocol-valid.
//! Owned payloads are held in private zeroizing allocations and are cleared
//! when dropped.
//!
//! # Example
//!
//! ```
//! use kmipkit_ttlv::{Item, ItemType, RawTag, Value, ValueView};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let raw_tag = RawTag::new(0x0042_0173)?;
//! let tag = raw_tag.try_checked()?;
//! let item = Item::new(tag, Value::integer(42))?;
//!
//! assert_eq!(item.item_type(), ItemType::Integer);
//! let is_integer_42 = item.with_value(|view| {
//!     matches!(view, ValueView::Integer(value) if *value == 42)
//! });
//! assert!(is_integer_42);
//! # Ok(())
//! # }
//! ```
#![forbid(unsafe_code)]

mod error;
mod item;
mod structure;
mod tag;
mod value;

mod generated;

#[cfg(test)]
mod codec_red_tests;

pub use error::ModelError;
pub use item::Item;
pub use structure::{Structure, StructureView};
pub use tag::{RawTag, Tag};
pub use value::{ItemType, Value, ValueView};
