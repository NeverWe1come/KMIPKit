# Lifecycle and certificate operations

This guide covers the nine client-initiated KMIP 2.1 operations in the 1.0
lifecycle and certificate slice: Certify, Check, Obtain Lease, Re-certify,
Re-key, Re-key Key Pair, Re-Provision, Register, and Revoke. Each example
accepts an already configured `Client`; connection setup and TLS policy are
covered in the [production transport guide](production-transports.md).

KMIPKit sends the request once and reports the server result. It does not
create certificates or keys locally, simulate remote object state, retry,
poll, or perform follow-up Get calls. The previous lifecycle operations
Activate, Archive, Destroy, and Recover are covered in the
[managed-object lifecycle guide](lifecycle-operations.md).

## Certify

Certify forwards one certificate request to the server. Supply the public-key
identifier and request type with the opaque request bytes; KMIPKit does not
parse or generate the certificate request.

```rust,kmipkit-test
use kmipkit_client::{Client, ClientOperation};
use kmipkit_protocol::{CertifyRequest, SecretBytes, UniqueIdentifier};
use kmipkit_ttlv::codec::CodecLimits;

fn certify(
    client: &mut Client,
    limits: &CodecLimits,
    public_key_id: UniqueIdentifier,
    request_bytes: SecretBytes,
) -> Result<(), kmipkit_client::ClientError> {
    let request = CertifyRequest::new()
        .with_unique_identifier(public_key_id)
        .with_certificate_request_type(1)
        .with_certificate_request_value(request_bytes);
    let item = client.certify(request, limits)?;
    let outcome = item.outcome();
    assert_eq!(outcome.operation(), ClientOperation::Certify);
    let response = outcome.response().certify().expect("typed Certify response");
    let _status = response.result().status().raw();
    let _certificate_id = response.unique_identifier();
    Ok(())
}

fn main() { let _example = certify; }
```

## Check

Check asks the server whether the requested use is permitted. The returned
policy values can accompany a failed result, so inspect them independently of
the status and identifier.

```rust,kmipkit-test
use kmipkit_client::{Client, ClientOperation};
use kmipkit_protocol::{CheckRequest, UniqueIdentifier};
use kmipkit_ttlv::codec::CodecLimits;

fn check(client: &mut Client, limits: &CodecLimits) -> Result<(), kmipkit_client::ClientError> {
    let request = CheckRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString("object-id".to_owned()))
        .with_usage_limits_count(1)
        .with_cryptographic_usage_mask(2)
        .with_lease_time(60);
    let item = client.check(request, limits)?;
    let outcome = item.outcome();
    assert_eq!(outcome.operation(), ClientOperation::Check);
    let response = outcome.response().check().expect("typed Check response");
    let _status = response.result().status().raw();
    let _usage_count = response.usage_limits_count();
    let _usage_mask = response.cryptographic_usage_mask();
    let _lease_time = response.lease_time();
    Ok(())
}

fn main() { let _example = check; }
```

## Obtain Lease

Obtain Lease returns the server's lease interval and last-change time. KMIPKit
does not start a timer or track lease expiration for the caller.

```rust,kmipkit-test
use kmipkit_client::{Client, ClientOperation};
use kmipkit_protocol::{ObtainLeaseRequest, UniqueIdentifier};
use kmipkit_ttlv::codec::CodecLimits;

fn obtain_lease(client: &mut Client, limits: &CodecLimits) -> Result<(), kmipkit_client::ClientError> {
    let request = ObtainLeaseRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString("object-id".to_owned()));
    let item = client.obtain_lease(request, limits)?;
    let outcome = item.outcome();
    assert_eq!(outcome.operation(), ClientOperation::ObtainLease);
    let response = outcome.response().obtain_lease().expect("typed Obtain Lease response");
    let _status = response.result().status().raw();
    let _lease_seconds = response.lease_time();
    let _last_change = response.last_change_date();
    Ok(())
}

fn main() { let _example = obtain_lease; }
```

## Re-certify

Re-certify asks the server to renew one existing certificate. With no new
certificate request or type supplied, the server uses the existing
certificate's type.

```rust,kmipkit-test
use kmipkit_client::{Client, ClientOperation};
use kmipkit_protocol::{RecertifyRequest, UniqueIdentifier};
use kmipkit_ttlv::codec::CodecLimits;

fn recertify(client: &mut Client, limits: &CodecLimits) -> Result<(), kmipkit_client::ClientError> {
    let request = RecertifyRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString("certificate-id".to_owned()))
        .with_offset(86_400);
    let item = client.recertify(request, limits)?;
    let outcome = item.outcome();
    assert_eq!(outcome.operation(), ClientOperation::Recertify);
    let response = outcome.response().recertify().expect("typed Re-certify response");
    let _status = response.result().status().raw();
    let _certificate_id = response.unique_identifier();
    Ok(())
}

fn main() { let _example = recertify; }
```

## Re-key

Re-key asks the server to create a replacement symmetric key. The server
generates the key material and maintains replacement links; KMIPKit does not
do either locally.

```rust,kmipkit-test
use kmipkit_client::{Client, ClientOperation};
use kmipkit_protocol::{RekeyRequest, UniqueIdentifier};
use kmipkit_ttlv::codec::CodecLimits;

fn rekey(client: &mut Client, limits: &CodecLimits) -> Result<(), kmipkit_client::ClientError> {
    let request = RekeyRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString("key-id".to_owned()))
        .with_offset(86_400);
    let item = client.rekey(request, limits)?;
    let outcome = item.outcome();
    assert_eq!(outcome.operation(), ClientOperation::Rekey);
    let response = outcome.response().rekey().expect("typed Re-key response");
    let _status = response.result().status().raw();
    let _replacement_id = response.unique_identifier();
    Ok(())
}

fn main() { let _example = rekey; }
```

## Re-key Key Pair

Re-key Key Pair requests a server-generated replacement pair. Public and
private attributes can be supplied independently; the response exposes both
identifiers.

```rust,kmipkit-test
use kmipkit_client::{Client, ClientOperation};
use kmipkit_protocol::{RekeyKeyPairRequest, UniqueIdentifier};
use kmipkit_ttlv::codec::CodecLimits;

fn rekey_key_pair(client: &mut Client, limits: &CodecLimits) -> Result<(), kmipkit_client::ClientError> {
    let request = RekeyKeyPairRequest::new()
        .with_private_key_unique_identifier(UniqueIdentifier::TextString("private-key-id".to_owned()))
        .with_offset(86_400);
    let item = client.rekey_key_pair(request, limits)?;
    let outcome = item.outcome();
    assert_eq!(outcome.operation(), ClientOperation::RekeyKeyPair);
    let response = outcome.response().rekey_key_pair().expect("typed Re-key Key Pair response");
    let _status = response.result().status().raw();
    let _private_id = response.private_key_unique_identifier();
    let _public_id = response.public_key_unique_identifier();
    Ok(())
}

fn main() { let _example = rekey_key_pair; }
```

## Re-Provision

Re-Provision can submit a certificate request, submit a certificate, or omit
both so the server creates credentials. This example uses the parameterless
server-created form; any later Get is a separate caller action.

```rust,kmipkit-test
use kmipkit_client::{Client, ClientOperation};
use kmipkit_protocol::ReProvisionRequest;
use kmipkit_ttlv::codec::CodecLimits;

fn reprovision(client: &mut Client, limits: &CodecLimits) -> Result<(), kmipkit_client::ClientError> {
    let item = client.reprovision(ReProvisionRequest::new(), limits)?;
    let outcome = item.outcome();
    assert_eq!(outcome.operation(), ClientOperation::ReProvision);
    let response = outcome.response().reprovision().expect("typed Re-Provision response");
    let _status = response.result().status().raw();
    let _credential_id = response.unique_identifier();
    Ok(())
}

fn main() { let _example = reprovision; }
```

## Register

Register sends the caller-supplied object and attributes. The caller remains
responsible for supplying a structurally valid certificate object and its
required attributes, including Certificate Length.

```rust,kmipkit-test
use kmipkit_client::{Client, ClientOperation};
use kmipkit_protocol::{AttributeSet, ObjectType, RegisterRequest};
use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::Item;

fn register_certificate(
    client: &mut Client,
    limits: &CodecLimits,
    attributes: AttributeSet,
    certificate: Item,
) -> Result<(), kmipkit_client::ClientError> {
    let request = RegisterRequest::new(ObjectType::from_raw(1), attributes, certificate);
    let item = client.register(request, limits)?;
    let outcome = item.outcome();
    assert_eq!(outcome.operation(), ClientOperation::Register);
    let response = outcome.response().register().expect("typed Register response");
    let _status = response.result().status().raw();
    let _registered_id = response.unique_identifier();
    Ok(())
}

fn main() { let _example = register_certificate; }
```

### Registering an Opaque Object

An Opaque Object is sent as the standard Opaque Object `Structure`. Its required
`Opaque Data Type` is an Enumeration and its required `Opaque Data Value` is a
Byte String. KMIPKit preserves unknown Enumeration values and child items; it
does not inspect or interpret the data bytes.

With no context selector, Register uses generic opaque handling. This keeps
non-cryptographic Opaque Objects free of cryptographic attribute requirements.
The caller may also select `OpaqueObjectContext::Generic` explicitly. The
example uses `0x80000001`, a value in the OASIS extension range from §11.35,
Table 469; an application should use the value assigned to its extension. To
treat the object as managed cryptographic material for Register, select
`ManagedCryptographicObject`; supply direct `Cryptographic Usage Mask`,
`Cryptographic Algorithm`, and `Cryptographic Length` attributes. Algorithm
and Length must appear together. The Opaque Object has no Key Block, so its
opaque bytes are never used to infer those attributes.

```rust,kmipkit-test
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
```

This complete example is checked in CI: [register_opaque_context.rs](../../../crates/kmipkit-protocol/examples/register_opaque_context.rs).

The language bindings expose the same context choices through
`RegisterRequest.create(...).withOpaqueObjectContext(...)` in Java and
`RegisterRequest.create(...).with_opaque_object_context(...)` in Python. Both
selectors consume the previous request handle and return a new one. Creating a
request consumes its attribute and object handles; adding it to a
`ClientBatchItem` consumes the request. Close any request that is not transferred
to a batch (`RegisterRequest.close()` in Java/Python, or
`kmipkit_register_request_release` in C). C callers can select a context with
`kmipkit_register_request_with_opaque_object_context` and use
`kmipkit_client_batch_item_register` to build the executable batch item.

The context selector is local metadata: it does not add a KMIP wire field or
change the normal Register payload. Invalid local requests fail during
preflight with delivery state `NotSent`, before the transport writes bytes.
The server remains authoritative for accepting the object and its attributes;
selecting this context does not authorize later cryptographic operations.
These rules trace to OASIS KMIP 2.1 §2.3/Table 4, §6.1.43/Tables 291 and 293,
and §11.34/Table 468 (`KMIPKIT-REQ-SPEC-2.3-002` and
`KMIPKIT-REQ-SPEC-2.3-003`).

### Register response mapping

After Success, the typed response exposes the required returned Unique
Identifier and complete ordered response TTLV. Rust exposes
`RegisterResponse::unique_identifier` and `with_ttlv`; C uses
`kmipkit_client_batch_item_response_register` and
`kmipkit_client_batch_item_response_register_ttlv`; Java uses
`ClientBatchItemResult.registerResponse()` and its `uniqueIdentifier()` and
`responseTtlv()` accessors; Python uses `BatchItemResult.register_response`
with `unique_identifier` and `response_ttlv`. The C TTLV result is a caller-owned
copy. Java and Python response objects own their TTLV copy and must be closed;
closing the batch result does not invalidate that independent response copy.
For both C getters, output regions must be writable and must not overlap;
overlapping outputs return `InvalidInput` before any output is written.

Failure and Pending keep shared result status, reason/message where present,
and Pending correlation metadata. They do not expose Register's success
identifier or response TTLV. Delivery state describes a failed client exchange,
not a completed item result. Errors never include raw response TTLV. The caller
chooses whether to request asynchronous behavior; KMIPKit does not retry, issue
Poll, or send a follow-up request automatically.

## Revoke

Revoke sends an explicit reason. The optional message is for audit logging;
the compromise date is permitted only for key or CA compromise.

```rust,kmipkit-test
use kmipkit_client::{Client, ClientOperation};
use kmipkit_protocol::{RevocationReason, RevokeRequest, UniqueIdentifier};
use kmipkit_ttlv::codec::CodecLimits;

fn revoke(client: &mut Client, limits: &CodecLimits) -> Result<(), kmipkit_client::ClientError> {
    let request = RevokeRequest::new(RevocationReason::KEY_COMPROMISE)
        .with_unique_identifier(UniqueIdentifier::TextString("certificate-id".to_owned()))
        .with_revocation_message("approved retirement".to_owned())
        .with_compromise_occurrence_date(1_700_000_000);
    let item = client.revoke(request, limits)?;
    let outcome = item.outcome();
    assert_eq!(outcome.operation(), ClientOperation::Revoke);
    let response = outcome.response().revoke().expect("typed Revoke response");
    let _status = response.result().status().raw();
    let _revoked_id = response.unique_identifier();
    Ok(())
}

fn main() { let _example = revoke; }
```

The marked examples are compiled by `python scripts/test_user_guide_examples.py`.
They demonstrate API shape only and do not assert that an external KMIP server
accepts a request or grants an operation.
