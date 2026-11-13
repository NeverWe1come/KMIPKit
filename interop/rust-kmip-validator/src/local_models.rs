//! Explicit local parameter model calls, OASIS KMIP 2.1 §4.16, Table 59.
use crate::{driver::item, models::Result};
use kmipkit::protocol::{
    CheckRequest, CryptographicParameters, CryptographicParametersBuilder, KeyBlock, KeyFormatType,
    QueryResponse, ResponseMessage,
};
use kmipkit::ttlv::{Item, Value, ValueView};

fn scalar_u32(sample: &Item) -> Result<u32> {
    sample.with_value(|v| match v {
        ValueView::Enumeration(v) => Ok(*v),
        _ => Err("parameter fixture type mismatch"),
    })
}
fn scalar_i32(sample: &Item) -> Result<i32> {
    sample.with_value(|v| match v {
        ValueView::Integer(v) => Ok(*v),
        _ => Err("parameter fixture type mismatch"),
    })
}

fn push(structure: &mut kmipkit::ttlv::Structure, child: Item) -> Result<()> {
    structure
        .try_push(child)
        .map_err(|_| "response fixture structure rejected")
}

fn query_response_payload(sample: &Item, protocol_version: &str) -> Result<Item> {
    // OASIS KMIP 2.0 §6.1.37, Table 260; KMIP 2.1 §6.1.40, Table 283.
    let (major, minor) = match protocol_version {
        "2.0" => (2, 0),
        "2.1" => (2, 1),
        _ => return Err("unsupported Query response fixture version"),
    };
    let mut version = kmipkit::ttlv::Structure::new();
    push(&mut version, item(0x0042_006a, Value::integer(major))?)?;
    push(&mut version, item(0x0042_006b, Value::integer(minor))?)?;

    let mut header = kmipkit::ttlv::Structure::new();
    push(&mut header, item(0x0042_0069, Value::structure(version))?)?;
    push(&mut header, item(0x0042_0092, Value::date_time(1))?)?;
    push(&mut header, item(0x0042_000d, Value::integer(1))?)?;

    let mut payload = kmipkit::ttlv::Structure::new();
    push(
        &mut payload,
        sample
            .try_clone()
            .map_err(|_| "response fixture copy failed")?,
    )?;
    if sample.tag().raw() != 0x0042_015f {
        push(
            &mut payload,
            item(
                0x0042_015f,
                Value::structure(kmipkit::ttlv::Structure::new()),
            )?,
        )?;
    }
    let mut batch = kmipkit::ttlv::Structure::new();
    push(&mut batch, item(0x0042_005c, Value::enumeration(0x18))?)?;
    push(&mut batch, item(0x0042_007f, Value::enumeration(0))?)?;
    push(&mut batch, item(0x0042_007c, Value::structure(payload))?)?;

    let mut message = kmipkit::ttlv::Structure::new();
    push(&mut message, item(0x0042_007a, Value::structure(header))?)?;
    push(&mut message, item(0x0042_000f, Value::structure(batch))?)?;
    let response = ResponseMessage::try_from_ttlv(message)
        .map_err(|_| "response fixture validation rejected")?;
    let batch_item = response
        .batch_items()
        .next()
        .ok_or("response fixture has no batch item")?;
    let query = QueryResponse::try_from_response_item(batch_item)
        .map_err(|_| "Query response parser rejected fixture")?;
    item(
        0x0042_007c,
        Value::structure(
            query
                .to_ttlv_payload()
                .map_err(|_| "Query response payload accessor failed")?,
        ),
    )
}

fn keyblock_cryptographic_properties(sample: &Item) -> Result<Item> {
    // OASIS KMIP 2.0 §3.1, Table 12; KMIP 2.1 §3.1, Table 12.
    let (algorithm, length) = match sample.tag().raw() {
        0x0042_0028 => (scalar_u32(sample)?, 256),
        0x0042_002a => (3, scalar_i32(sample)?),
        _ => return Err("KeyBlock cryptographic property target is unsupported"),
    };
    KeyBlock::try_new(KeyFormatType::from_raw(1))
        .map_err(|_| "KeyBlock fixture rejected")?
        .with_cryptographic_properties(Some(algorithm), Some(length))
        .try_into_item()
        .map_err(|_| "KeyBlock cryptographic properties rejected")
}

fn check_usage_mask(sample: &Item) -> Result<Item> {
    // OASIS KMIP 2.0 §6.1.7, Table 183; KMIP 2.1 §6.1.7, Table 183.
    let mask = scalar_i32(sample)?;
    let payload = CheckRequest::new()
        .with_cryptographic_usage_mask(mask)
        .into_ttlv_payload()
        .map_err(|_| "Check request payload construction failed")?;
    item(0x0042_0079, Value::structure(payload))
}

fn method(tag: u32) -> Option<&'static str> {
    Some(match tag {
        0x0042_0011 => "with_block_cipher_mode",
        0x0042_005f => "with_padding_method",
        0x0042_0038 => "with_hashing_algorithm",
        0x0042_0083 => "with_key_role_type",
        0x0042_00ae => "with_digital_signature_algorithm",
        0x0042_0028 => "with_cryptographic_algorithm",
        0x0042_00c5 => "with_random_iv",
        0x0042_00cd => "with_iv_length",
        0x0042_00ce => "with_tag_length",
        0x0042_00cf => "with_fixed_field_length",
        0x0042_00d2 => "with_invocation_field_length",
        0x0042_00d0 => "with_counter_length",
        0x0042_00d1 => "with_initial_counter_value",
        0x0042_0100 => "with_salt_length",
        0x0042_0101 => "with_mask_generator",
        0x0042_0102 => "with_mask_generator_hashing_algorithm",
        0x0042_0103 => "with_p_source",
        0x0042_0104 => "with_trailer_field",
        _ => return None,
    })
}

#[allow(clippy::too_many_lines)] // Explicit setters demonstrate the installed public calls.
fn builder(sample: &Item) -> Result<CryptographicParametersBuilder> {
    let builder = CryptographicParametersBuilder::new();
    Ok(match sample.tag().raw() {
        0x0042_002b
            if sample
                .as_structure()
                .is_some_and(|s| s.children().is_empty()) =>
        {
            builder
        }
        0x0042_0011 => {
            let mode = scalar_u32(sample)?;
            let builder = builder.with_block_cipher_mode(mode);
            let builder = if [6, 8, 9].contains(&mode) {
                builder.with_iv_length(128)
            } else {
                builder
            };
            if mode == 9 {
                builder.with_tag_length(128)
            } else {
                builder
            }
        }
        0x0042_005f => builder.with_padding_method(scalar_u32(sample)?),
        0x0042_0038 => builder.with_hashing_algorithm(scalar_u32(sample)?),
        0x0042_0083 => builder.with_key_role_type(scalar_u32(sample)?),
        0x0042_00ae => builder.with_digital_signature_algorithm(scalar_u32(sample)?),
        0x0042_0028 => builder.with_cryptographic_algorithm(scalar_u32(sample)?),
        0x0042_00c5 => builder.with_random_iv(sample.with_value(|v| match v {
            ValueView::Boolean(v) => Ok(*v),
            _ => Err("parameter fixture type mismatch"),
        })?),
        0x0042_00cd => builder.with_iv_length(scalar_i32(sample)?),
        0x0042_00ce => builder.with_tag_length(scalar_i32(sample)?),
        0x0042_00cf => builder.with_fixed_field_length(scalar_i32(sample)?),
        0x0042_00d2 => builder.with_invocation_field_length(scalar_i32(sample)?),
        0x0042_00d0 => builder.with_counter_length(scalar_i32(sample)?),
        0x0042_00d1 => builder.with_initial_counter_value(scalar_i32(sample)?),
        0x0042_0100 => builder.with_salt_length(scalar_i32(sample)?),
        0x0042_0101 => builder.with_mask_generator(scalar_u32(sample)?),
        0x0042_0102 => builder.with_mask_generator_hashing_algorithm(scalar_u32(sample)?),
        0x0042_0103 => builder.with_p_source(sample.with_value(|v| match v {
            ValueView::ByteString(v) => Ok(v.to_vec()),
            _ => Err("parameter fixture type mismatch"),
        })?),
        0x0042_0104 => builder.with_trailer_field(scalar_i32(sample)?),
        _ => return Err("parameter target fixture is unfinished"),
    })
}

pub(crate) fn parameter_context(sample: &Item) -> Result<Item> {
    let structure = if sample.tag().raw() == 0x0042_002b {
        sample
            .as_structure()
            .ok_or("parameter fixture type mismatch")?
            .try_clone()
            .map_err(|_| "parameter copy failed")?
    } else {
        builder(sample)?
            .build()
            .map_err(|_| "parameter fixture rejected")?
            .into_ttlv()
    };
    item(0x0042_002b, Value::structure(structure))
}

pub(crate) fn invoke(symbol: &str, sample: &Item, protocol_version: &str) -> Option<Result<Item>> {
    if symbol == "kmipkit_protocol::CheckRequest::with_cryptographic_usage_mask" {
        return Some(check_usage_mask(sample));
    }
    if symbol == "kmipkit_protocol::KeyBlock::with_cryptographic_properties" {
        return Some(keyblock_cryptographic_properties(sample));
    }
    if symbol == "kmipkit_protocol::QueryResponse::to_ttlv_payload" {
        return Some(query_response_payload(sample, protocol_version));
    }
    if let Some(selected) =
        symbol.strip_prefix("kmipkit_protocol::CryptographicParametersBuilder::")
    {
        if !["new", "build"].contains(&selected) && Some(selected) != method(sample.tag().raw()) {
            return None;
        }
        let builder = builder(sample).ok()?;
        return Some(
            builder
                .build()
                .map_err(|_| "parameter constructor rejected")
                .and_then(|model| item(0x0042_002b, Value::structure(model.into_ttlv()))),
        );
    }
    if ![
        "kmipkit_protocol::CryptographicParameters::try_from_ttlv",
        "kmipkit_protocol::CryptographicParameters::into_ttlv",
    ]
    .contains(&symbol)
    {
        return None;
    }
    let context = parameter_context(sample).ok()?;
    let structure = context.as_structure()?.try_clone().ok()?;
    if symbol.ends_with("::try_from_ttlv") {
        Some(
            CryptographicParameters::try_from_ttlv(structure)
                .map_err(|_| "parameter constructor rejected")
                .and_then(|model| item(0x0042_002b, Value::structure(model.into_ttlv()))),
        )
    } else {
        let model = CryptographicParameters::try_from_ttlv(structure).ok()?;
        Some(item(0x0042_002b, Value::structure(model.into_ttlv())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_response_payload_accessor_preserves_operation_field() {
        let sample =
            item(0x0042_005c, Value::enumeration(0x18)).expect("Query Operation sample is valid");
        let output = query_response_payload(&sample, "2.1");
        assert!(
            output.is_ok(),
            "Query response fixture rejected: {:?}",
            output.err()
        );
        let output = output.expect("response accessor returns a payload");
        let projected = crate::local_wire::target_hash(&output, &sample);
        assert!(
            projected.is_ok(),
            "Query response payload mismatch: {:?}",
            projected.err()
        );
    }
}
