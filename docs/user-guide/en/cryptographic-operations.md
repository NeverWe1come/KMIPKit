# Hash, MAC, and signature operations

KMIPKit sends Hash, MAC, MAC Verify, Sign, and Signature Verify requests to a
KMIP 2.1 server. It does not hash, calculate a MAC, sign, or verify data
locally. Each explicit `Client` call makes one exchange; the client does not
retry, split data, or continue a multipart operation for you. Request and
response fields follow OASIS KMIP Specification v2.1 §§6.1.24, 6.1.32,
6.1.33, 6.1.55, and 6.1.56, Tables 235–237, 259–264, and 334–339.

## Construct and send requests

Choose keys and Cryptographic Parameters according to application policy and
server capabilities. The open enumeration wrappers preserve the exact KMIP
value. The example uses SHA-256 (Hashing Algorithm value 6 from §11.21) for
Hash; for MAC, Sign, and verification, pass caller-selected parameter
structures or omit them only when the server can obtain them from the object.

```rust,kmipkit-test
use kmipkit_client::{Client, ClientBatchItemResponse, ClientError};
use kmipkit_protocol::{
    HashRequest, HashingAlgorithm, MacRequest, MacVerifyRequest, OperationData,
    SecretBytes, SignRequest, SignatureVerifyRequest, UniqueIdentifier,
    VerificationResponseContext,
};
use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::{Item, RawTag, Structure, Value};

fn hash_parameters() -> Structure {
    let tag = RawTag::new(0x0042_0038)
        .expect("Hashing Algorithm is a 24-bit KMIP tag")
        .try_checked()
        .expect("Hashing Algorithm is allocated");
    let item = Item::new(
        tag,
        Value::enumeration(HashingAlgorithm::from_raw(6).raw()),
    )
    .expect("the enumeration is a valid TTLV value");
    let mut parameters = Structure::new();
    parameters
        .try_push(item)
        .expect("one item fits the Cryptographic Parameters structure");
    parameters
}

#[allow(dead_code)]
fn request_hash(
    client: &mut Client,
    limits: &CodecLimits,
) -> Result<ClientBatchItemResponse, ClientError> {
    let request = HashRequest::new(hash_parameters()).with_data(
        OperationData::ByteString(SecretBytes::new(b"hash-input-example".to_vec())),
    );
    client.hash(request, limits)
}

#[allow(dead_code)]
fn request_mac(
    client: &mut Client,
    limits: &CodecLimits,
    key_id: String,
    parameters: Option<Structure>,
) -> Result<ClientBatchItemResponse, ClientError> {
    let mut request = MacRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString(key_id))
        .with_data(OperationData::ByteString(SecretBytes::new(
            b"mac-input-example".to_vec(),
        )));
    if let Some(parameters) = parameters {
        request = request.with_cryptographic_parameters(parameters);
    }
    client.mac(request, limits)
}

#[allow(dead_code)]
fn request_mac_verify(
    client: &mut Client,
    limits: &CodecLimits,
    key_id: String,
    parameters: Option<Structure>,
) -> Result<ClientBatchItemResponse, ClientError> {
    let mut request = MacVerifyRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString(key_id))
        .with_data(OperationData::ByteString(SecretBytes::new(
            b"original-data-example".to_vec(),
        )))
        .with_mac_data(SecretBytes::new(b"mac-data-example".to_vec()));
    if let Some(parameters) = parameters {
        request = request.with_cryptographic_parameters(parameters);
    }
    client.mac_verify(request, limits)
}

#[allow(dead_code)]
fn request_sign(
    client: &mut Client,
    limits: &CodecLimits,
    key_id: String,
    parameters: Option<Structure>,
) -> Result<ClientBatchItemResponse, ClientError> {
    let mut request = SignRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString(key_id))
        .with_data(OperationData::ByteString(SecretBytes::new(
            b"sign-input-example".to_vec(),
        )));
    if let Some(parameters) = parameters {
        request = request.with_cryptographic_parameters(parameters);
    }
    client.sign(request, limits)
}

#[allow(dead_code)]
fn request_signature_verify(
    client: &mut Client,
    limits: &CodecLimits,
    key_id: String,
    parameters: Option<Structure>,
) -> Result<ClientBatchItemResponse, ClientError> {
    let mut request = SignatureVerifyRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString(key_id))
        .with_data(OperationData::ByteString(SecretBytes::new(
            b"original-data-example".to_vec(),
        )))
        .with_signature_data(SecretBytes::new(b"signature-data-example".to_vec()));
    if let Some(parameters) = parameters {
        request = request.with_cryptographic_parameters(parameters);
    }
    client.signature_verify(request, limits)
}

#[allow(dead_code)]
fn hash_initial_part(
    client: &mut Client,
    limits: &CodecLimits,
) -> Result<ClientBatchItemResponse, ClientError> {
    client.hash(
        HashRequest::new(hash_parameters()).with_init_indicator(true),
        limits,
    )
}

#[allow(dead_code)]
fn hash_final_part(
    client: &mut Client,
    limits: &CodecLimits,
    correlation_from_prior_response: SecretBytes,
) -> Result<ClientBatchItemResponse, ClientError> {
    let request = HashRequest::new(hash_parameters())
        .with_correlation_value(correlation_from_prior_response)
        .with_final_indicator(true);
    client.hash(request, limits)
}

#[allow(dead_code)]
fn hash_output_length(response: &ClientBatchItemResponse) -> Option<usize> {
    response
        .outcome()
        .response()
        .hash()
        .and_then(|value| value.data())
        .map(|bytes| bytes.with_bytes(<[u8]>::len))
}

#[allow(dead_code)]
fn mac_output_length(response: &ClientBatchItemResponse) -> Option<usize> {
    response
        .outcome()
        .response()
        .mac()
        .and_then(|value| value.mac_data())
        .map(|bytes| bytes.with_bytes(<[u8]>::len))
}

#[allow(dead_code)]
fn signature_output_length(response: &ClientBatchItemResponse) -> Option<usize> {
    response
        .outcome()
        .response()
        .sign()
        .and_then(|value| value.signature_data())
        .map(|bytes| bytes.with_bytes(<[u8]>::len))
}

#[allow(dead_code)]
fn verification_result(response: &ClientBatchItemResponse) -> (u32, Option<u32>) {
    let status = response.outcome().result().status().raw();
    let indicator = response
        .outcome()
        .response()
        .mac_verify()
        .and_then(|value| value.validity_indicator())
        .map(|value| value.raw());
    (status, indicator)
}

#[allow(dead_code)]
fn signature_verify_recovered_length(response: &ClientBatchItemResponse) -> Option<usize> {
    response
        .outcome()
        .response()
        .signature_verify()
        .and_then(|value| value.recovered_data())
        .map(|bytes| bytes.with_bytes(<[u8]>::len))
}

#[allow(dead_code)]
fn context_for_direct_conversion(
    request: &SignatureVerifyRequest,
) -> VerificationResponseContext {
    request.verification_response_context()
}

fn main() {}
```

The multipart example is a caller-selected final Hash part. Supply the opaque
Correlation Value obtained from the earlier exchange; KMIPKit does not retain
or replay it automatically. Multipart operation Data fields remain absent as
required by the operation tables.

## Read results and errors

Check the KMIP Result Status before consuming operation-specific output. A
server failure remains a completed operation result. A transport, local
validation, or malformed-response failure is a `ClientError` with the
strongest available delivery state. Operation Pending remains explicit for
caller-selected asynchronous handling; it never triggers an automatic poll.

MAC Verify and Signature Verify expose the raw Validity Indicator value.
`Valid`, `Invalid`, `Unknown`, and future values remain operation results and
are not converted into local cryptographic decisions. Signature Verify may
also expose recovered Data through a borrowed callback on `SecretBytes`.

`KMIPKIT-DISC-048` remains open: Tables 263 and 338 omit Validity Indicator on
all multipart responses, while §§6.1.33 and 6.1.56 require it for the final
multipart response. The typed API requires it for single-part, rejects it for
non-final multipart, and accepts either form for final multipart without
making a server-conformance claim. Use `verification_response_context()` or
`try_from_response_item_with_context` when converting a response outside the
client association path; the convenience converter assumes single-part.

Request builders redact their Debug output and own sensitive byte inputs in
zeroizing `SecretBytes`. `with_bytes` lends received values to a callback;
copies made by the caller are outside KMIPKit's zeroization guarantee. Avoid
logging raw TTLV payloads or copying secret data into ordinary strings.
