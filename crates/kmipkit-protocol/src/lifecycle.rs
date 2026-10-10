//! Shared TTLV mechanics for managed-object lifecycle operations.

use kmipkit_ttlv::{Item, ModelError, RawTag, Structure, StructureView, Tag, Value, ValueView};

use crate::{ProtocolCauseCategory, ProtocolError, ProtocolErrorKind, UniqueIdentifier};

const UNIQUE_IDENTIFIER: u32 = 0x0042_0094; // §11.56 Table 487.

pub(crate) fn request_payload(
    unique_identifier: Option<&UniqueIdentifier>,
) -> Result<Structure, ProtocolError> {
    let mut payload = Structure::new();
    if let Some(identifier) = unique_identifier {
        let value = match identifier {
            UniqueIdentifier::TextString(value) => Value::text_string(value.clone()),
            UniqueIdentifier::Enumeration(value) => Value::enumeration(*value),
            UniqueIdentifier::Integer(value) => Value::integer(*value),
        };
        payload.try_push(item(value)?).map_err(model_error)?;
    }
    Ok(payload)
}

pub(crate) fn successful_response_identifier(
    payload: &StructureView<'_>,
) -> Option<UniqueIdentifier> {
    let mut identifier = None;
    for field in payload.children() {
        if field.tag().raw() == UNIQUE_IDENTIFIER {
            if identifier.is_some() {
                return None;
            }
            identifier = parse_unique_identifier(field);
            identifier.as_ref()?;
        }
    }
    identifier
}

fn parse_unique_identifier(field: &Item) -> Option<UniqueIdentifier> {
    field.with_value(|value| match value {
        ValueView::TextString(value) => Some(UniqueIdentifier::TextString(value.to_owned())),
        ValueView::Enumeration(value) => Some(UniqueIdentifier::Enumeration(*value)),
        ValueView::Integer(value) => Some(UniqueIdentifier::Integer(*value)),
        _ => None,
    })
}

fn item(value: Value) -> Result<Item, ProtocolError> {
    Item::new(tag()?, value).map_err(model_error)
}

fn tag() -> Result<Tag, ProtocolError> {
    RawTag::new(UNIQUE_IDENTIFIER)
        .and_then(|raw| raw.try_checked())
        .map_err(model_error)
}

fn model_error(error: ModelError) -> ProtocolError {
    ProtocolError::new(
        ProtocolErrorKind::InvalidValue,
        ProtocolCauseCategory::InvalidValue,
        error,
    )
}
