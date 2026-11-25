# 03. KMIP 2.0 / 2.1 / 3.0 Protocol & Operations Guide

KMIPKit targets **OASIS KMIP 2.1 by default**, with explicit version-gated support for **KMIP 2.0** and **KMIP 3.0** (including NIST Post-Quantum Cryptography primitives).

---

## 1. Protocol Version & Wire Encoding Selection

```rust
use kmipkit::client::extension_registry::{ClientConfiguration, client_extension_registry};
use kmipkit::client::{RequestOptions, WireEncoding};
use kmipkit::protocol::ProtocolVersion;

let registry = client_extension_registry(Vec::new(), kmipkit::protocol::extension::defaults())?;

// Configure default version (KMIP 3.0) and default encoding (TTLV)
let config = ClientConfiguration::new(registry)
    .with_protocol_version(ProtocolVersion::V3_0)
    .with_wire_encoding(WireEncoding::Ttlv);

// Override wire encoding for a single request (JSON over HTTPS)
let json_opts = RequestOptions::default().with_wire_encoding(WireEncoding::Json);
```

| Wire Encoding | Default Content-Type (HTTPS) | Raw TLS Framing | Best Use Case |
| :--- | :--- | :--- | :--- |
| **TTLV** (default) | `application/octet-stream` | 8-byte Tag/Type/Length header + 8-byte padding | Native HSMs, lowest latency, smallest payload |
| **JSON** | `application/json` | Incremental balanced-brace document scanner | Cloud gateways, REST/HTTPS inspection, debugging |
| **XML** | `text/xml` | Incremental DTD-free XML element scanner | Enterprise XML/SOAP environments |

---

## 2. Complete Operation Coverage Matrix (59 Operations)

### Client-to-Server (C2S) Operations

| Category | Operations | KMIP Version |
| :--- | :--- | :--- |
| **Discovery & Health** | `Discover Versions`, `Query`, `Ping`, `Interop`, `Log` | 2.0 / 2.1 / 3.0 |
| **Key & Object Creation** | `Create`, `Create Key Pair`, `Create Split Key`, `Register`, `Import`, `Derive Key`, `Join Split Key` | 2.0 / 2.1 / 3.0 |
| **Lifecycle & Rotation** | `Activate`, `Deactivate` (3.0), `Revoke`, `Destroy`, `Obliterate` (3.0), `Archive`, `Recover`, `Re-key`, `Re-key Key Pair`, `Re-Provision` | 2.0 / 2.1 / 3.0 |
| **Certificates** | `Certify`, `Re-certify`, `Validate` | 2.0 / 2.1 / 3.0 |
| **Retrieval & Search** | `Locate`, `Get`, `Export`, `Check`, `Obtain Lease`, `Get Usage Allocation` | 2.0 / 2.1 / 3.0 |
| **Attribute Management** | `Get Attributes`, `Get Attribute List`, `Set Attribute`, `Add Attribute`, `Modify Attribute`, `Adjust Attribute`, `Delete Attribute` | 2.0 / 2.1 / 3.0 |
| **Cryptographic Operations** | `Encrypt`, `Decrypt`, `Sign`, `Signature Verify`, `MAC`, `MAC Verify`, `Hash`, `RNG Retrieve`, `RNG Seed` | 2.0 / 2.1 / 3.0 |
| **Post-Quantum KEM (3.0)** | `Encapsulate`, `Decapsulate` (`ML-KEM-512/768/1024`, `X25519MLKEM768`, `SECP256R1MLKEM768`) | **3.0** |
| **Session & Access** | `Login`, `Logout`, `Delegated Login`, `Create User` (3.0), `Create Group` (3.0), `Create Credential` (3.0) | 2.0 / 2.1 / 3.0 |
| **Policy & Admin** | `Get Constraints`, `Set Constraints`, `Set Defaults`, `PKCS#11`, `Set Endpoint Role` | 2.0 / 2.1 / 3.0 |
| **Asynchronous Control** | `Poll`, `Cancel`, `Process`, `Query Asynchronous Requests` | 2.0 / 2.1 / 3.0 |

### Server-to-Client (S2C) Operations

| Operation | Purpose | Transport Support |
| :--- | :--- | :--- |
| **`Discover Versions`** | Server probes client's supported KMIP versions | Raw TLS Listener, HTTPS Listener, Role-Reversed TLS |
| **`Query`** | Server queries client capabilities & namespaces | Raw TLS Listener, HTTPS Listener, Role-Reversed TLS |
| **`Notify`** | Server pushes attribute/lifecycle change notifications | Raw TLS Listener, HTTPS Listener, Role-Reversed TLS |
| **`Put`** | Server proactively pushes new/replacement keys or certs to client | Raw TLS Listener, HTTPS Listener, Role-Reversed TLS |
| **`Set Endpoint Role`** | Negotiates role inversion on an existing TLS connection | Raw TLS Role-Reversed Stream |

---

## 3. Atomic Multi-Operation Batches & `ID Placeholder`

Execute multiple dependent KMIP operations in a single network round-trip using `ClientBatch` and `UniqueIdentifier::id_placeholder()`:

```rust
use kmipkit::client::{ClientBatch, ClientBatchItem, ClientRequest};
use kmipkit::protocol::{ActivateRequest, CreateRequest, GetRequest, ObjectType, UniqueIdentifier};

let mut batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::create(
    CreateRequest::builder(ObjectType::SymmetricKey).build()?,
)));

// Item 2: Activate the key just created in Item 1 using ID Placeholder
batch.push(ClientBatchItem::new(ClientRequest::activate(
    ActivateRequest::new(UniqueIdentifier::id_placeholder()),
)));

// Item 3: Export/Get the activated key in the same atomic exchange
batch.push(ClientBatchItem::new(ClientRequest::get(
    GetRequest::new(UniqueIdentifier::id_placeholder()),
)));

let outcome = client.execute(&batch)?;
assert_eq!(outcome.response().items().len(), 3);
```

---

## 4. Asynchronous Operations (`Poll`, `Cancel`, `Process`)

When a KMIP server returns `ResultStatus::OperationPending` with an `Asynchronous Correlation Value`, KMIPKit exposes explicit follow-up methods (never hidden background threads):

```rust
use kmipkit::protocol::{CancelRequest, PollRequest};

// Explicit Poll
let poll_outcome = client.poll(&PollRequest::new(correlation_value.clone()))?;

// Explicit Cancel if the caller decides to abort
let cancel_outcome = client.cancel(&CancelRequest::new(correlation_value))?;
```

---

## 5. KMIP 3.0 Post-Quantum Cryptography (`ML-KEM` & `ML-DSA`)

When configured with `ProtocolVersion::V3_0`, KMIPKit supports NIST FIPS 203 (`ML-KEM`), FIPS 204 (`ML-DSA`), and FIPS 205 (`SLH-DSA`):

```rust
// Generate an ML-KEM-768 key pair and perform Encapsulate / Decapsulate
let kp = client
    .create_key_pair()
    .algorithm(CryptographicAlgorithm::MlKem768)
    .send()?;

let encap = client.encapsulate(kp.public_key_unique_identifier().unwrap().clone()).send()?;
let decap = client
    .decapsulate(
        kp.private_key_unique_identifier().unwrap().clone(),
        encap.ciphertext().unwrap(),
    )
    .send()?;
```

---

## 6. Vendor Extensions & Criticality Handling

KMIPKit implements OASIS **Message Extension** criticality rules:
* **Unknown non-critical extensions (`Criticality Indicator = false`):** preserved verbatim in the generic TTLV tree.
* **Unknown critical extensions (`Criticality Indicator = true`):** rejected with a protocol error per OASIS KMIP specification.
* **Registered Vendor Extensions:** register data-only `ExtensionDefinition` schemas in the immutable `ClientExtensionRegistry` to validate vendor tags, enumerations, and structures across Rust, C, Java, and Python without modifying core code.
