use crate::generic_wire;
use kmipkit::protocol::{
    Certificate, KeyBlock, ManagedObject, ManagedObjectError, ObjectType, OpaqueObject, PgpKey,
    PrivateKey, PublicKey, SecretData, SplitKey, SplitKeyMethod, SymmetricKey,
};
use kmipkit::ttlv::{Item, Structure, ValueView};

pub(crate) fn check(
    code: u32,
    fields: &Structure,
    item: Item,
    expected: &[u8],
) -> (&'static str, &'static str) {
    let object_type = ObjectType::from_raw(code);
    let construct = if code == 6 {
        let unknown = item
            .try_clone()
            .map_err(|_| ManagedObjectError::TtlvModelLimit)
            .and_then(|item| ManagedObject::unknown(object_type, item));
        verdict(unknown, object_type, expected)
    } else {
        rebuild(code, fields.view().children()).map_or("not-applicable", |value| {
            verdict(value, object_type, expected)
        })
    };
    let read = verdict(
        ManagedObject::try_from_item(object_type, item),
        object_type,
        expected,
    );
    (construct, read)
}

/// The direct constructors have fixed argument counts and field types. A
/// malformed fixture that cannot supply those arguments is sent to the reader.
fn rebuild(code: u32, fields: &[Item]) -> Option<Result<ManagedObject, ManagedObjectError>> {
    Some(match (code, fields) {
        (1, [kind, data]) => {
            Certificate::try_new(enumeration(kind)?, bytes(data)?).map(ManagedObject::Certificate)
        }
        (2, [block]) => SymmetricKey::try_new(key_block(block)?).map(ManagedObject::SymmetricKey),
        (3, [block]) => PublicKey::try_new(key_block(block)?).map(ManagedObject::PublicKey),
        (4, [block]) => PrivateKey::try_new(key_block(block)?).map(ManagedObject::PrivateKey),
        (5, [parts, index, threshold, method, block]) => SplitKey::try_new(
            key_block(block)?,
            integer(index)?,
            SplitKeyMethod::from_raw(enumeration(method)?),
            integer(parts)?,
            integer(threshold)?,
            None,
        )
        .map(ManagedObject::SplitKey),
        (7, [kind, block]) => SecretData::try_new(enumeration(kind)?, key_block(block)?)
            .map(ManagedObject::SecretData),
        (8, [kind, data]) => {
            OpaqueObject::try_new(enumeration(kind)?, bytes(data)?).map(ManagedObject::OpaqueObject)
        }
        (9, [version, block]) => {
            PgpKey::try_new(integer(version)?, key_block(block)?).map(ManagedObject::PgpKey)
        }
        _ => return None,
    })
}

fn key_block(item: &Item) -> Option<KeyBlock> {
    KeyBlock::try_from_item(item.try_clone().ok()?).ok()
}

fn enumeration(item: &Item) -> Option<u32> {
    item.with_value(|value| match value {
        ValueView::Enumeration(raw) => Some(*raw),
        _ => None,
    })
}

fn integer(item: &Item) -> Option<i32> {
    item.with_value(|value| match value {
        ValueView::Integer(raw) => Some(*raw),
        _ => None,
    })
}

fn bytes(item: &Item) -> Option<Vec<u8>> {
    item.with_value(|value| match value {
        ValueView::ByteString(value) => Some(value.to_vec()),
        _ => None,
    })
}

fn verdict(
    value: Result<ManagedObject, ManagedObjectError>,
    expected_type: ObjectType,
    expected: &[u8],
) -> &'static str {
    let Ok(object) = value else {
        return "rejected";
    };
    let same_type = object.object_type() == expected_type;
    let mut wire = Vec::new();
    if same_type && generic_wire::encode(&object.into_item(), &mut wire).is_ok() && wire == expected
    {
        "passed"
    } else {
        "failed"
    }
}
