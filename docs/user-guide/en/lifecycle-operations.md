# Managed-object lifecycle operations

KMIPKit exposes explicit client calls for Activate, Archive, Destroy, Recover,
Re-certify, Re-key, and Re-key Key Pair. Each call sends one typed request,
performs one exchange, and returns the server's operation result. The client
does not simulate remote object state, retry requests, or issue Poll/Get follow-ups
automatically. For connection setup and TLS policy, see the
[production transport guide](production-transports.md).

The operation payloads follow OASIS KMIP Specification v2.1 §6.1.1 Tables
164–166 (Activate), §6.1.4 Tables 173–175 (Archive), §6.1.15 Tables 208–210
(Destroy), §6.1.42 Tables 288–290 (Recover), §6.1.46 Tables 303–307
(Re-key), and §6.1.47 Tables 308–312 (Re-key Key Pair). Unique Identifier
forms and the assigned tag are defined by §4.58 Tables 145–146 and §11.56
Table 487.

Activate, Archive, Destroy, and Recover accept an optional typed
`UniqueIdentifier`. KMIPKit sends the supplied representation as given;
omitting it does not select or discover an object locally. Successful
responses expose the server-returned identifier. Re-certify, Re-key, and
Re-key Key Pair have operation-specific identifier fields described below.
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

## Re-certify

Re-certify asks the server to renew one certificate. The caller may identify
the existing certificate explicitly or omit its Unique Identifier to use the
batch ID Placeholder. The caller supplies any Certificate Request, request
type, Offset, attributes, and protection masks; a Certificate Request Value
requires its type. If request data and Certificate Type are omitted, the
server uses the existing certificate type. The server creates and maintains
the replacement links. KMIPKit does not create certificates or track which
certificates have already been renewed, so callers should issue Re-certify no
more than once for a given existing certificate.

```rust,kmipkit-test
use kmipkit_client::{Client, ClientOperation};
use kmipkit_protocol::{RecertifyRequest, UniqueIdentifier};
use kmipkit_ttlv::codec::CodecLimits;

fn recertify(
    client: &mut Client,
    limits: &CodecLimits,
) -> Result<(), kmipkit_client::ClientError> {
    let request = RecertifyRequest::new().with_unique_identifier(
        UniqueIdentifier::TextString("certificate-42".to_owned()),
    );
    let item = client.recertify(request, limits)?;
    let outcome = item.outcome();
    assert_eq!(outcome.operation(), ClientOperation::Recertify);
    let response = outcome
        .response()
        .recertify()
        .expect("Re-certify returns its typed response");
    let _server_status = response.result().status().raw();
    let _replacement_identifier = response.unique_identifier();
    Ok(())
}

fn main() {
    let _example = recertify;
}
```

## Re-key

Re-key asks the server to create a replacement Symmetric Key. The caller may
identify the existing key explicitly or omit its Unique Identifier to use a
preceding batch item's ID Placeholder. Offset, Attributes, and Protection
Storage Masks are optional and are forwarded in KMIP-defined order. When
Offset is omitted, the server applies the standard's lifecycle-date behavior.
The server generates the replacement key and maintains its Replacement Object
and Replaced Key links. KMIPKit does not generate key material or keep a local
record of which keys were re-keyed; callers should issue Re-key no more than
once for a given existing key.

```rust,kmipkit-test
use kmipkit_client::{Client, ClientOperation};
use kmipkit_protocol::{RekeyRequest, UniqueIdentifier};
use kmipkit_ttlv::codec::CodecLimits;

fn rekey(
    client: &mut Client,
    limits: &CodecLimits,
) -> Result<(), kmipkit_client::ClientError> {
    let request = RekeyRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString("key-42".to_owned()));
    let item = client.rekey(request, limits)?;
    let outcome = item.outcome();
    assert_eq!(outcome.operation(), ClientOperation::Rekey);
    let response = outcome
        .response()
        .rekey()
        .expect("Re-key returns its typed response");
    let _server_status = response.result().status().raw();
    let _replacement_identifier = response.unique_identifier();
    Ok(())
}

fn main() {
    let _example = rekey;
}
```

## Re-key Key Pair

Re-key Key Pair asks the server to create a replacement public/private key
pair. The caller may identify the existing pair with its Private Key Unique
Identifier or omit that field to use a preceding batch item's ID Placeholder.
The caller can set an Offset and separate Common, Private Key, and Public Key
Attributes and Protection Storage Masks. If Offset is omitted, the server
copies the existing pair's Activation Date and Deactivation Date. KMIPKit does
not generate key material or maintain the replacement links; the caller should
request Re-key Key Pair no more than once for a given pair.

```rust,kmipkit-test
use kmipkit_client::{Client, ClientOperation};
use kmipkit_protocol::{RekeyKeyPairRequest, UniqueIdentifier};
use kmipkit_ttlv::codec::CodecLimits;

fn rekey_key_pair(
    client: &mut Client,
    limits: &CodecLimits,
) -> Result<(), kmipkit_client::ClientError> {
    let request = RekeyKeyPairRequest::new()
        .with_private_key_unique_identifier(UniqueIdentifier::TextString(
            "private-key-42".to_owned(),
        ))
        .with_offset(86_400);
    let item = client.rekey_key_pair(request, limits)?;
    let outcome = item.outcome();
    assert_eq!(outcome.operation(), ClientOperation::RekeyKeyPair);
    let response = outcome
        .response()
        .rekey_key_pair()
        .expect("Re-key Key Pair returns its typed response");
    let _server_status = response.result().status().raw();
    let _replacement_private_identifier = response.private_key_unique_identifier();
    let _replacement_public_identifier = response.public_key_unique_identifier();
    Ok(())
}

fn main() {
    let _example = rekey_key_pair;
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
