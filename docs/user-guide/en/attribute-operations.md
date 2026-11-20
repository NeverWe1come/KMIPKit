# Attribute Operations

KMIPKit provides typed Rust request and response models for the seven client-initiated KMIP 2.1 attribute operations. The models build the operation payload; the client execution API sends it through the configured transport. See Client execution for transport setup, delivery state, and asynchronous results.

The five mutation operations also have standalone request models in C, Java, and Python. This keeps the inspectable request separate from the batch item used for submission:

| Language | Add Attribute model | Payload copy | Batch item copy |
| --- | --- | --- | --- |
| C | `kmipkit_add_attribute_request_t` | `kmipkit_add_attribute_request_to_ttlv_payload` | `kmipkit_client_batch_item_add_attribute_from_request` |
| Java | `org.kmipkit.extensions.AddAttributeRequest` | `toTtlvPayload()` | `toBatchItem()` |
| Python | `kmipkit.attribute_operations.AddAttributeRequest` | `to_ttlv_payload()` | `to_batch_item()` |

The same naming pattern applies to Adjust, Delete, Modify, and Set Attribute. Each constructor copies the supplied fields into an owned model. TTLV field accessors return independent owned copies, while identifiers, references, and raw Adjustment Type values remain directly inspectable. Converting to a batch item leaves the request model open; callers release both objects according to their language's ownership conventions.

Use `create` to build the standalone model, its payload method to inspect or encode the fields, and its batch-item conversion to add an independent copy to client execution.

| Operation | Request fields | Successful response |
| --- | --- | --- |
| Add Attribute | Optional Unique Identifier; one New Attribute | Unique Identifier |
| Adjust Attribute | Optional Unique Identifier; Attribute Reference; Adjustment Type; optional Adjustment Value | Unique Identifier |
| Delete Attribute | Optional Unique Identifier, Current Attribute, and Attribute Reference | Unique Identifier |
| Get Attributes | Optional Unique Identifier and ordered Attribute References; an empty reference list requests all attributes | Unique Identifier; Attributes is omitted when none of the requested attributes exist, while a present empty Structure remains distinguishable |
| Get Attribute List | Optional Unique Identifier | Unique Identifier and ordered Attribute References |
| Modify Attribute | Optional Unique Identifier and Current Attribute; one New Attribute | Unique Identifier |
| Set Attribute | Optional Unique Identifier; one New Attribute | Unique Identifier |

These shapes follow OASIS KMIP v2.1 §§5.5–5.7 and §6.1.2 Tables 167–169, §6.1.3 Tables 170–172, §6.1.13 Tables 202–204, §6.1.20 Tables 223–225, §6.1.21 Tables 226–228, §6.1.34 Tables 265–267, and §6.1.51 Tables 322–324.

## Build typed requests

The example creates all seven payloads. Standard attributes can use catalog-backed typed markers; the generic TTLV `Item` path remains available for extension and manufacturer tags. Current Attribute and New Attribute accept either form. The example uses the allocated `Comment` tag (`0x4200FD`).

```rust
use kmipkit_protocol::{
    AddAttributeRequest, AdjustAttributeRequest, AdjustmentType, AttributeReference,
    Attribute, AttributeSet, CurrentAttribute, DeleteAttributeRequest, GetAttributeListRequest,
    GetAttributesRequest, ModifyAttributeRequest, NewAttribute, SetAttributeRequest,
    attribute::typed,
};
use kmipkit_ttlv::{Item, RawTag, Tag, Value};

fn allocated_tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("the example uses a 24-bit KMIP tag")
        .try_checked()
        .expect("Comment is allocated in KMIP 2.1")
}

fn comment(value: &str) -> Result<Attribute<typed::Comment>, kmipkit_protocol::AttributeValueError> {
    Attribute::new(value.to_owned())
}

fn build_attribute_payloads() -> Result<(), Box<dyn std::error::Error>> {
    let object = Some("object-id-17".to_owned());
    let by_tag = AttributeReference::tag(0x0042_00FD);

    let mut attributes = AttributeSet::new();
    attributes.try_push_typed(comment("finance")?)?;
    let comment_value = attributes
        .typed::<typed::Comment>()
        .next()
        .map(|comment| comment.with_value(str::to_owned))
        .transpose()?
        .ok_or("the typed Comment was just inserted")?;
    assert_eq!(comment_value, "finance");

    // Unknown manufacturer tags remain available through the generic Item API.
    attributes.try_push(Item::new(
        allocated_tag(0x0054_1234),
        Value::byte_string(vec![0x01, 0x02]),
    )?)?;

    let add = AddAttributeRequest::new(
        object.clone(),
        NewAttribute::from_typed(comment("finance")?),
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
        Some(CurrentAttribute::from_typed(comment("legacy")?)),
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
        Some(CurrentAttribute::from_typed(comment("legacy")?)),
        NewAttribute::from_typed(comment("current")?),
    );
    let _modify_payload = modify.to_ttlv_payload()?;

    let set = SetAttributeRequest::new(
        object,
        NewAttribute::from_typed(comment("current")?),
    );
    let _set_payload = set.to_ttlv_payload()?;

    Ok(())
}
```

`AttributeReference::tag` preserves the raw tag identity; `AttributeReference::name` preserves the exact Vendor Identification and Attribute Name strings. Choose the form the server and attribute require. KMIPKit does not infer a vendor identifier from a tag.

## Preserve values and inspect results

`AttributeSet::try_push_typed` and `AttributeSet::typed` construct and read standard attributes without manually matching raw tags. Enumeration values remain open `u32` values, so values unknown to this release survive unchanged. The generic `Item` path remains available for manufacturer extensions. Both paths retain wire order and repeated instances; operation-level checks still enforce any source-defined single-instance rule. Adjustment Type supports the assigned values Increment, Decrement, and Negate plus its KMIP extension range; Reserved values are rejected for outbound typed requests.

KMIP v2.1 §4 recommends different values for multiple instances of the same attribute. The caller chooses those values; KMIPKit preserves them as supplied and does not rewrite or deduplicate them.

Response models expose the server's `KmipOperationResult`, including Result Status, optional Result Reason, and optional Result Message. Successful response payload accessors expose the operation's Unique Identifier and, for reads, the returned attributes or references. KMIPKit does not apply Adjust Attribute locally and does not maintain a local copy of server attributes.

Mutation policy checks reject only source-backed, unconditional prohibitions that can be determined from the supplied request. Those failures have delivery state `NotSent` and do not call the transport. State-dependent or unrecognized cases are sent to the server. Requests are not automatically retried. A `Pending` result retains its typed response and asynchronous correlation value; use the asynchronous workflow described in Client execution.

The protocol tests exercise derived vectors and malformed inputs. They do not claim that every KMIP server supports every attribute or that an unavailable official OASIS test case has passed.

## Fluent Set Attribute request

Rust callers can build and send a Set Attribute request through the existing typed client without assembling the New Attribute TTLV structure or a batch by hand:

```rust
use std::error::Error;

use kmipkit_client::Client;
use kmipkit_protocol::{attribute::typed, Attribute};
use kmipkit_ttlv::codec::CodecLimits;

fn set_comment(client: &mut Client) -> Result<String, Box<dyn Error>> {
    let response = client
        .request()
        .set_attribute()
        .unique_identifier("object-id-17")
        .attribute(Attribute::<typed::Comment>::new("managed by service".to_owned())?)
        .execute(&CodecLimits::defaults())?;

    response
        .outcome()
        .response()
        .set_attribute()
        .and_then(|result| result.unique_identifier())
        .map(str::to_owned)
        .ok_or_else(|| "Set Attribute did not return a successful identifier".into())
}
```

The builder validates the catalogued value and operation payload locally, supports `build`, `validate`, and `to_batch_item` when the caller needs explicit control, and exposes `attribute_item` for an explicitly typed TTLV Item. Use `vendor_attribute` to construct a KMIP Vendor Attribute structure from its vendor identifier, name, and value. The request executes through the existing client in one exchange and does not poll or retry.

## Fluent Add Attribute request

Add Attribute implements KMIP v2.1 §6.1.2, Table 167; its New Attribute uses the structure defined in §5.7, Table 163. The fluent request builds the existing typed request for adding one attribute to an identified object and accepts a catalogued `Attribute`, an explicitly typed TTLV `Item`, or a Vendor Attribute constructed from its fields:

```rust
use std::error::Error;

use kmipkit_client::Client;
use kmipkit_protocol::{attribute::typed, Attribute};
use kmipkit_ttlv::codec::CodecLimits;

fn add_comment(client: &mut Client) -> Result<String, Box<dyn Error>> {
    let response = client
        .request()
        .add_attribute()
        .unique_identifier("object-id-17")
        .attribute(Attribute::<typed::Comment>::new("reviewed by service".to_owned())?)
        .execute(&CodecLimits::defaults())?;

    response
        .outcome()
        .response()
        .add_attribute()
        .and_then(|result| result.unique_identifier())
        .map(str::to_owned)
        .ok_or_else(|| "Add Attribute did not return a successful identifier".into())
}
```

The operation requires one New Attribute. The builder validates its catalogued TTLV type and request payload locally; `build`, `validate`, and `to_batch_item` are available when a caller needs explicit control. `attribute_item` accepts a fully typed Item, while `vendor_attribute` constructs the KMIP Vendor Attribute structure.

| Language | Start the request | Add an attribute | Submit and release |
| --- | --- | --- | --- |
| C | `kmipkit_add_attribute_builder_create` | `..._add_catalogued_attribute`, `..._add_attribute_item`, or `..._add_vendor_attribute` | `..._execute_v1` or `..._build_batch_item`; release the builder and results |
| Java | `new FluentRequest(client).addAttribute()` | `cataloguedAttribute`, `attributeItem`, or `vendorAttribute` | `execute(limits)` or `build`; close the builder and results |
| Python | `fluent.Request(client).add_attribute()` | `catalogued_attribute`, `attribute_item`, or `vendor_attribute` | `execute(limits)` or `build`; use the builder as a context manager |

The exchange uses the existing client once and does not poll or retry. Client-side mutation policy checks can reject prohibited attributes before transport and report `NotSent`.

The same task-oriented builder is available from each binding's fluent request facade:

| Language | Start the request | Catalogued attribute | Expert TTLV or vendor attribute | Submit and release |
| --- | --- | --- | --- | --- |
| C | `kmipkit_set_attribute_builder_create` | `kmipkit_set_attribute_builder_add_catalogued_attribute(..., KMIPKIT_STANDARD_ATTRIBUTE_DESCRIPTION, value)` | `..._add_attribute_item` or `..._add_vendor_attribute` | `..._execute_v1` or `..._build_batch_item`; release the builder and response |
| Java | `new FluentRequest(client).setAttribute()` | `cataloguedAttribute(StandardAttribute.DESCRIPTION, value)` | `attributeItem` or `vendorAttribute` | `execute(limits)` or `build`; close the builder and owned results |
| Python | `fluent.Request(client).set_attribute()` | `catalogued_attribute(StandardAttribute.DESCRIPTION, value)` | `attribute_item` or `vendor_attribute` | `execute(limits)` or `build`; use the builder as a context manager |

Catalogued values are checked against the attribute's TTLV type. Unknown tags require an explicitly typed Item. The vendor helper constructs the protocol's Vendor Attribute Structure; a vendor tag alone is not treated as that structure. The C consumer, Java builder, and Python builder tests compile and validate these construction paths.

## Fluent Delete Attribute request

Delete Attribute follows OASIS KMIP v2.1 §6.1.13, Tables 202–204. Its Current Attribute uses the direct attribute Item defined in §5.6, Table 162, and Attribute Reference follows §5.5, Table 161. The fluent builder creates the existing typed request; it does not send arbitrary TTLV. The caller may select an attribute by its current value, by an Attribute Reference, or by both. The request validator and server remain authoritative for the combination and whether that attribute can be deleted.

Rust callers can use a catalogued attribute without constructing the Current Attribute TTLV structure or a batch:

```rust
use std::error::Error;

use kmipkit_client::Client;
use kmipkit_protocol::{attribute::typed, Attribute};
use kmipkit_ttlv::codec::CodecLimits;

fn delete_comment(client: &mut Client) -> Result<String, Box<dyn Error>> {
    let response = client
        .request()
        .delete_attribute()
        .unique_identifier("object-id-17")
        .current_attribute(Attribute::<typed::Comment>::new("legacy value".to_owned())?)
        .execute(&CodecLimits::defaults())?;

    response
        .outcome()
        .response()
        .delete_attribute()
        .and_then(|result| result.unique_identifier())
        .map(str::to_owned)
        .ok_or_else(|| "Delete Attribute did not return a successful identifier".into())
}
```

| Language | Start the request | Select the current attribute | Validate or submit |
| --- | --- | --- | --- |
| Rust | `client.request().delete_attribute()` | `current_attribute(Attribute<D>)`, `current_attribute_item(Item)`, `current_vendor_attribute(...)`, `attribute_reference(...)` | `validate`, `build_request`, `to_batch_item`, or one `execute` |
| C | `kmipkit_delete_attribute_builder_create` | `..._set_catalogued_attribute`, `..._set_current_attribute_item`, `..._set_vendor_attribute`, `..._set_attribute_reference` | `..._validate`, `..._build_request`, `..._build_batch_item`, or one `..._execute_v1`; release each owned handle |
| Java | `new FluentRequest(client).deleteAttribute()` | `cataloguedAttribute`, `currentAttribute(TtlvItem)`, `currentVendorAttribute`, `attributeReference` | `validate`, `buildRequest`, `toBatchItem`, or one `execute(limits)`; close owned values |
| Python | `fluent.Request(client).delete_attribute()` | `catalogued_attribute`, `current_attribute_item`, `current_vendor_attribute`, `attribute_reference` | `validate`, `build_request`, `to_batch_item`, or one `execute(limits)`; use the builder as a context manager |

Catalogued values are checked against the selected attribute's TTLV type. Unknown tags require an explicitly typed Item, and the vendor helper constructs the KMIP Vendor Attribute structure. Transport options use `with_options`; the Request Header Asynchronous Indicator has its own setter. Each `execute` performs one exchange through the existing client and does not poll or retry. Java and Python retain language-managed copies in addition to the native request owner; close the returned request, batch item, builder, and response according to the examples in their APIs.
