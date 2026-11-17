# Query and Ping

KMIP 2.1 Query lets the caller ask a server for selected protocol information.
Ping sends an empty operation payload and observes the server's KMIP result.
Both operations use the existing client execution path and perform one exchange
per call.

## Query selected server information

Create a `QueryRequest` with one or more `QueryFunction` values. Repetitions and
order are preserved. The fourteen standard values have named constants, and
valid KMIP enumeration extension values can be supplied with `from_raw`.
Reserved values and an empty function list are rejected locally before any
request bytes are sent.

`Object Groups` is optional. If supplied, it is encoded as a structure that
contains the caller's ordered `Object Group` text attributes. An empty list
encodes a present empty structure; omitting the builder leaves the field absent.
KMIPKit does not add groups or deduplicate repeated values.

The Query-specific `ClientBatchItem` builders can check the Query payload
against caller-selected codec limits before copying or encoding it. Execution
checks the complete Request Message, including its headers and batch wrappers,
against the same limits before transport exchange.

```rust,kmipkit-test
use kmipkit_client::{Client, ClientBatchItemResponse, ClientError};
use kmipkit_protocol::{QueryFunction, QueryRequest};
use kmipkit_ttlv::codec::CodecLimits;

#[allow(dead_code)]
fn query_server(
    client: &mut Client,
    limits: &CodecLimits,
) -> Result<ClientBatchItemResponse, ClientError> {
    let request = QueryRequest::new([
        QueryFunction::OPERATIONS,
        QueryFunction::OBJECTS,
        QueryFunction::SERVER_INFORMATION,
    ])
    .with_object_groups(["symmetric-keys", "certificates"]);
    client.query(request, limits)
}

fn main() {}
```

The typed response exposes each Table 283 member in wire order. Enumeration
values remain raw so values unknown to this KMIPKit release are preserved.
Complex members such as Server Information, Extension Information, and
Protection Storage Masks remain available as generic TTLV Items.

```rust,kmipkit-test
use kmipkit_client::{Client, ClientError};
use kmipkit_protocol::{QueryFunction, QueryRequest};
use kmipkit_ttlv::codec::CodecLimits;

#[allow(dead_code)]
fn query_supported_operations(
    client: &mut Client,
    limits: &CodecLimits,
) -> Result<Vec<u32>, ClientError> {
    let response = client.query(
        QueryRequest::new([QueryFunction::OPERATIONS]),
        limits,
    )?;
    let response_view = response.outcome().response();
    let query = response_view.query().expect("the request was Query");
    Ok(query.operations().collect())
}

fn main() {}
```

Query reports information returned by the server. It does not prove that the
server enforces each reported capability, that this client implements it, or
that local policy permits its use. If Query Extension List and Query Extension
Map are both requested, KMIPKit sends both values in the caller's order and
does not interpret or rewrite the server's response.

Section 6.1.40 describes an empty response payload when there are no values to
return. Table 283 also marks Protection Storage Masks required for a structured
response while allowing its list to be empty. KMIPKit accepts both the empty
payload and structured forms and preserves which form arrived. The conflict
remains open as `KMIPKIT-DISC-047`; accepting both forms is not a server
conformance determination.

## Ping

Ping has an empty request and successful response payload. A successful typed
result means the server returned a successful KMIP Ping response. It is not a
general health, readiness, or future-operation guarantee.

```rust,kmipkit-test
use kmipkit_client::{Client, ClientBatchItemResponse, ClientError};
use kmipkit_ttlv::codec::CodecLimits;

#[allow(dead_code)]
fn ping_server(
    client: &mut Client,
    limits: &CodecLimits,
) -> Result<ClientBatchItemResponse, ClientError> {
    client.request().ping().execute(limits)
}

fn main() {}
```

The Rust fluent `PingBuilder` builds the existing empty `PingRequest` with
`.build()`, checks its payload locally with `.validate()`, or converts it to a
typed batch item with `.to_batch_item()`. KMIP 2.1 §6.1.36, Tables 271–272
defines the empty request and successful response. The execution path rejects
KMIP 2.0 before sending. `.with_options(...)` sets exchange-local options;
`.with_asynchronous_indicator(1)` explicitly permits Pending and retains the
typed correlation value without polling. Batch conversion retains only the
request; configure exchange options and asynchronous behavior on the batch.
`Client::ping` remains available. The C binding offers the direct
`kmipkit_client_ping_v1` function and an opaque Ping builder for setting the
asynchronous indicator, validation, batch conversion, and execution with
default or explicit exchange options. Java uses
`client.request().ping().asynchronousIndicator(1).execute(limits)`; Python uses
`client.request().ping().asynchronous_indicator(1).execute(limits)`. Both also
provide `withOptions`/`with_options`, `validate`, `build`, `toBatchItem`/
`to_batch_item`, and one-exchange execution. All four paths use the existing
typed client and return the language's existing batch response view. None
polls or retries automatically.

## Results, errors, and retries

KMIP operation failures remain typed responses with their Result Status,
optional Result Reason, and permitted Result Message. Local validation,
protocol decoding, and transport failures return `ClientError` with the
strongest available delivery state. An empty Query function list has
`NotSent` delivery evidence.

Each `Client::query` or `Client::ping` call makes at most one exchange. The
client does not retry, poll, issue a follow-up Query, or run Discover Versions
as a hidden preflight. Callers decide whether and when to make another request.
