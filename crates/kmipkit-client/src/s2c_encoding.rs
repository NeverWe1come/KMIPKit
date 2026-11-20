//! Private TTLV encoding shared by S2C responses and typed binding adapters.
//!
//! This preserves the existing S2C representation and is separate from the
//! validated, bounded request writer owned by `Client::execute`.

use kmipkit_ttlv::{Item, StructureView};

pub(crate) fn encode_structure_view(tag: u32, view: &StructureView<'_>, out: &mut Vec<u8>) {
    let raw_tag = tag.to_be_bytes();
    let tag_bytes = [raw_tag[1], raw_tag[2], raw_tag[3]];
    let mut children_bytes = zeroize::Zeroizing::new(Vec::new());
    for child in view.children() {
        encode_ttlv_item(child, &mut children_bytes);
    }
    write_ttlv_header(out, tag_bytes, 0x01, children_bytes.len());
    out.extend_from_slice(&children_bytes);
}

pub(crate) fn encode_ttlv_item(item: &Item, out: &mut Vec<u8>) {
    let raw_tag = item.tag().raw().to_be_bytes();
    let tag_bytes = [raw_tag[1], raw_tag[2], raw_tag[3]];

    item.with_value(|view| match view {
        kmipkit_ttlv::ValueView::Structure(structure) => {
            let mut children_bytes = zeroize::Zeroizing::new(Vec::new());
            for child in structure.children() {
                encode_ttlv_item(child, &mut children_bytes);
            }
            write_ttlv_header(out, tag_bytes, 0x01, children_bytes.len());
            out.extend_from_slice(&children_bytes);
        }
        kmipkit_ttlv::ValueView::Integer(val) => {
            write_ttlv_header(out, tag_bytes, 0x02, 4);
            out.extend_from_slice(&val.to_be_bytes());
            out.extend_from_slice(&[0u8; 4]);
        }
        kmipkit_ttlv::ValueView::LongInteger(val) => {
            write_ttlv_header(out, tag_bytes, 0x03, 8);
            out.extend_from_slice(&val.to_be_bytes());
        }
        kmipkit_ttlv::ValueView::BigInteger(bytes) => {
            write_ttlv_header(out, tag_bytes, 0x04, bytes.len());
            out.extend_from_slice(bytes);
            write_ttlv_padding(out, bytes.len());
        }
        kmipkit_ttlv::ValueView::Enumeration(val) => {
            write_ttlv_header(out, tag_bytes, 0x05, 4);
            out.extend_from_slice(&val.to_be_bytes());
            out.extend_from_slice(&[0u8; 4]);
        }
        kmipkit_ttlv::ValueView::Boolean(val) => {
            write_ttlv_header(out, tag_bytes, 0x06, 8);
            let byte_val = u8::from(*val);
            out.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, byte_val]);
        }
        kmipkit_ttlv::ValueView::TextString(s) => {
            let bytes = s.as_bytes();
            write_ttlv_header(out, tag_bytes, 0x07, bytes.len());
            out.extend_from_slice(bytes);
            write_ttlv_padding(out, bytes.len());
        }
        kmipkit_ttlv::ValueView::ByteString(bytes) => {
            write_ttlv_header(out, tag_bytes, 0x08, bytes.len());
            out.extend_from_slice(bytes);
            write_ttlv_padding(out, bytes.len());
        }
        kmipkit_ttlv::ValueView::DateTime(dt) => {
            write_ttlv_header(out, tag_bytes, 0x09, 8);
            out.extend_from_slice(&dt.to_be_bytes());
        }
        kmipkit_ttlv::ValueView::Interval(val) => {
            write_ttlv_header(out, tag_bytes, 0x0A, 4);
            out.extend_from_slice(&val.to_be_bytes());
            out.extend_from_slice(&[0u8; 4]);
        }
        kmipkit_ttlv::ValueView::DateTimeExtended(dt) => {
            write_ttlv_header(out, tag_bytes, 0x0B, 8);
            out.extend_from_slice(&dt.to_be_bytes());
        }
        kmipkit_ttlv::ValueView::Identifier(s) => {
            let bytes = s.as_bytes();
            write_ttlv_header(out, tag_bytes, 0x0C, bytes.len());
            out.extend_from_slice(bytes);
            write_ttlv_padding(out, bytes.len());
        }
        kmipkit_ttlv::ValueView::Reference(s) => {
            let bytes = s.as_bytes();
            write_ttlv_header(out, tag_bytes, 0x0D, bytes.len());
            out.extend_from_slice(bytes);
            write_ttlv_padding(out, bytes.len());
        }
        kmipkit_ttlv::ValueView::NameReference(s) => {
            let bytes = s.as_bytes();
            write_ttlv_header(out, tag_bytes, 0x0E, bytes.len());
            out.extend_from_slice(bytes);
            write_ttlv_padding(out, bytes.len());
        }
        _ => {}
    });
}

fn write_ttlv_header(out: &mut Vec<u8>, tag_bytes: [u8; 3], item_type: u8, len: usize) {
    out.extend_from_slice(&tag_bytes);
    out.push(item_type);
    out.extend_from_slice(&u32::try_from(len).unwrap_or(0).to_be_bytes());
}

fn write_ttlv_padding(out: &mut Vec<u8>, len: usize) {
    let rem = len % 8;
    if rem > 0 {
        let pad = 8 - rem;
        out.extend(std::iter::repeat_n(0u8, pad));
    }
}
