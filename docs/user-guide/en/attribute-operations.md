# Attribute Operations

KMIPKit provides typed Rust request and response models for the seven client-initiated KMIP 2.1 attribute operations. The models build the operation payload; the client execution API sends it through the configured transport. See [Client execution](client-execution.md) for transport setup, delivery state, and asynchronous results.

| Operation | Request fields | Successful response |
| --- | --- | --- |
| Add Attribute | Optional Unique Identifier; one New Attribute | Unique Identifier |
| Adjust Attribute | Optional Unique Identifier; Attribute Reference; Adjustment Type; optional Adjustment Value | Unique Identifier |
| Delete Attribute | Optional Unique Identifier, Current Attribute, and Attribute Reference | Unique Identifier |
| Get Attributes | Optional Unique Identifier and ordered Attribute References; an empty reference list requests all attributes | Unique Identifier and ordered direct attribute Items |
| Get Attribute List | Optional Unique Identifier | Unique Identifier and ordered Attribute References |
| Modify Attribute | Optional Unique Identifier and Current Attribute; one New Attribute | Unique Identifier |
| Set Attribute | Optional Unique Identifier; one New Attribute | Unique Identifier |

These shapes follow OASIS KMIP v2.1 §§5.5–5.7 and §6.1.2 Tables 167–169, §6.1.3 Tables 170–172, §6.1.13 Tables 202–204, §6.1.20 Tables 223–225, §6.1.21 Tables 226–228, §6.1.34 Tables 265–267, and §6.1.51 Tables 322–324.

## Build typed requests

The example creates all seven payloads. An attribute is a generic TTLV `Item`: its tag identifies the attribute and its value retains its KMIP type. Current Attribute and New Attribute wrap that one direct Item. The example uses the allocated `Comment` tag (`0x4200FD`).

```rust
use kmipkit_protocol::{
    AddAttributeRequest, AdjustAttributeRequest, AdjustmentType, AttributeReference,
    CurrentAttribute, DeleteAttributeRequest, GetAttributeListRequest,
    GetAttributesRequest, ModifyAttributeRequest, NewAttribute, SetAttributeRequest,
};
use kmipkit_ttlv::{Item, RawTag, Tag, Value};

fn allocated_tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("the example uses a 24-bit KMIP tag")
        .try_checked()
        .expect("Comment is allocated in KMIP 2.1")
}

fn comment(value: &str) -> Item {
    Item::new(
        allocated_tag(0x0042_00FD),
        Value::text_string(value.to_owned()),
    )
    .expect("the example Item is structurally valid")
}

fn build_attribute_payloads() -> Result<(), Box<dyn std::error::Error>> {
    let object = Some("object-id-17".to_owned());
    let by_tag = AttributeReference::tag(0x0042_00FD);

    let add = AddAttributeRequest::new(
        object.clone(),
        NewAttribute::new(comment("finance")),
    );
    let _add_payload = add.to_ttlv_payload()?;

    let adjust = AdjustAttributeRequest::new(
        object.clone(),
        by_tag.clone(),
        AdjustmentType::INCREMENT,
        Some(Value::integer(1)),
    );
    let _adjust_payload = adjust.to_ttlv_payload()?;

    let delete = DeleteAttributeRequest::new(
        object.clone(),
        Some(CurrentAttribute::new(comment("legacy"))),
        None,
    );
    let _delete_payload = delete.to_ttlv_payload()?;

    let get_attributes = GetAttributesRequest::try_new(
        object.clone(),
        [by_tag.clone(), AttributeReference::name("example.org", "CostCenter")],
    )?;
    let _get_attributes_payload = get_attributes.to_ttlv_payload()?;

    let get_attribute_list = GetAttributeListRequest::new(object.clone());
    let _get_attribute_list_payload = get_attribute_list.to_ttlv_payload()?;

    let modify = ModifyAttributeRequest::new(
        object.clone(),
        Some(CurrentAttribute::new(comment("legacy"))),
        NewAttribute::new(comment("current")),
    );
    let _modify_payload = modify.to_ttlv_payload()?;

    let set = SetAttributeRequest::new(object, NewAttribute::new(comment("current")));
    let _set_payload = set.to_ttlv_payload()?;

    Ok(())
}
```

`AttributeReference::tag` preserves the raw tag identity; `AttributeReference::name` preserves the exact Vendor Identification and Attribute Name strings. Choose the form the server and attribute require. KMIPKit does not infer a vendor identifier from a tag.

## Preserve values and inspect results

`AttributeSet` retains direct Items, repeated values, and wire order. Unknown tags and enumeration values remain available through the generic TTLV model when allowed by the tag allocation policy. Adjustment Type supports the assigned values Increment, Decrement, and Negate plus its KMIP extension range; Reserved values are rejected for outbound typed requests.

Response models expose the server's `KmipOperationResult`, including Result Status, optional Result Reason, and optional Result Message. Successful response payload accessors expose the operation's Unique Identifier and, for reads, the returned attributes or references. KMIPKit does not apply Adjust Attribute locally and does not maintain a local copy of server attributes.

Mutation policy checks reject only source-backed, unconditional prohibitions that can be determined from the supplied request. Those failures have delivery state `NotSent` and do not call the transport. State-dependent or unrecognized cases are sent to the server. Requests are not automatically retried. A `Pending` result retains its typed response and asynchronous correlation value; use the asynchronous workflow described in [Client execution](client-execution.md).

The protocol tests exercise derived vectors and malformed inputs. They do not claim that every KMIP server supports every attribute or that an unavailable official OASIS test case has passed.
