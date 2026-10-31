# Interop, Log and certificate-chain Validate

The Rust client exposes `Client::interop`, `Client::log` and
`Client::validate`. Each call makes one explicit exchange and returns a typed
batch item result. Use `ClientRequest::{Interop,Log,Validate}` in an explicit
`ClientBatch` to permit asynchronous Pending. The common
[client execution contract](client-execution.md) preserves the exact
correlation bytes, Result Status, Result Reason and permitted Result Message;
the application selects any subsequent Poll or retry.

Interop follows OASIS KMIP v2.1 §6.1.26, Tables 241–243. Both Interop Function
and Interop Identifier are required. The function remains an open raw
Enumeration: Begin is 1, End is 2 and Reset is 3 (§11.22); vendor and future
values remain intact. Success has an empty operation payload. The restriction
against production-server availability applies to the server. Implementing
the client operation does not establish an interoperability result.

The approved KMIPKIT-0042 mapping is also available through the C, Java and
Python client facades. C creates the batch item with
`kmipkit_client_batch_item_interop` and reads the complete valid response with
`kmipkit_client_batch_item_response_interop_ttlv`; release the returned
structure with `kmipkit_ttlv_structure_release`. Java uses
`ClientBatchItem.interop` and `ClientBatchItemResult.interopResponse`; close
the returned `InteropResponse`. Python uses `client_batch_item_interop` and
`BatchItemResult.interop_response`; close the returned `InteropResponse`.
The function value remains an open unsigned 32-bit Enumeration, and the
identifier must be UTF-8. The response accessor is available for valid
Success, Failure and Pending results; it does not expose a response body from
local transport, decoding or validation errors. These accessors transfer an
owned generic TTLV structure to the caller. Java and Python may retain
additional runtime-managed copies of identifier or response data.

```rust,kmipkit-test
use kmipkit_protocol::{InteropRequest, SecretText};

fn main() -> Result<(), kmipkit_protocol::ProtocolError> {
    let request = InteropRequest::new(
        1,
        SecretText::new("caller-selected-test-id".to_owned()),
    );
    let payload = request.into_ttlv_payload()?;
    assert_eq!(payload.view().children().len(), 2);
    Ok(())
}
```

Log follows §6.1.29, Tables 250–252, with Table 252 assigned to Log by the
approved DISC-017 disposition. Supply a required Log Message and expect an
empty success payload. The caller must not submit secrets to the server log.
KMIPKit owns the text as `SecretText`, redacts it from diagnostics and moves
it into the zeroizing TTLV payload without cloning it.

```rust,kmipkit-test
use kmipkit_protocol::{LogRequest, SecretText};

fn main() -> Result<(), kmipkit_protocol::ProtocolError> {
    let request = LogRequest::new(SecretText::new("test completed".to_owned()));
    let payload = request.into_ttlv_payload()?;
    assert_eq!(payload.view().children()[0].tag().raw(), 0x420141);
    Ok(())
}
```

Validate follows §6.1.57, Tables 340–342. Append Certificate structures,
Managed Certificate identifiers, or both to two lists composing one chain with
`with_certificate` and `with_unique_identifier`. Certificate structures must
contain one Certificate Type Enumeration and one Certificate Value Byte String
(§2.1, Table 2), with Certificate Type before Certificate Value. Section 10.1.2's unnumbered Type table Structure row requires Table 340 wire order: all Certificates, all Unique Identifiers, then optional Validity Date. Relative order inside each list is retained even when the builders are called in mixed order. The DER contents remain opaque. Unknown certificate fields
and Enumeration values remain intact. The client does not construct or
evaluate chains, select trust anchors, or infer server policy.

```rust,kmipkit-test
use kmipkit_protocol::{UniqueIdentifier, ValidateRequest};

fn main() -> Result<(), kmipkit_protocol::ProtocolError> {
    let request = ValidateRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString("leaf-certificate".to_owned()))
        .with_unique_identifier(UniqueIdentifier::TextString("issuer-certificate".to_owned()))
        .with_validity_date(1_700_000_000);
    let payload = request.into_ttlv_payload()?;
    assert_eq!(payload.view().children().len(), 3);
    Ok(())
}
```

Omit `with_validity_date` to leave Validity Date absent; the server assumes
its current time. No local timestamp is synthesized. The successful response
exposes `ValidateResponse::validity_indicator`, whose raw value is server
produced: Valid 1, Invalid 2, Unknown 3 (§11.61), with unknown values retained.
Failure and Pending do not fabricate an indicator. Each typed response lends
its retained generic response through `with_ttlv`, including unknown fields.

Log text, certificate contents, correlation bytes, Result Message and raw
payloads are redacted from normal formatting. Explicit inspection callbacks
can create caller-owned copies; those copies are outside KMIPKit's memory
cleanup guarantee. Java and Python runtimes may also retain external copies.
See the [public ownership contract](../../architecture/public-api.md).

The examples are tested by `diagnostics_guide_examples.rs` and can be compiled
directly with `python scripts/test_user_guide_examples.py --guide
docs/user-guide/en/diagnostics-and-interop.md`. The Rust and binding suites
are linked in [feature traceability](../../../specs/028-diagnostics-interop/traceability.md)
and the KMIPKIT-0043 evidence record. The Interop mapping is implemented in
all four languages; final PR and CI gates remain pending. No direct
official OASIS fixture, profile support, certification or two-server
interoperability claim follows from these examples.


`with_extension(Item)` preserves ordered unknown request-payload fields;
use the typed builders for fields defined by the operation tables. Batch
Message Extension envelopes use the common client extension contract.
