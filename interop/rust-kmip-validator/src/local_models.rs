//! Explicit local parameter model calls, OASIS KMIP 2.1 §4.16, Table 59.
use crate::{driver::item, models::Result};
use kmipkit::protocol::{CryptographicParameters, CryptographicParametersBuilder};
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

pub(crate) fn invoke(symbol: &str, sample: &Item) -> Option<Result<Item>> {
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
