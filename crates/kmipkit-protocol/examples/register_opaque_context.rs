use std::error::Error;

use kmipkit_protocol::{AttributeSet, ObjectType, OpaqueObjectContext, RegisterRequest};
use kmipkit_ttlv::{Item, RawTag, Structure, Value};

fn tagged_item(tag: u32, value: Value) -> Result<Item, Box<dyn Error>> {
    let tag = RawTag::new(tag)?.try_checked()?;
    Ok(Item::new(tag, value)?)
}

fn opaque_object() -> Result<Item, Box<dyn Error>> {
    let mut opaque = Structure::new();
    opaque.try_push(tagged_item(0x0042_0059, Value::enumeration(0x8000_0001))?)?;
    opaque.try_push(tagged_item(
        0x0042_005a,
        Value::byte_string(vec![0x01, 0x02]),
    )?)?;
    tagged_item(0x0042_005b, Value::structure(opaque))
}

fn generic_request() -> Result<RegisterRequest, Box<dyn Error>> {
    Ok(RegisterRequest::new(
        ObjectType::from_raw(8),
        AttributeSet::new(),
        opaque_object()?,
    ))
}

fn managed_request() -> Result<RegisterRequest, Box<dyn Error>> {
    let attributes = AttributeSet::try_new([
        tagged_item(0x0042_002c, Value::integer(3))?,
        tagged_item(0x0042_0028, Value::enumeration(3))?,
        tagged_item(0x0042_002a, Value::integer(256))?,
    ])?;
    let request = RegisterRequest::new(ObjectType::from_raw(8), attributes, opaque_object()?)
        .with_opaque_object_context(OpaqueObjectContext::ManagedCryptographicObject);
    Ok(request)
}

fn main() -> Result<(), Box<dyn Error>> {
    let generic = generic_request()?;
    let _generic_payload = generic.into_ttlv_payload()?;

    let managed = managed_request()?;
    let _managed_payload = managed.into_ttlv_payload()?;

    println!("built generic and managed-context Register payloads");
    Ok(())
}
