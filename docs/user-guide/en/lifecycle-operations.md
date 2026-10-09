# Managed-object lifecycle operations

KMIPKit exposes explicit client calls for Activate, Archive, Destroy, and
Recover. Each call sends one typed request, performs one exchange, and returns
the server's operation result. The client does not simulate remote object
state, retry requests, or issue Poll/Get follow-ups automatically. For
connection setup and TLS policy, see the [production transport guide](production-transports.md).

The operation payloads follow OASIS KMIP Specification v2.1 §6.1.1 Tables
164–166 (Activate), §6.1.4 Tables 173–175 (Archive), §6.1.15 Tables 208–210
(Destroy), and §6.1.42 Tables 288–290 (Recover). Unique Identifier forms and
the assigned tag are defined by §4.58 Tables 145–146 and §11.56 Table 487.

All four requests accept an optional typed `UniqueIdentifier`. KMIPKit sends
the supplied representation as given; omitting it does not select or discover
an object locally. Successful responses expose the server-returned identifier.
Non-success responses preserve their result without interpreting a success
payload as an object state.

## Activate

Activate requests that the server activate the identified object. The client
returns the server result and the required response identifier on Success; it
does not claim the server's object state has changed locally.

```rust,kmipkit-test
use kmipkit_client::{Client, ClientOperation};
use kmipkit_protocol::{ActivateRequest, UniqueIdentifier};
use kmipkit_ttlv::codec::CodecLimits;

fn activate(
    client: &mut Client,
    limits: &CodecLimits,
) -> Result<(), kmipkit_client::ClientError> {
    let request = ActivateRequest::new(Some(UniqueIdentifier::TextString(
        "archive-record-42".to_owned(),
    )));
    let item = client.activate(request, limits)?;
    let outcome = item.outcome();
    assert_eq!(outcome.operation(), ClientOperation::Activate);
    let response = outcome
        .response()
        .activate()
        .expect("Activate returns its typed response");
    let _server_status = response.result().status().raw();
    let _returned_identifier = response.unique_identifier();
    Ok(())
}

fn main() {
    let _example = activate;
}
```

## Archive

Archive expresses the caller's archival preference. A successful response is
the server's result; it is not evidence that archival has completed. Server
policy determines what the request means for that installation.

```rust,kmipkit-test
use kmipkit_client::{Client, ClientOperation};
use kmipkit_protocol::{ArchiveRequest, UniqueIdentifier};
use kmipkit_ttlv::codec::CodecLimits;

fn archive(
    client: &mut Client,
    limits: &CodecLimits,
) -> Result<(), kmipkit_client::ClientError> {
    let request = ArchiveRequest::new(Some(UniqueIdentifier::TextString(
        "archive-record-42".to_owned(),
    )));
    let item = client.archive(request, limits)?;
    let outcome = item.outcome();
    assert_eq!(outcome.operation(), ClientOperation::Archive);
    let response = outcome
        .response()
        .archive()
        .expect("Archive returns its typed response");
    let _server_status = response.result().status().raw();
    let _server_identifier = response.unique_identifier();
    Ok(())
}

fn main() {
    let _example = archive;
}
```

## Destroy

Destroy requests server-side destruction. KMIPKit reports the response and
does not infer that an object or its metadata was removed.

```rust,kmipkit-test
use kmipkit_client::{Client, ClientOperation};
use kmipkit_protocol::{DestroyRequest, UniqueIdentifier};
use kmipkit_ttlv::codec::CodecLimits;

fn destroy(
    client: &mut Client,
    limits: &CodecLimits,
) -> Result<(), kmipkit_client::ClientError> {
    let request = DestroyRequest::new(Some(UniqueIdentifier::TextString(
        "archive-record-42".to_owned(),
    )));
    let item = client.destroy(request, limits)?;
    let outcome = item.outcome();
    assert_eq!(outcome.operation(), ClientOperation::Destroy);
    let response = outcome
        .response()
        .destroy()
        .expect("Destroy returns its typed response");
    let _server_status = response.result().status().raw();
    let _server_identifier = response.unique_identifier();
    Ok(())
}

fn main() {
    let _example = destroy;
}
```

## Recover

Recover requests server-side recovery of an object and returns the result and,
on Success, the server-returned identifier. A later Get or Poll is a separate
caller action.

```rust,kmipkit-test
use kmipkit_client::{Client, ClientOperation};
use kmipkit_protocol::{RecoverRequest, UniqueIdentifier};
use kmipkit_ttlv::codec::CodecLimits;

fn recover(
    client: &mut Client,
    limits: &CodecLimits,
) -> Result<(), kmipkit_client::ClientError> {
    let request = RecoverRequest::new(Some(UniqueIdentifier::TextString(
        "archive-record-42".to_owned(),
    )));
    let item = client.recover(request, limits)?;
    let outcome = item.outcome();
    assert_eq!(outcome.operation(), ClientOperation::Recover);
    let response = outcome
        .response()
        .recover()
        .expect("Recover returns its typed response");
    let _server_status = response.result().status().raw();
    let _recovered_identifier = response.unique_identifier();
    Ok(())
}

fn main() {
    let _example = recover;
}
```

## Results, pending work, and errors

Inspect `item.outcome().result()` for the exact raw result status, optional raw
reason, and optional Result Message. Result Message text is available only via
its explicit accessor; Debug and default error formatting redact it. Unknown
status and reason values remain available as raw values. Accepted non-critical
Message Extensions remain available through `item.extensions()` as opaque
generic TTLV; this does not validate their vendor-specific meaning.

The convenience methods above use default batch options. To permit
`Operation Pending`, use `Client::execute` with a one-item `ClientBatch` whose
Asynchronous Indicator is explicitly set to Mandatory or Optional. A Pending
result exposes the exact opaque correlation bytes through its borrowed
accessor. The caller decides whether to issue Poll or another operation;
KMIPKit does not retry, poll, or wait automatically. See the
[client execution guide](client-execution.md) for Pending and delivery-state
details.

Local validation, protocol, and transport failures return a redacted
`ClientError` with `NotSent`, `PossiblySent`, or `ResponseStarted` delivery
evidence. That evidence does not establish that retrying is safe. Server
Failure is a complete operation result and is not a local error.

All marked examples in this guide are compiled by
`python scripts/test_user_guide_examples.py`.
