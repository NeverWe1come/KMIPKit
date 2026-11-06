//! TTLV encoding and decoding for kmipkit-test-server.

use kmipkit_ttlv::codec::{DecodeError, decode};
use kmipkit_ttlv::{Item, ValueView};

/// Decodes raw TTLV bytes into a generic KMIP [`Item`].
pub fn decode_ttlv(bytes: &[u8]) -> Result<Item, DecodeError> {
    decode(bytes)
}

/// Encodes a generic KMIP [`Item`] into standard OASIS TTLV bytes.
pub fn encode_ttlv(item: &Item) -> Vec<u8> {
    let mut out = Vec::new();
    encode_item_into(item, &mut out);
    out
}

fn encode_item_into(item: &Item, out: &mut Vec<u8>) {
    let raw_tag = item.tag().raw().to_be_bytes();
    let tag_bytes = [raw_tag[1], raw_tag[2], raw_tag[3]];

    item.with_value(|view| match view {
        ValueView::Structure(structure) => {
            let mut children_bytes = Vec::new();
            for child in structure.children() {
                encode_item_into(child, &mut children_bytes);
            }
            write_header(out, tag_bytes, 0x01, children_bytes.len());
            out.extend_from_slice(&children_bytes);
            // Structures do not need padding because children are already padded to 8 bytes.
        }
        ValueView::Integer(val) => {
            write_header(out, tag_bytes, 0x02, 4);
            out.extend_from_slice(&val.to_be_bytes());
            out.extend_from_slice(&[0u8; 4]); // 4 bytes padding
        }
        ValueView::LongInteger(val) => {
            write_header(out, tag_bytes, 0x03, 8);
            out.extend_from_slice(&val.to_be_bytes());
        }
        ValueView::BigInteger(bytes) => {
            write_header(out, tag_bytes, 0x04, bytes.len());
            out.extend_from_slice(bytes);
            write_padding(out, bytes.len());
        }
        ValueView::Enumeration(val) => {
            write_header(out, tag_bytes, 0x05, 4);
            out.extend_from_slice(&val.to_be_bytes());
            out.extend_from_slice(&[0u8; 4]); // 4 bytes padding
        }
        ValueView::Boolean(val) => {
            write_header(out, tag_bytes, 0x06, 8);
            let bool_val: u64 = if *val { 1 } else { 0 };
            out.extend_from_slice(&bool_val.to_be_bytes());
        }
        ValueView::TextString(text) => {
            let bytes = text.as_bytes();
            write_header(out, tag_bytes, 0x07, bytes.len());
            out.extend_from_slice(bytes);
            write_padding(out, bytes.len());
        }
        ValueView::ByteString(bytes) => {
            write_header(out, tag_bytes, 0x08, bytes.len());
            out.extend_from_slice(bytes);
            write_padding(out, bytes.len());
        }
        ValueView::DateTime(val) => {
            write_header(out, tag_bytes, 0x09, 8);
            out.extend_from_slice(&val.to_be_bytes());
        }
        ValueView::Interval(val) => {
            write_header(out, tag_bytes, 0x0A, 4);
            out.extend_from_slice(&val.to_be_bytes());
            out.extend_from_slice(&[0u8; 4]); // 4 bytes padding
        }
        ValueView::DateTimeExtended(val) => {
            write_header(out, tag_bytes, 0x0B, 8);
            out.extend_from_slice(&val.to_be_bytes());
        }
        _ => {}
    });
}

fn write_header(out: &mut Vec<u8>, tag_bytes: [u8; 3], type_byte: u8, len: usize) {
    out.extend_from_slice(&tag_bytes);
    out.push(type_byte);
    let len_u32 = u32::try_from(len).unwrap_or(u32::MAX);
    out.extend_from_slice(&len_u32.to_be_bytes());
}

fn write_padding(out: &mut Vec<u8>, len: usize) {
    let remainder = len % 8;
    if remainder != 0 {
        let pad = 8 - remainder;
        out.resize(out.len() + pad, 0);
    }
}
