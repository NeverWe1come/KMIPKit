# Fluent API journeys

KMIPKit's four package consumers exercise the same five user journeys in Rust,
C, Java, and Python over a loopback TLS 1.3/mutual-TLS fixture:

1. Create a symmetric key, Get it using the returned identifier, then Destroy
   it.
2. Create an AES-256 key, Encrypt and Decrypt with explicit AES-GCM parameters,
   then Destroy the key.
3. Create an ECDSA P-256 key pair, Sign known bytes, and verify the signature
   using the returned public identifier.
4. Add the catalogued Object Group Text String attribute to Create; a negative
   Integer value is rejected locally before transport.
5. Create a symmetric key and Get it in the next item of the same ordered
   batch using the KMIP ID Placeholder.

The last journey follows KMIP Specification v2.1 §6.1 and §11.58 Table 489.
The batch has Batch Order Option set to true. Its Create item does not supply a
Unique Identifier; the Get item carries Enumeration value `1`, the ID
Placeholder. Both items carry distinct Unique Batch Item IDs, which the peer
echoes in their responses. The controlled peer assigns an identifier in the
Create response and returns that identifier from Get. The identifier is read
from the Create response only for result checks; it is never guessed or copied
into the Get request.

The examples and fixture are executable from the repository root on Linux or
macOS with:

```sh
sh tests/consumers/0093-journeys/verify.sh
```

On Windows, run `tests/consumers/0093-journeys/verify.ps1`. The runner builds
each consumer from the checked-out package sources and requires all 52 ordered
TLS connections to finish. The fixture validates decoded request fields and
returns encoded KMIP responses. This verifies the client path and controlled
exchange; it is not interoperability evidence for an external KMS.

Consumer implementations:

- Rust: `tests/consumers/0093-journeys/rust/src/main.rs`
- C: `tests/consumers/0093-journeys/c/main.c`
- Java: `tests/consumers/0093-journeys/java/src/main/java/Consumer.java`
- Python: `tests/consumers/0093-journeys/python/main.py`
- TLS 1.3/mTLS fixture: `tests/consumers/0093-journeys/local_journey_responder.py`

## KMIP 3.0 Working Draft operations

The fluent client also provides typed builders for the seven KMIP 3.0
Working Draft operations assigned by KMIPKIT-0116: Create Credential, Create
Group, Create User, Deactivate, Decapsulate, Encapsulate, and Obliterate.
Select protocol version 3.0 explicitly on the client or in request options
before execution; 2.1 remains the default. This bounded surface does not claim
general KMIP 3.0 conformance.

| Operation | Inputs handled by the builder |
| --- | --- |
| Create Credential | Ordered attributes and one typed credential alternative: Certificate, Password, Device, One Time Password, or Hashed Password. The selected alternative determines Credential Type. |
| Create Group / Create User | Ordered `AttributeSet` values. |
| Deactivate | Unique Identifier, with optional Deactivation Date and Deactivation Reason. |
| Decapsulate | Unique Identifier, ciphertext, and typed cryptographic parameters. The server-managed shared secret is not returned. |
| Encapsulate | Unique Identifier and typed cryptographic parameters. The result may expose its Unique Identifier and public Data. |
| Obliterate | Unique Identifier. |

Representative entry points for Create Group are `client.request().create_group(attributes)`
in Rust, `new FluentRequest(client).createGroup(attributes)` in Java,
`Request(client).create_group(attributes)` in Python, and
`kmipkit_client_create_group_v1(...)` in C. The other operations use
snake_case in Rust/Python and camelCase in Java. C offers fixed-width status
returning `kmipkit_client_*_v1` helpers for direct tasks and an opaque
`kmipkit_kmip30_operation_builder_t` for options and advanced fields. Each
builder validates locally, can produce a batch item, and executes through one
typed exchange. It does not retry or issue Poll automatically.

Executable route tests are kept with each binding:

- Rust: `crates/kmipkit-client/tests/unit/fluent_request_tests.rs`
- C: `bindings/c/tests/fluent_kmip30_consumer.c`
- Java: `bindings/java/src/test/java/org/kmipkit/Kmip30FluentExecutionTest.java`
- Python: `bindings/python/tests/test_fluent_kmip30.py`
