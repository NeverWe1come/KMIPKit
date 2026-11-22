use crate::{Item, StructureView, ValueView};

/// Appends one complete TTLV Item to `out`, including its header and padding.
pub fn encode_ttlv_item(item: &Item, out: &mut Vec<u8>) {
    let raw_tag = item.tag().raw().to_be_bytes();
    let tag_bytes = [raw_tag[1], raw_tag[2], raw_tag[3]];

    item.with_value(|view| match view {
        ValueView::Structure(structure) => {
            let mut children_bytes = Vec::new();
            for child in structure.children() {
                encode_ttlv_item(child, &mut children_bytes);
            }
            write_ttlv_header(out, tag_bytes, 0x01, children_bytes.len());
            out.extend_from_slice(&children_bytes);
        }
        ValueView::Integer(value) => {
            write_ttlv_header(out, tag_bytes, 0x02, 4);
            out.extend_from_slice(&value.to_be_bytes());
            out.extend_from_slice(&[0; 4]);
        }
        ValueView::LongInteger(value) => {
            write_ttlv_header(out, tag_bytes, 0x03, 8);
            out.extend_from_slice(&value.to_be_bytes());
        }
        ValueView::BigInteger(bytes) => {
            write_ttlv_header(out, tag_bytes, 0x04, bytes.len());
            out.extend_from_slice(bytes);
            write_ttlv_padding(out, bytes.len());
        }
        ValueView::Enumeration(value) => {
            write_ttlv_header(out, tag_bytes, 0x05, 4);
            out.extend_from_slice(&value.to_be_bytes());
            out.extend_from_slice(&[0; 4]);
        }
        ValueView::Boolean(value) => {
            write_ttlv_header(out, tag_bytes, 0x06, 8);
            out.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, u8::from(*value)]);
        }
        ValueView::TextString(value) => {
            let bytes = value.as_bytes();
            write_ttlv_header(out, tag_bytes, 0x07, bytes.len());
            out.extend_from_slice(bytes);
            write_ttlv_padding(out, bytes.len());
        }
        ValueView::ByteString(bytes) => {
            write_ttlv_header(out, tag_bytes, 0x08, bytes.len());
            out.extend_from_slice(bytes);
            write_ttlv_padding(out, bytes.len());
        }
        ValueView::DateTime(value) => {
            write_ttlv_header(out, tag_bytes, 0x09, 8);
            out.extend_from_slice(&value.to_be_bytes());
        }
        ValueView::Interval(value) => {
            write_ttlv_header(out, tag_bytes, 0x0A, 4);
            out.extend_from_slice(&value.to_be_bytes());
            out.extend_from_slice(&[0; 4]);
        }
        ValueView::DateTimeExtended(value) => {
            write_ttlv_header(out, tag_bytes, 0x0B, 8);
            out.extend_from_slice(&value.to_be_bytes());
        }
        ValueView::Identifier(value) => {
            let bytes = value.as_bytes();
            write_ttlv_header(out, tag_bytes, 0x0C, bytes.len());
            out.extend_from_slice(bytes);
            write_ttlv_padding(out, bytes.len());
        }
        ValueView::Reference(value) => {
            let bytes = value.as_bytes();
            write_ttlv_header(out, tag_bytes, 0x0D, bytes.len());
            out.extend_from_slice(bytes);
            write_ttlv_padding(out, bytes.len());
        }
        ValueView::NameReference(value) => {
            let bytes = value.as_bytes();
            write_ttlv_header(out, tag_bytes, 0x0E, bytes.len());
            out.extend_from_slice(bytes);
            write_ttlv_padding(out, bytes.len());
        }
    });
}

/// Appends the contents of a Structure View as a tagged TTLV Structure.
pub fn encode_ttlv_structure(tag: u32, view: &StructureView<'_>, out: &mut Vec<u8>) {
    let raw_tag = tag.to_be_bytes();
    let tag_bytes = [raw_tag[1], raw_tag[2], raw_tag[3]];
    let mut children_bytes = Vec::new();
    for child in view.children() {
        encode_ttlv_item(child, &mut children_bytes);
    }
    write_ttlv_header(out, tag_bytes, 0x01, children_bytes.len());
    out.extend_from_slice(&children_bytes);
}

fn write_ttlv_header(out: &mut Vec<u8>, tag_bytes: [u8; 3], item_type: u8, len: usize) {
    out.extend_from_slice(&tag_bytes);
    out.push(item_type);
    out.extend_from_slice(&u32::try_from(len).unwrap_or(0).to_be_bytes());
}

fn write_ttlv_padding(out: &mut Vec<u8>, len: usize) {
    let remainder = len % 8;
    if remainder > 0 {
        out.extend(std::iter::repeat_n(0, 8 - remainder));
    }
}

#[cfg(test)]
mod tests {
    use super::encode_ttlv_item;
    use crate::{Item, ItemType, RawTag, Value, ValueView};

    #[test]
    fn encoded_item_round_trips_through_the_bounded_decoder() {
        let item = Item::new(
            RawTag::new(0x0042_0173).unwrap().try_checked().unwrap(),
            Value::text_string("kmip".to_owned()),
        )
        .unwrap();
        let mut encoded = Vec::new();

        encode_ttlv_item(&item, &mut encoded);

        let decoded = crate::codec::decode(&encoded).unwrap();
        assert_eq!(decoded.item_type(), ItemType::TextString);
        assert!(
            decoded
                .with_value(|value| matches!(value, ValueView::TextString(text) if text == "kmip"))
        );
    }
}
