# Get and Locate Objects

Use `Locate` to ask a KMIP server for identifiers matching caller-supplied
attributes and constraints. Use `Get` to request one object by identifier.
`Locate` returns the server's matching identifiers; it does not retrieve the
objects. A later `Get` is a separate request.

These request and response shapes follow OASIS KMIP Specification v2.1:
Get is §6.1.19, Tables 220–222; Locate is §6.1.28, Tables 247–249. Pending
results and correlation values follow §§9.1–9.2 and §8.6, Table 399.

## Build a Locate request

`LocateRequest` always carries an `AttributeSet`. The set can be empty, or it
can contain validated generic TTLV attribute Items. The example supplies a
Comment attribute, caps the result count, and keeps an explicit zero offset.

```rust,kmipkit-test
use std::error::Error;

use kmipkit_protocol::{AttributeSet, LocateRequest};
use kmipkit_ttlv::{Item, RawTag, Value};

fn main() -> Result<(), Box<dyn Error>> {
    let comment_tag = RawTag::new(0x0042_00FD)?.try_checked()?;
    let attributes = AttributeSet::try_new([Item::new(
        comment_tag,
        Value::text_string("ledger".to_owned()),
    )?])?;
    let query = LocateRequest::new(attributes)
        .with_maximum_items(25)
        .with_offset_items(0);

    assert_eq!(query.attributes().len(), 1);
    assert_eq!(query.maximum_items(), Some(25));
    assert_eq!(query.offset_items(), Some(0));
    Ok(())
}
```

An empty set asks the server to match all objects. Other optional selectors
include Storage Status Mask and Object Group Member. Supply the values your
application intends to send. KMIPKit sends these criteria to the server; it
does not evaluate attributes, search local state, or sort or deduplicate the
server's returned identifiers. The response preserves their original order
and repetitions.

## Build a Get request

Unique Identifier is optional in the model. Omission is preserved for a
protocol context where the server resolves an ID Placeholder. Optional key
format, compression, wrap type, and wrapping specification are caller choices;
KMIPKit forwards them without converting or wrapping an object locally.

```rust,kmipkit-test
use kmipkit_protocol::{
    GetRequest, KeyFormatType, KeyWrapType, UniqueIdentifier,
};
use kmipkit_ttlv::Structure;

fn build_get(
    identifier: UniqueIdentifier,
    format: KeyFormatType,
    wrap_type: KeyWrapType,
    wrapping_specification: Structure,
) -> GetRequest {
    GetRequest::new()
        .with_unique_identifier(identifier)
        .with_key_format_type(format)
        .with_key_wrap_type(wrap_type)
        .with_key_wrapping_specification(wrapping_specification)
}

fn main() {
    let _ = build_get;
}
```

The raw enumeration values and wrapping structure above are inputs supplied
by the example caller, not KMIPKit defaults or recommendations. Select values
appropriate to the server and your protection policy. See the
[attribute operations guide](attribute-operations.md) for building validated
attribute Items and the [production transport guide](production-transports.md)
for constructing a client with verified TLS 1.3 mutual authentication.

## Execute a typed batch and inspect outcomes

`Client::execute` performs one exchange for the submitted batch. The example
assumes `client` was constructed as shown in the
[production transport guide](production-transports.md). It submits explicit
Get and Locate requests and reads their responses in request order. An
Asynchronous Indicator of `1` allows a valid Pending result as described in
OASIS KMIP v2.1 §9.2, Table 401.

```rust,kmipkit-test
use kmipkit_client::{
    Client, ClientBatch, ClientBatchItem, ClientError, ClientRequest,
};
use kmipkit_protocol::{
    AttributeSet, GetRequest, LocateRequest, UniqueIdentifier,
};
use kmipkit_ttlv::codec::CodecLimits;

fn execute_reads(client: &mut Client, limits: &CodecLimits) -> Result<(), ClientError> {
    let get = GetRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString("object-123".to_owned()));
    let locate = LocateRequest::new(AttributeSet::new()).with_offset_items(0);
    let batch = ClientBatch::from_items([
        ClientBatchItem::new(ClientRequest::Get(get)),
        ClientBatchItem::new(ClientRequest::Locate(locate)),
    ])
    .with_asynchronous_indicator(1);

    let response = client.execute(batch, limits)?;
    if let Some(item) = response.get(0) {
        let outcome = item.outcome();
        match outcome.result().status().known_name() {
            Some("Success") => {
                if let Some(get) = outcome.get_response() {
                    let metadata = (get.object_type(), get.unique_identifier());
                    if let Some(object) = get.object() {
                        // The Any Object is opaque; inspect its tag, not its contents.
                        let object_tag = object.tag();
                        let _ = (metadata, object_tag);
                    }
                }
            }
            Some("Operation Failed") => {
                // Error results expose the shared result, not Get-only fields.
                let reason = outcome.result().reason();
                let message = outcome.result().message();
                let _ = (reason, message);
            }
            Some("Operation Pending") => {
                // The completed-only accessor is None; the shared view retains Get.
                let pending_get = outcome.response().get();
                let correlation = outcome.asynchronous_correlation_value();
                let _ = (pending_get, correlation);
            }
            _ => {}
        }
    }

    if let Some(item) = response.get(1) {
        let outcome = item.outcome();
        match outcome.result().status().known_name() {
            Some("Success") => {
                if let Some(locate) = outcome.locate_response() {
                    let count = locate.located_items();
                    let identifiers = locate.unique_identifiers();
                    // Keep the identifiers in server-returned order, including repeats.
                    let _ = (count, identifiers);
                }
            }
            Some("Operation Failed") => {
                let reason = outcome.result().reason();
                let message = outcome.result().message();
                let _ = (reason, message);
            }
            Some("Operation Pending") => {
                let pending_locate = outcome.response().locate();
                let correlation = outcome.asynchronous_correlation_value();
                let _ = (pending_locate, correlation);
            }
            _ => {}
        }
    }
    Ok(())
}

fn main() {
    let _ = execute_reads;
}
```

`get_response()` and `locate_response()` return a typed response only for a
completed operation. The shared `outcome.response().get()` and `.locate()`
views can expose the typed response carried by Pending. A Pending outcome is
not a completed object or identifier result. Its correlation value is opaque;
KMIPKit does not poll or wait automatically. If you choose a follow-up
operation, issue it explicitly with the correlation value required by that
workflow.

For a completed Get, inspect only permitted metadata such as Object Type,
Unique Identifier, and the generic Any Object's tag or structure. Treat object
contents as secret: do not format or log the object, key material, or raw KMIP
bodies. KMIPKit-owned TTLV storage is zeroized when dropped; caller-created
copies are outside that guarantee.

Server Failure is a typed operation result with Result Status, Result Reason,
and optional Result Message, not a local `ClientError`. Inspect an untrusted
message only when needed and do not log it blindly. Local `ClientError`
values are for validation, protocol, and transport failures and retain the
request-delivery classification described in the
[client execution guide](client-execution.md). KMIPKit does not retry Get or
Locate, perform matching locally, or issue Recover follow-ups for archived
objects. Getting archived contents requires an application-directed Recover
and then Get workflow.

The marked Rust examples in this guide are compiled offline by
`python -B scripts/test_user_guide_examples.py`. Compilation demonstrates API
usage only; this guide does not claim external server interoperability,
full OASIS test-case execution, profile support, or certification.
