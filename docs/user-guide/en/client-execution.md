# Typed client execution

This guide began with the Rust execution slice introduced by KMIPKIT-0007.
Its operation list and examples are historical entry points, not the current
complete API inventory. The release branch now includes 56 client-initiated
operation mappings, the separate Set Endpoint Role route, and the approved
server-to-client listener and dispatch APIs under KMIPKIT-0066. See the
[project status](../../project-status.md), [language package and manifest
status](language-packages.md), and [server interoperability
matrix](../../development/kmip-interoperability.md) for implementation and
evidence distinctions.

This page focuses on Rust execution examples. KMIPKit's approved 1.0 surface
also includes TTLV, JSON, and XML; the synchronous client and protocol-level
asynchronous outcomes remain caller-driven. KMIP 2.1 is the default, with
catalog-gated KMIP 2.0 and a bounded KMIP 3.0 draft surface; details are
version-specific and do not imply blanket conformance.

This guide describes the Rust `kmipkit-client` API surface. A Rust model or
accessor does not establish four-language 1.0 operation availability. In the
manifest, `blocked` means the complete four-language 1.0 operation mapping is
unavailable, even if Rust types exist. See the [language and package
status](language-packages.md) for current operation and capability states.

## Typed request boundary

The synchronous Rust `kmipkit_client::Client::execute` API accepts a
`ClientBatch` containing only the closed `ClientRequest` variants. The Rust
client supports explicitly requested client-to-server Discover Versions,
Create, Create Key Pair, Create Split Key, Add Attribute, Adjust Attribute,
Delete Attribute, Get Attributes, Get Attribute List, Modify Attribute, Set
Attribute, Activate, Archive, Destroy, Recover, Encrypt, Decrypt, Query, and Ping
requests.
With default configuration, Discover Versions advertises KMIP 2.1. The
multi-version API can advertise an explicitly configured set of supported
versions; see KMIPKIT-0069 and ADR-0022. Each response is still subject to the
effective-version checks and server behavior.
Get Attributes and Get Attribute List use their typed request and response
models from §§6.1.20–6.1.21.
Passing generic TTLV `Item` or
`Structure` values, encoded message bytes, or caller-defined conversion hooks
to this API is not supported.

Discover Versions is an ordinary operation that the caller must request. The
client does not run it as a hidden preflight or use it to negotiate another
operation. The response reports versions only; it does not establish support
for other operations. An empty version list or a server's `Operation Not
Supported` result is returned as an ordinary typed outcome.

Each batch item retains its operation-specific response. Calling
`ClientBatchOutcome::response()` returns a borrowed `ClientResponseView` with
the shared `result()` accessor and typed accessors for Discover Versions,
Create, Create Key Pair, Create Split Key, Add Attribute, Adjust Attribute,
Delete Attribute, Get Attributes, Get Attribute List, Modify Attribute, Set
Attribute, Activate, Archive, Destroy, and Recover. When an operation is
Pending, its `PendingOutcome` retains that typed response and lends the opaque correlation value through
`asynchronous_correlation_value()`. KMIPKit does not poll or retry
automatically.

The four-language package consumers execute Ping through their configured
client APIs against a loopback-only TLS 1.3/mTLS fixture responder. This checks
the client execution and typed-result path; the fixture is not a KMIP server
implementation and does not provide interoperability evidence.

KMIP 2.1 remains the default. KMIP 2.0 and the catalogued KMIP 3.0
draft/standard surface are selected explicitly and checked before transport
writes under KMIPKIT-0069–0071 and ADR-0022. KMIP 1.x is not supported by the
client library. The 3.0 APIs are limited to their approved version-specific
models and routes; their presence does not claim blanket 3.0 conformance,
profile support, or certification. The initial 2.1-only execution specification
and ADR-0002 have been superseded in scope by later approved specifications and
ADRs.

## Fluent key lifecycle

KMIPKIT-0091 adds a task-oriented path for the common symmetric-key lifecycle.
It builds existing typed requests and keeps algorithm, key size, and usage
policy explicit. The AES presets set only the algorithm and bit length; select
the allowed operations yourself. Creation returns the server's typed Unique
Identifier, which is then passed to Get and Destroy:

```rust,kmipkit-test
use kmipkit_client::{Client, ClientError, CryptographicUsageMask};
use kmipkit_ttlv::codec::CodecLimits;

#[allow(dead_code)]
fn fluent_key_lifecycle(
    client: &mut Client,
    limits: &CodecLimits,
) -> Result<(), ClientError> {
let created = client
    .request()
    .create_symmetric_key()
    .aes_256()
    .usage_mask(CryptographicUsageMask::ENCRYPT | CryptographicUsageMask::DECRYPT)
    .name("application-key")
    .execute(&limits)?;
let identifier = created.outcome().create_response()
    .and_then(|response| response.unique_identifier())
    .expect("successful Create returns a Unique Identifier").clone();
let key = client.request().get(identifier.clone()).execute(&limits)?;
let destroyed = client.request().destroy(identifier).execute(&limits)?;
let _ = (key, destroyed);
Ok(())
}

fn main() {}
```

For expert extensions, Rust also accepts an explicit TTLV `Item` through
`attribute_item()`. `attribute_at(FluentAttributePosition::ObjectAttributes
{ index })` places that item at a zero-based position in the final ordered
Object Attributes list. The index includes semantic fields such as algorithm,
length, usage mask, and name, as well as append-only expert attributes. The
builder validates catalogued TTLV types, the Vendor Attribute structure, and
the final typed request before sending; duplicate or out-of-range positions
fail with delivery state `NotSent`. This path cannot address a message header,
batch item, or transport, and it does not enable arbitrary message submission.

The same scoped flow is available in Java (`client.request().createSymmetricKey()`),
Python (`client.request().create_symmetric_key()`), and C. Java and Python
builders expose `aes256()` / `aes_256()`, `usageMask()` / `usage_mask()`,
`name()`, and `execute()`, followed by `get(id)` and `destroy(id)`. Their direct
client methods also expose `createSymmetricKey`, `getById`, and `destroyById`
in Java, and `create_symmetric_key`, `get_by_id`, and `destroy_by_id` in Python.
Get and Destroy builders can also produce typed batch items (`to_batch_item()`
in Rust, `build()` / `toBatchItem()` in Java, and `build()` / `to_batch_item()`
in Python). Their asynchronous indicator is a separate request-header setting;
`with_options` remains limited to exchange-local transport options.

C provides direct one-operation helpers
`kmipkit_client_create_symmetric_key_v1`, `kmipkit_client_get_by_id_v1`, and
`kmipkit_client_destroy_by_id_v1`, plus an opaque symmetric-key builder for
expert attributes. For a compact raw-TLS/mTLS setup,
`kmipkit_client_create_mtls_v1` accepts certificate and key spans and uses the
default 8 MiB request cap when no cap is supplied. Certificate spans are read
during construction; the returned client owns its transport configuration.
The direct calls return the existing status code and owned typed response
handles. Get receives the identifier returned by Create; the bindings do not
invent one locally.

Each language has a loopback TLS 1.3/mTLS journey that sends Create, Get, and
Destroy to a controlled peer, reads the identifier from Create, and supplies it
to the following requests. This proves the client path and typed exchange
against the fixture, not interoperability with an external KMS. Pending remains
an explicit outcome; the client does not poll or retry automatically.

## Fluent Create Key Pair in Rust

KMIPKIT-0097 adds `client.request().create_key_pair()` over the existing typed
Create Key Pair request. Select the algorithm and length explicitly, then
choose independent private/public usage masks and names:

```rust,kmipkit-test
use kmipkit_client::{Client, ClientBatchItemResponse, ClientError, CryptographicUsageMask};
use kmipkit_protocol::CryptographicAlgorithm;
use kmipkit_protocol::named_values_generated::values::enumeration::cryptographic_algorithm;
use kmipkit_ttlv::codec::CodecLimits;

#[allow(dead_code)]
fn create_signing_key_pair(
    client: &mut Client,
    limits: &CodecLimits,
) -> Result<ClientBatchItemResponse, ClientError> {
    let response = client.request().create_key_pair()
        .algorithm(CryptographicAlgorithm::from_raw(cryptographic_algorithm::RSA))
        .length(3072)
        .private_usage_mask(CryptographicUsageMask::SIGN)
        .public_usage_mask(CryptographicUsageMask::VERIFY)
        .private_name("application-signing-private")
        .public_name("application-signing-public")
        .execute(limits)?;
    if let Some(pair) = response.outcome().create_key_pair_response() {
        let private_id = pair.private_key_unique_identifier();
        let public_id = pair.public_key_unique_identifier();
        let _ = (private_id, public_id);
    }
    Ok(response)
}

fn main() {}
```

The server supplies both Unique Identifiers; inspect the existing typed
`create_key_pair_response()` and its `private_key_unique_identifier()` /
`public_key_unique_identifier()` accessors when the outcome is completed.
Failure and Pending retain their existing outcome types; identifiers are
success-only, and Pending preserves the asynchronous correlation value.
There is no automatic Poll or retry. The example executes one typed exchange
and returns the complete `ClientBatchItemResponse`. The caller can inspect
Success identifiers, Failure status/reason, or Pending correlation; a successful
Rust `Result` preserves whichever KMIP outcome the server returned.

`common_attribute`, `private_attribute`, and `public_attribute` append typed
`Attribute<D>` values to their separate groups. `build()` and
`build_request()` validate and return the existing `CreateKeyPairRequest`;
`to_batch_item()` creates a typed batch item without sending it. Group and
attribute order are preserved. Local construction errors are payload-free;
missing algorithm/length produce `FluentBuildError::MissingCreateKeyPairAlgorithm`
or `FluentBuildError::MissingCreateKeyPairLength`. Invalid construction prevents
an exchange and `execute()` reports delivery state `NotSent`.

`validate()` performs the same local construction and typed request checks as
`build_request()` without consuming the builder or invoking the transport.
Prebuilt `AttributeSet` values can be appended with `common_attributes`,
`private_key_attributes`, and `public_key_attributes`:

```rust,kmipkit-test
use kmipkit_client::{Client, FluentBuildError};
use kmipkit_protocol::{AttributeSet, CreateKeyPairRequest, CryptographicAlgorithm};
use kmipkit_protocol::named_values_generated::values::enumeration::cryptographic_algorithm;

#[allow(dead_code)]
fn prepare_key_pair(
    client: &mut Client,
    common: AttributeSet,
    private: AttributeSet,
    public: AttributeSet,
) -> Result<CreateKeyPairRequest, FluentBuildError> {
    let builder = client.request().create_key_pair()
        .algorithm(CryptographicAlgorithm::from_raw(cryptographic_algorithm::RSA))
        .length(3072)
        .common_attributes(common)
        .private_key_attributes(private)
        .public_key_attributes(public);
    builder.validate()?;
    builder.build_request()
}

fn main() {}
```

Each set keeps its internal item order. Semantic attributes come first;
typed attributes and prebuilt sets follow in method-call order within their
group. Repeated set methods append. Final typed request validation rejects
duplicate or conflicting effective values before transport; it does not replace
or silently discard earlier attributes.

No usage mask, curve, key format, or protocol version is inferred by this
builder. The client retains its configured version (2.1 by default); an explicit
version override stays in `RequestOptions`. `with_options` supplies exchange
options, and `with_asynchronous_indicator` explicitly selects the request-header
indicator. This slice is Rust-only: equivalent C, Java, and Python builder
mappings remain blocked in the manifest pending their binding specifications.


## Fluent Create Split Key in Rust

KMIPKIT-0099 adds `client.request().create_split_key()` over the existing typed
Create Split Key operation. The caller explicitly selects the object type,
split method, number of parts, and threshold:

```rust,kmipkit-test
use kmipkit_client::{Client, ClientBatchOutcome, ClientBatchItemResponse, ClientError};
use kmipkit_protocol::{ObjectType, SplitKeyMethod};
use kmipkit_protocol::named_values_generated::values::enumeration::object_type;
use kmipkit_ttlv::codec::CodecLimits;

#[allow(dead_code)]
fn split_key(
    client: &mut Client,
    limits: &CodecLimits,
) -> Result<ClientBatchItemResponse, ClientError> {
    let response = client.request().create_split_key()
        .object_type(ObjectType::from_raw(object_type::SYMMETRIC_KEY))
        .method(SplitKeyMethod::XOR)
        .parts(3)
        .threshold(2)
        .execute(limits)?;
    if let ClientBatchOutcome::CreateSplitKeyCompleted(completed) = response.outcome() {
        let identifiers = completed.unique_identifiers();
        let _ = identifiers;
    }
    Ok(response)
}

fn main() {}
```

The response exposes every returned Unique Identifier in wire order. The
builder does not generate split material or choose split parameters. The
optional input identifier, Prime Field Size, Protection Storage Masks, and
Object Attributes are added only when the caller supplies them. Polynomial
Sharing Prime Field requires an explicit Prime Field Size; the typed request
validator rejects its absence locally. An empty Attributes Structure remains
present as required by KMIP.

`attribute(Attribute<D>)`, `attribute_item(Item)`, and `attributes(AttributeSet)`
support typed and expert attributes while preserving insertion order.
`vendor_attribute(...)` constructs the KMIP Vendor Attribute structure;
unknown allocated tags, including `0x54xxxx` extension tags, require an
explicit TTLV `Item` and remain unchanged as Object Attributes. They are not
reinterpreted as Vendor Attributes. `validate()` and `build_request()` perform
local typed validation without using the transport, and `to_batch_item()`
returns the existing typed batch item. `execute()` uses one typed exchange.
Success, Failure, Pending, correlation, and delivery state remain explicit; it
does not retry or Poll automatically. The client keeps its configured protocol
version and exchange options are supplied separately with `with_options`.


## Direct operation methods and per-exchange options

`Client` exposes a typed direct method for each of the 56 client-initiated
operations. Methods that perform an exchange also have a matching
`*_with_options` method accepting `RequestOptions`; the options apply to that
exchange only. A direct call sends one request and returns its one-item
response. It does not retry or issue Poll implicitly. Pending status and its
correlation value remain explicit in the returned outcome.

Use `ClientBatch` when a request needs Request Header Authentication, an
Asynchronous Indicator, multiple ordered Batch Items, or other shared header
fields. Authentication is attached to that one batch and is encoded once for
the Request Message; direct convenience methods do not select or retain
credentials.

```rust,kmipkit-test
use kmipkit_client::{Client, ClientBatch, ClientBatchItem, ClientError, ClientRequest, RequestOptions};
use kmipkit_protocol::{Authentication, GetRequest};
use kmipkit_ttlv::codec::CodecLimits;

#[allow(dead_code)]
fn direct_get(
    client: &mut Client,
    limits: &CodecLimits,
    options: &RequestOptions,
) -> Result<(), ClientError> {
    let response = client.get_with_options(GetRequest::new(), limits, options)?;
    let _outcome = response.outcome();
    Ok(())
}

#[allow(dead_code)]
fn authenticated_get(
    client: &mut Client,
    limits: &CodecLimits,
    authentication: Authentication,
) -> Result<(), ClientError> {
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::Get(GetRequest::new())))
        .with_authentication(authentication);
    let _response = client.execute(batch, limits)?;
    Ok(())
}

fn main() {}
```

`Authentication` is distinct from TLS client authentication. The first is a
caller-supplied KMIP Request Header value; the second authenticates the TLS
connection. KMIPKit does not persist KMIP credentials on `Client`.

## Server-generated creation operations

Create, Create Key Pair, and Create Split Key send caller-selected typed
requests through the same `ClientBatch` writer. Constructing a batch does not
open a connection; pass it to `Client::execute` when a production client and
transport configuration are available. Each marked example is compiled by
`python scripts/test_user_guide_examples.py`.

### Create

Supply the KMIP Object Type and an `AttributeSet`. Create always encodes the
required outer Attributes Structure, even when it has no members. KMIPKit does
not choose cryptographic attributes or protection policy.

```rust,kmipkit-test
use kmipkit_client::{ClientBatch, ClientBatchItem, ClientRequest};
use kmipkit_protocol::{AttributeSet, CreateRequest, ObjectType};

fn main() {
    let request = CreateRequest::new(ObjectType::from_raw(7), AttributeSet::new());
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::Create(request)));
    assert_eq!(batch.items().len(), 1);
}
```

### Create Key Pair

The three attribute groups stay distinct. Add only values selected by the
caller; KMIPKit does not choose an algorithm, length, parameters, or key usage.

```rust,kmipkit-test
use kmipkit_client::{ClientBatch, ClientBatchItem, ClientRequest};
use kmipkit_protocol::{AttributeSet, CreateKeyPairRequest};

fn main() {
    let request = CreateKeyPairRequest::new()
        .with_common_attributes(AttributeSet::new());
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::CreateKeyPair(request)));
    assert_eq!(batch.items().len(), 1);
}
```

### Create Split Key

Supply the object type, part count, threshold, split method, and required
Attributes Structure. The optional input Unique Identifier is sent only when
provided. For Polynomial Sharing Prime Field, KMIPKit's FR-015 policy requires
the caller to provide Prime Field Size explicitly.

```rust,kmipkit-test
use kmipkit_client::{ClientBatch, ClientBatchItem, ClientRequest};
use kmipkit_protocol::{AttributeSet, CreateSplitKeyRequest, ObjectType, SplitKeyMethod};

fn main() {
    let request = CreateSplitKeyRequest::new(
        ObjectType::from_raw(7),
        3,
        2,
        SplitKeyMethod::XOR,
        AttributeSet::new(),
    );
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::CreateSplitKey(request)));
    assert_eq!(batch.items().len(), 1);
}
```

Create Split Key may return repeated Unique Identifiers. Its request advertises
`Maximum Response Size` as the smaller of the local response byte limit and
the largest signed KMIP Integer. The client independently enforces its local
byte cap before entering the selected message decoder, including when a peer
ignores the advertised size.

The C, Java, and Python batch-item APIs expose the same three request shapes:
C uses `kmipkit_client_batch_item_create`,
`kmipkit_client_batch_item_create_key_pair`, and
`kmipkit_client_batch_item_create_split_key`; Java uses
`ClientBatchItem.create`, `createKeyPair`, and `createSplitKey`; Python uses
`extensions.client_batch_item_create`, `client_batch_item_create_key_pair`,
and `client_batch_item_create_split_key`. Create requires an Attributes
structure, which may be empty. Create Key Pair keeps its six optional attribute
and mask groups separate. Create Split Key retains signed part and threshold
values, the open method value, and the optional input identifier and prime
field bytes.

Successful response accessors preserve each Unique Identifier's original
TTLV type. C provides `kmipkit_client_batch_item_response_create`,
`..._create_key_pair`, and the bounded count/index pair
`..._create_split_key_count` / `..._create_split_key_unique_identifier_at`;
Java exposes `createResponse()`, `createKeyPairResponse()`, and
`createSplitKeyUniqueIdentifiers()`; Python exposes the corresponding
`BatchItemResult` response fields. Private and public key identifiers remain
distinct, and split-key identifiers remain in response wire order.

C's additive `kmipkit_client_batch_item_response_metadata_v2` exposes the raw
source operation and exact optional UTF-8 Result Message. The existing v1
metadata struct and function remain unchanged. The message pointer is borrowed
while the response-item handle is open; copy it before releasing the handle.
Java's `ClientBatchItemResult.metadataV2()` and Python's
`BatchItemResult.metadata_v2` copy the message into their runtime string before
the native response is closed. These external runtime copies are not guaranteed
to be zeroized when discarded. The configured complete-message limit bounds
the text (16 MiB by default). An absent message differs from a present empty
message.

Result Message is untrusted server text. It is available through explicit
metadata inspection and is omitted from default formatting, logs, errors, and
diagnostics. OASIS KMIP v2.1 §8.6, Table 399 makes it optional only when Result
Status is neither Pending nor Success; §9.17 permits a descriptive error
message. Accordingly, the shared success and Pending fixtures omit it, while
`creation.result-message-on-failure.v1` checks exact text and UTF-8 preservation
on an Operation Failed result.

The C, Java, and Python batch-item APIs expose the same three request shapes:
C uses `kmipkit_client_batch_item_create`,
`kmipkit_client_batch_item_create_key_pair`, and
`kmipkit_client_batch_item_create_split_key`; Java uses
`ClientBatchItem.create`, `createKeyPair`, and `createSplitKey`; Python uses
`extensions.client_batch_item_create`, `client_batch_item_create_key_pair`,
and `client_batch_item_create_split_key`. Create requires an Attributes
structure, which may be empty. Create Key Pair keeps its six optional attribute
and mask groups separate. Create Split Key retains signed part and threshold
values, the open method value, and the optional input identifier and prime
field bytes.

Successful response accessors preserve each Unique Identifier's original
TTLV type. C provides `kmipkit_client_batch_item_response_create`,
`..._create_key_pair`, and the bounded count/index pair
`..._create_split_key_count` / `..._create_split_key_unique_identifier_at`;
Java exposes `createResponse()`, `createKeyPairResponse()`, and
`createSplitKeyUniqueIdentifiers()`; Python exposes the corresponding
`BatchItemResult` response fields. Private and public key identifiers remain
distinct, and split-key identifiers remain in response wire order.

C's additive `kmipkit_client_batch_item_response_metadata_v2` exposes the raw
source operation and exact optional UTF-8 Result Message. The existing v1
metadata struct and function remain unchanged. The message pointer is borrowed
while the response-item handle is open; copy it before releasing the handle.
Java's `ClientBatchItemResult.metadataV2()` and Python's
`BatchItemResult.metadata_v2` copy the message into their runtime string before
the native response is closed. These external runtime copies are not guaranteed
to be zeroized when discarded. The configured complete-message limit bounds
the text (16 MiB by default). An absent message differs from a present empty
message.

Result Message is untrusted server text. It is available through explicit
metadata inspection and is omitted from default formatting, logs, errors, and
diagnostics. OASIS KMIP v2.1 §8.6, Table 399 makes it optional only when Result
Status is neither Pending nor Success; §9.17 permits a descriptive error
message. Accordingly, the shared success and Pending fixtures omit it, while
`creation.result-message-on-failure.v1` checks exact text and UTF-8 preservation
on an Operation Failed result.

## Navigating response TTLV

Each high-level batch response item exposes `raw()` for its decoded Response
Payload and `response_header_raw()` for the common Response Header. Rust returns
borrowed `StructureView`s tied to the response item. `None` means the server
sent no Response Payload; a present empty Structure remains an empty view.
Use the ordered child items and `Item::as_structure()` to reach any nested
Structure, including vendor extensions inside a Get `KeyBlock` or Query Server
Information. Unknown tags, repeated items, item types, values, and source order
remain available.

C exposes `kmipkit_client_batch_item_response_raw_ttlv` and
`kmipkit_client_batch_item_response_header_ttlv`. The returned Structure
handles are owned copies; release them with `kmipkit_ttlv_structure_release`.
The raw payload function reports an absent payload with `out_present = 0` and a
null handle. Java's `ClientBatchItemResult.raw()` returns
`Optional<TtlvStructure>` and `responseHeaderTtlv()` returns an owned
`TtlvStructure`; close each structure when finished. Python's `BatchItemResult`
provides `raw()` and `response_header_ttlv()` as owned `ttlv.Structure` values;
use their context managers or call `close()`.

Foreign-language structures are independent copies and remain usable after
the response result is closed. KMIPKit zeroizes its owned copies on release.
Copies or values made by application code remain the application's
responsibility; Java and Python runtimes do not promise deterministic wiping of
external copies. Default formatting and errors redact response values. Read
secret-bearing fields only when needed and do not log them.

`raw()` exposes the decoded TTLV tree, not the received wire bytes. The decoder
discards padding, so the API does not promise byte-for-byte reproduction of a
network response.

`Client::new` constructs a production client from an immutable client
configuration and a validated `TransportConfig`. It accepts TTLV, JSON, or
XML over raw TLS and HTTPS/HTTP 1.1; it does not accept caller-implemented
transports or raw request bytes. The immutable configuration defaults to TTLV,
and `RequestOptions` can override the encoding for one exchange. The selected
format applies to both request and response. See the
[transport guide](production-transports.md) for selection, trust configuration,
timeouts, and connection behavior.

## Credentials and attestation capability

KMIPKIT-0008 provides typed, raw-preserving Credential and Authentication
values in `kmipkit-protocol`. Authentication values contain one or more
Credentials in caller-supplied order. These are in-memory models in this
increment; the request payload does not include Authentication or Credential
data.

A typed Hashed Password Credential requires the caller's Timestamp and hashed
bytes. Hashing Algorithm is optional. If omitted, the model reports effective
SHA-256 (raw Enumeration value `6`) while leaving the field absent in the
retained TTLV tree. KMIPKit does not calculate the hash.
KMIP Specification v2.1 §9.11 requires this Timestamp to increase
monotonically. The current model preserves the supplied value but does not
retain prior timestamps or enforce that sequence. KMIPKIT-0008 OD-003 leaves
the enforcement owner, comparison scope, and clock behavior unresolved, so
KMIPKit does not claim this requirement is satisfied.

A typed Device Credential must include at least one of Device Serial Number,
Network Identifier, Machine Identifier, or Media Identifier. The caller is
responsible for choosing an identifier or combination that is actually
unique. KMIPKit does not define the comparison scope or check actual
uniqueness. Password and Device Identifier fields do not
replace this requirement; their presence does not imply non-empty text.

Synchronous and asynchronous Request Header builders emit
`Attestation Capable Indicator = True` because the Rust API can construct an
Attestation Credential. This
advertises construction capability only: it does not generate or verify
attestation evidence, submit a Credential, or predict server acceptance. There
is no per-request override. The indicator reports API construction capability
only. It does not submit a Credential, generate or verify evidence, or predict
server acceptance.

KMIPKit does not log Credential contents; Debug, Display, and validation
diagnostics redact them. `SecretText` and `SecretBytes` zeroize initialized
bytes in the KMIPKit-owned current allocation when its owner is dropped;
conversion into TTLV moves ownership without cloning. Spare or uninitialized
capacity is covered only when initialized and cleanup is verified. This does
not erase caller-made copies, older allocations left by buffer growth, copies
made from callback-borrowed views, temporary stack/register copies, or copies
retained by foreign runtimes or dependencies.

## Request metadata and batch results

The optional Request Header Time Stamp is omitted by default. If supplied by
the caller, its KMIP Date-Time value is sent exactly as supplied. The client
does not generate a timestamp from a clock or countdown timer. Client-initiated
requests omit Server Correlation Value. Client Correlation Value, if present,
is independent metadata and never identifies a batch item.

The batch getter preserves absence separately from the valid value zero. Rust
uses `ClientBatch::request_time_stamp` and returns `Option<i64>`; C uses
`kmipkit_client_batch_request_time_stamp` with a presence flag and signed
value; Java uses `ClientBatch.getRequestTimestamp()` and returns `OptionalLong`;
Python exposes `Batch.request_timestamp` as `int | None`.

KMIP Specification v2.1 §9.20 also permits a client with a countdown timer but
no real-time clock to derive relative seconds by subtracting Date attributes
from a Time Stamp. KMIPKit exposes the required server response Time Stamp and
generic TTLV Date-Time values; the application owns the timer and subtraction.
This does not synthesize an outgoing request Time Stamp.

For a multi-item batch, every request item has a Unique Batch Item ID. The
client associates each response with its request by that ID, even when the
server returns response items in a different order. Results are returned in
request order. A supplied ID must be echoed; a single-item response may omit
the ID when the request did. Duplicate, missing, unexpected, or mismatched
identities and mismatched operations fail as redacted protocol errors. Batch
Order Option controls server execution order; it does not select the response
matching rule.

Each valid item result is represented as completed or Pending. Pending is
accepted only when the request's effective Asynchronous Indicator permits it
and the response contains its required Asynchronous Correlation Value. The
correlation value is opaque: KMIPKit preserves its bytes and exposes them only
through an explicit borrowed accessor. It is redacted from formatted output,
errors, and logs, and KMIPKit-owned storage is zeroized when the Pending value
is dropped.

## Explicit asynchronous operations

`Client::execute_poll`, `Client::execute_cancel`,
`Client::execute_process`, and `Client::execute_query_async_requests` each
perform one explicit synchronous exchange. They never poll again, retry, wait,
or schedule background work. A transport error retains the same delivery-state
meaning described below; that state does not establish whether a retry is safe.

The caller constructs a `PollRequest`, `CancelRequest`, or `ProcessRequest`
with the exact bytes obtained from a pending outcome's borrowed correlation
accessor. `execute_poll` returns Pending without issuing another request. Its
successful terminal payload stays generic TTLV until the original operation's
typed response model is available; terminal Failure exposes its result and
reason without a payload. `execute_cancel` verifies that a successful response
echoes the exact requested correlation bytes and exposes the assigned or
unknown Cancellation Result. A Pending Cancel response is rejected under
OASIS KMIP v2.1 §6.1.5, Tables 176–178, and §11.7, Tables 437–438.

`execute_process` is a separate server operation. The caller selects whether
the Process request permits an asynchronous result. Every non-Failure Process
response, including Pending, carries the empty Response Payload required by
OASIS KMIP v2.1 §8.6, Table 399, and defined by §6.1.39, Table 279. Failure
has no payload under §8.6. A Pending outcome is returned to the caller;
KMIPKit does not assert that a later Poll will complete. The §6.1.39 prose
notes that Process may affect other batch items when Batch Order Option is
true, its default. KMIPKit does not claim to control those server-side effects.

`execute_query_async_requests` supports the optional correlation-value and
operation filters from OASIS KMIP v2.1 §6.1.41, Table 285. Its response payload
is exposed as generic TTLV by this feature. `KMIPKIT-DEC-015` records the
accepted interpretation of Table 286 for the separate typed mapping work in
KMIPKIT-0043 T035; the printed caption and pinned OASIS source remain
unchanged. This interpretation is not a formal conformance claim. The catalog
gap for the required Process request field in Table 278 remains open under
`OD-002`.

Java and Python expose the correlation through an explicit callback as a
read-only view backed by temporary KMIPKit-managed storage. The temporary
storage is zeroized before the callback method returns, including when the
callback raises. Any copy made by application code is caller-owned.

Each async outcome offers an operation-specific response view: `cancelResponse()`/
`cancel_response()`, `pollResponse()`/`poll_response()`,
`processResponse()`/`process_response()`, and
`queryAsyncRequestsResponse()`/`query_async_requests_response()`. Rust uses
the corresponding snake-case readers, and the C ABI has one `_response_v1`
function for each operation. A reader returns absence for another operation.
Cancel exposes the Cancellation Result; Poll and Process expose Pending state;
Process also exposes payload presence and member count. All views retain or
borrow from the outcome owner, so Java/Python views cannot be used after the
outcome is closed. Generic payload access remains available through `raw()`;
Query Asynchronous Requests deliberately leaves its payload generic because
KMIP v2.1 §6.1.41 Table 286 has a caption/body discrepancy. C correlation
spans and Java/Python callbacks read the outcome-owned bytes without storing a
second correlation copy.

## Resource limits

`CodecLimits` applies to each request execution. Its defaults are:

- Complete message size: 16 MiB.
- Nested Structure depth: 64.
- TTLV Item count, including the root Item: 100,000.

Callers can configure the message-size and Item-count limits, including zero;
Structure depth can be lowered but cannot exceed the model maximum of 64. The
same borrowed `CodecLimits` value is used for request encoding and response
decoding. Its `max_message_bytes()` is also the exact response-byte cap passed
to the transport, and the client checks the returned length before decoding.
There is no separate per-value byte limit; the complete-message limit bounds
any one value. These limits apply locally to every operation.

Discover Versions does not advertise the peer-visible Maximum Response Size
field. Create Split Key does, because its response may contain repeated
identifiers. That field is distinct from the local byte cap: the client still
rejects a response that exceeds its configured limit before decoding it.
Production connect, write, read, and total
deadlines are described in the [transport guide](production-transports.md).

## Local request validation

KMIPKit validates each typed client request before encoding it or calling the
transport. You can inspect the same local result without sending the request:

```rust
use kmipkit_client::ClientRequest;

let request = ClientRequest::discover_versions();
let report = request.validation_report();
assert!(report.is_valid());
```

The report contains stable rule codes, field paths, severity, and the exact
OASIS or KMIPKit policy reference. It never includes request values. The
validator borrows your request and does not encode or clone credentials or key
material for diagnostics. If a request fails local validation,
`Client::execute` returns a sanitized validation error with `NotSent`; the
transport receives no exchange and no request bytes.

For generic TTLV, select the request or nested structure schema explicitly
when you want KMIP field checks. KMIPKit does not guess a schema from matching
tags. It preserves unknown tags, future Enumeration values, unknown bit-mask
bits, and vendor extensions while checking known fields. Field order follows
the relevant structure description in KMIP v2.1 §10.1.2, not numeric tag
order. For managed objects, select `NestedStructureSchema::PgpKey`,
`NestedStructureSchema::SecretData`, or `NestedStructureSchema::SplitKey` to
check the corresponding object structure directly. A successful local report
does not predict server capabilities,
authorization, object state, or policy and is not a claim of profile
conformance.

## Errors, delivery state, and redaction

Local `ClientError` values distinguish validation, protocol, and transport
failures and retain only safe cause categories plus the strongest available
`RequestDeliveryState`:

- `NotSent`: no request byte was sent.
- `PossiblySent`: transmission began, but no response byte arrived.
- `ResponseStarted`: at least one response byte arrived, but no complete
  result is available.

A complete KMIP server result is returned as a server result, without local
delivery-state metadata. Delivery state does not establish that a retry is
safe. The client performs one exchange per call and never retries, fails over,
polls, or waits automatically.

When operation-specific validation fails after a complete response has been
decoded, `ClientError::response_ttlv()` returns the retained generic response.
Inspect it through `ClientErrorResponseTtlv::with_ttlv`, whose view is scoped to
the callback; call `StructureView::try_clone` inside the callback only when an
owned copy is needed. The retained response is redacted from `Debug`,
`Display`, and error sources, and its KMIPKit-owned payloads are zeroized when
the last error owner is dropped. `response_ttlv()` is absent for earlier
framing or decoding failures before a complete response exists, and for
failures unrelated to response processing. Batch association failures after a
complete response is decoded also retain the generic response. Treat an
explicitly inspected response as sensitive data.

Request and response bodies, credentials, secret key material, and opaque
asynchronous correlation values are not logged, formatted into client errors,
or retained in their exposed error-source chains. The private encoded request
owner remains alive through the synchronous exchange and zeroizes its
initialized bytes after it is dropped. This guarantee does not cover spare or
uninitialized capacity, copies made by the caller, earlier allocations not
cleared before reallocation, or copies owned by TLS, the operating system, or
third-party libraries. The approved policy and its precise limits are in
[ADR-0012](../../adr/0012-caller-requested-wire-encoding-policy.md).

## Message Extensions

KMIP §9.13 requires rejection of an unrecognized critical Message
Extension. An unknown
non-critical response extension is preserved as opaque TTLV for explicit
inspection; preservation does not interpret or validate vendor semantics.
This slice does not register or send vendor extensions. KMIPKIT-0012 owns the
immutable client extension registry and validated typed vendor-extension
adapters before the 1.0 API is frozen, as recorded in
[ADR-0013](../../adr/0013-client-extension-registry-ownership.md).

## Low-level transport contract

`kmipkit-transport::Transport::exchange` is a documented public API for direct
Rust users, but the top-level `kmipkit` facade does not re-export it and
`kmipkit-client` does not accept caller-implemented transports. The contract
exchanges caller-supplied request bytes synchronously with a caller-supplied
response-byte cap. It does not encode or validate KMIP messages, and it does
not provide the typed client's request-owner guarantee. The caller owns its
request buffer and is responsible for its validity and lifecycle.

On success, direct transport callers construct a `TransportResponse` with the public
`TransportResponse::new(Vec<u8>)` constructor; its
`as_bytes()` method lends a borrowed view, its `Debug` output is redacted, and
dropping it zeroizes the initialized bytes in its current owned allocation. The typed `Client::execute` path decodes successful wrappers and never returns raw response bodies.
Transport implementations must enforce the response cap while reading and stop
before growing response storage beyond it. They must not log or retain request
bytes after the exchange, must not retry, and must zeroize KMIPKit-owned
temporary request and partial response bytes before releasing them. A concrete
adapter must prevent response-buffer reallocation after storing bytes or
zeroize earlier allocations before release on success and error paths. These
guarantees do not cover
spare capacity, prior allocations unless cleared before reallocation, caller
copies, or external TLS/operating-system/library copies. See
[ADR-0014](../../adr/0014-public-transport-exchange-contract.md) for the exact
contract.

The production TLS and HTTPS adapters use the validated transport
configuration described in the [transport guide](production-transports.md).
The [transport architecture](../../architecture/transport-security.md)
records their security policy and connection lifecycle.

## Related guides and decisions

- [Inspecting KMIP messages](message-model.md)
- [Encrypt and Decrypt with a typed client](encrypt-decrypt.md)
- [Production TLS and HTTPS transports](production-transports.md)
- [Public API architecture](../../architecture/public-api.md)
- [KMIP 2.1 asynchronous operations review quickstart](../../../specs/009-asynchronous-operations/quickstart.md)
- [ADR-0002: KMIP 2.1 release scope](../../adr/0002-kmip-21-release-scope.md)
- [ADR-0012: caller-requested wire encoding](../../adr/0012-caller-requested-wire-encoding-policy.md)
- [ADR-0013: client extension registry ownership](../../adr/0013-client-extension-registry-ownership.md)
- [ADR-0014: public low-level transport contract](../../adr/0014-public-transport-exchange-contract.md)
