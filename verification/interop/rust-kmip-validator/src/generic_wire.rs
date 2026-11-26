use kmipkit::ttlv::{Item, ValueView};

// Encode the native generic tree for the exchange recipes.
pub(crate) fn encode(item: &Item, output: &mut Vec<u8>) -> Result<(), &'static str> {
    let start = output.len();
    output.extend_from_slice(&item.tag().raw().to_be_bytes()[1..]);
    output.extend_from_slice(&[0; 5]);
    let kind = item.with_value(|value| -> Result<u8, &'static str> {
        Ok(match value {
            ValueView::Structure(value) => {
                for field in value.children() {
                    encode(field, output)?;
                }
                1
            }
            ValueView::Integer(value) => {
                output.extend_from_slice(&value.to_be_bytes());
                2
            }
            ValueView::LongInteger(value) => {
                output.extend_from_slice(&value.to_be_bytes());
                3
            }
            ValueView::BigInteger(value) if !value.is_empty() && value.len() % 8 == 0 => {
                output.extend_from_slice(value);
                4
            }
            ValueView::Enumeration(value) => {
                output.extend_from_slice(&value.to_be_bytes());
                5
            }
            ValueView::Boolean(value) => {
                output.extend_from_slice(&u64::from(*value).to_be_bytes());
                6
            }
            ValueView::TextString(value) => {
                output.extend_from_slice(value.as_bytes());
                7
            }
            ValueView::ByteString(value) => {
                output.extend_from_slice(value);
                8
            }
            ValueView::DateTime(value) => {
                output.extend_from_slice(&value.to_be_bytes());
                9
            }
            ValueView::Interval(value) => {
                output.extend_from_slice(&value.to_be_bytes());
                10
            }
            ValueView::DateTimeExtended(value) => {
                output.extend_from_slice(&value.to_be_bytes());
                11
            }
            _ => return Err("fixture contains an unsupported KMIP 2.x value"),
        })
    })?;
    let length = u32::try_from(output.len() - start - 8).map_err(|_| "fixture too large")?;
    output[start + 3] = kind;
    output[start + 4..start + 8].copy_from_slice(&length.to_be_bytes());
    output.resize(output.len().next_multiple_of(8), 0);
    Ok(())
}

pub(crate) fn decode_hex(input: &str) -> Result<Vec<u8>, &'static str> {
    if !input.len().is_multiple_of(2) {
        return Err("fixture hex length is not even");
    }
    (0..input.len())
        .step_by(2)
        .map(|index| {
            u8::from_str_radix(
                input.get(index..index + 2).ok_or("invalid fixture hex")?,
                16,
            )
            .map_err(|_| "invalid fixture hex")
        })
        .collect()
}
